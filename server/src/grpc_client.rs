use chrono::Utc;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio_stream::{Stream, StreamExt};
use uuid::Uuid;
use std::pin::Pin;
use tokio::sync::mpsc;
use crate::error::AppError;
use crate::ftp as ftp;
pub use crate::ftp::transfer_service_client::TransferServiceClient;
use crate::ftp::{AttachmentInfo, FileInfo, MessageInfo, Metadata, TransferRequest, TransferResponse};

const CHUNK_SIZE: usize = 1024 * 1024; // 1 MB
const MAX_RETRIES: u8 = 3;

pub async fn transfer_data(
    client: &mut TransferServiceClient<tonic::transport::Channel>,
    file_details: Option<(&str, &str)>, // (path, name)
    message_content: Option<&str>,
    destination: Option<&str>,
    destination_ip: Option<&str>,
    sender: Option<&str>
) -> Result<(), AppError> {
    // Implementation identical to client crate

    if file_details.is_none() && message_content.is_none() {
        return Err(AppError::ClientError(
            "No file or message content provided.".to_string(),
        ));
    }

    let transfer_id = Uuid::new_v4().to_string();

    let has_file = file_details.is_some();
    let has_message = message_content.is_some();

    let payload_type = match (has_file, has_message) {
        (true, true) => {
            let (file_path, file_name) = file_details.unwrap();
            let file_size = Path::new(file_path).metadata()?.len();
            let message_length = message_content.unwrap().len() as u64;
            Some(ftp::metadata::PayloadType::AttachmentInfo(AttachmentInfo {
                file_info: Some(FileInfo {
                    name: file_name.to_string(),
                    path: file_path.to_string(),
                    size: file_size,
                    content_type: "application/octet-stream".to_string(),
                }),
                message_info: Some(MessageInfo { length: message_length }),
            }))
        }
        (true, false) => {
            let (file_path, file_name) = file_details.unwrap();
            let file_size = Path::new(file_path).metadata()?.len();
            Some(ftp::metadata::PayloadType::FileInfo(FileInfo {
                name: file_name.to_string(),
                path: file_path.to_string(),
                size: file_size,
                content_type: "application/octet-stream".to_string(),
            }))
        }
        (false, true) => {
            let message_length = message_content.unwrap().len() as u64;
            Some(ftp::metadata::PayloadType::MessageInfo(MessageInfo { length: message_length }))
        }
        _ => None,
    };

    // Split file into chunks
    let mut file_chunks: Vec<Vec<u8>> = Vec::new();
    let mut total_chunks = 0;
    if let Some((file_path, _)) = file_details {
        let mut file = File::open(file_path).await?;
        let file_size = file.metadata().await?.len();
        total_chunks = (file_size as f64 / CHUNK_SIZE as f64).ceil() as i32;

        loop {
            let mut buffer = vec![0; CHUNK_SIZE];
            let n = file.read(&mut buffer).await?;
            if n == 0 {
                break;
            }
            buffer.truncate(n);
            file_chunks.push(buffer);
        }
    }

    // Combine message + file logic
    let mut standalone_message: Option<Vec<u8>> = None;
    if has_message && has_file {
        if let Some(msg_content) = message_content {
            let mut first_chunk = msg_content.as_bytes().to_vec();
            first_chunk.extend_from_slice(b"---MESSAGE_END---");
            if !file_chunks.is_empty() {
                first_chunk.extend_from_slice(&file_chunks.remove(0));
            }
            file_chunks.insert(0, first_chunk);
        }
    } else if has_message {
        if let Some(msg_content) = message_content {
            standalone_message = Some(msg_content.as_bytes().to_vec());
        }
    }

    let (mut req_tx, req_rx) = mpsc::channel::<TransferRequest>(1);
    let response_stream = client
        .transfer(tokio_stream::wrappers::ReceiverStream::new(req_rx))
        .await?
        .into_inner();

    let mut response_stream: Pin<Box<dyn Stream<Item = Result<TransferResponse, tonic::Status>> + Send>> = Box::pin(response_stream);

    async fn send_with_retry(
        req_tx: &mut mpsc::Sender<TransferRequest>,
        req: TransferRequest,
        responses: &mut Pin<Box<dyn Stream<Item = Result<TransferResponse, tonic::Status>> + Send>>,    ) -> Result<(), AppError> {
        for attempt in 1..=MAX_RETRIES {
            req_tx.send(req.clone()).await.map_err(|e| AppError::ClientError(format!("Failed to send request: {e}")))?;
            match responses.next().await {
                Some(Ok(_ack)) => {
                    return Ok(());
                }
                Some(Err(e)) => {
                    if attempt == MAX_RETRIES {
                        return Err(AppError::ClientError(format!("Stream error after retries: {e}")));
                    }
                }
                None => {
                    if attempt == MAX_RETRIES {
                        return Err(AppError::ClientError("Stream closed".into()));
                    }
                }
            }
        }
        Err(AppError::ClientError("Exceeded max retries".into()))
    }

    if let Some(msg_bytes) = standalone_message {
        let meta = Metadata {
            transfer_id: transfer_id.clone(),
            sender_bank_id: sender.unwrap_or("").to_string(),
            receiver_bank_id: destination.unwrap_or("").to_string(),
            receiver_bank_ip: destination_ip.unwrap_or("").to_string(),
            chunk_index: 1,
            total_chunks: 1,
            timestamp: Utc::now().format("%Y-%m-%d %H:%M:%S%.3f %Z").to_string(),
            payload_type: payload_type.clone(),
        };
        let req = TransferRequest { metadata: Some(meta), content: msg_bytes };
        send_with_retry(&mut req_tx, req, &mut response_stream).await?;
    }

    if !file_chunks.is_empty() {
        for (idx, chunk) in file_chunks.into_iter().enumerate() {
            let meta = Metadata {
                transfer_id: transfer_id.clone(),
                sender_bank_id: sender.unwrap_or("").to_string(),
                receiver_bank_id: destination.unwrap_or("").to_string(),
                receiver_bank_ip: destination_ip.unwrap_or("").to_string(),
                chunk_index: (idx + 1) as i32,
                total_chunks,
                timestamp: Utc::now().format("%Y-%m-%d %H:%M:%S%.3f %Z").to_string(),
                payload_type: payload_type.clone(),
            };
            let req = TransferRequest { metadata: Some(meta), content: chunk.clone() };
            send_with_retry(&mut req_tx, req, &mut response_stream).await?;
        }
    }

    drop(req_tx);
    while let Some(res) = response_stream.next().await {
        match res {
            Ok(resp) => {
                if resp.status == ftp::Status::Success as i32 || resp.status == ftp::Status::Failure as i32 {
                    break;
                }
            }
            Err(status) => return Err(AppError::TonicStatus(status)),
        }
    }

    Ok(())
} 