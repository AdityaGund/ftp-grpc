use chrono::Utc;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio_stream::{Stream, StreamExt};
use uuid::Uuid;
use std::pin::Pin;
// use tokio::sync::broadcast;

use crate::error::AppError;

pub mod ftp {
    tonic::include_proto!("ftp");
}

pub use ftp::{
    transfer_service_client::TransferServiceClient, AttachmentInfo, FileInfo, MessageInfo,
    Metadata, TransferRequest, TransferResponse,
};

const CHUNK_SIZE: usize = (1024 * 1024) * 5; // 1 MB
const MAX_RETRIES: u8 = 3;

pub async fn transfer_data(
    client: &mut TransferServiceClient<tonic::transport::Channel>,
    file_details: Option<(&str, &str)>, // (path, name)
    message_content: Option<&str>,
    destination: Option<&str>,
    destination_ip: Option<&str>,
    sender: Option<&str>
    // notifier: Option<broadcast::Sender<TransferResponse>>,
) -> Result<String, AppError> {

    // can't do much without a file or a message
    if file_details.is_none() && message_content.is_none() {
        return Err(AppError::ClientError(
            "No file or message content provided.".to_string(),
        ));
    }

    let transfer_id = Uuid::new_v4().to_string();

    // === Build metadata (reused for every chunk) =====================
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

    // Split the file into chunks =================================================
    let mut file_chunks: Vec<Vec<u8>> = Vec::new();
    let mut total_chunks = 0;
    if let Some((file_path, _)) = file_details {
        let mut file = File::open(file_path).await?;
        let file_size = file.metadata().await?.len();
        total_chunks = (file_size as f64 / CHUNK_SIZE as f64).ceil() as i32;

        let mut i = 1;
        loop {
            let mut buffer = vec![0; CHUNK_SIZE];
            let n = file.read(&mut buffer).await?;
            if n == 0 {
                break;
            }
            buffer.truncate(n);
            file_chunks.push(buffer);
            println!("[CLIENT] created chunk {i}");
            i += 1;
        }
    }

    // Combine message with first chunk when both present -------------------------
    let mut standalone_message: Option<Vec<u8>> = None;
    if has_message && has_file {
        if let Some(msg_content) = message_content {
            // making sure message comes first and then file.
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

    // === Build a channel-backed request stream =================================
    use tokio::sync::mpsc;
    let (mut req_tx, req_rx) = mpsc::channel::<TransferRequest>(1);
    let response_stream = client
        .transfer(tokio_stream::wrappers::ReceiverStream::new(req_rx))
        .await?
        .into_inner();

    // send_with_retry requires multiple mutable references to response stream
    // the fn can be called from different contexts (i.e. for standalone message or file)
    let mut response_stream: Pin<Box<dyn Stream<Item = Result<TransferResponse, tonic::Status>> + Send>> = Box::pin(response_stream);

    async fn send_with_retry(
        req_tx: &mut mpsc::Sender<TransferRequest>,
        req: TransferRequest,
        responses: &mut Pin<Box<dyn Stream<Item = Result<TransferResponse, tonic::Status>> + Send>>,
        // notifier: &Option<broadcast::Sender<TransferResponse>>,
    ) -> Result<(), AppError> {
        for attempt in 1..=MAX_RETRIES {
            if attempt > 1 {
                println!("[CLIENT] Retry attempt {} for chunk {}", attempt, req.metadata.as_ref().map_or(0, |m| m.chunk_index));
            }

            req_tx.send(req.clone()).await.map_err(|e| AppError::ClientError(format!("Failed to send request over stream: {e}")))?;

            match responses.next().await {
                Some(Ok(ack)) => {
                    let origin = ack.error_info.as_ref().map(|e| e.error_code.as_str()).unwrap_or("UNKNOWN");
                    println!("[CLIENT] ACK received from {}. status: {} (chunk {} )", origin, ack.status, ack.transfer_id);
                    // if let Some(tx) = notifier {
                    //     let _ = tx.send(ack.clone());
                    // }
                    return Ok(());
                }
                Some(Err(e)) => {
                    if attempt == MAX_RETRIES {
                        return Err(AppError::ClientError(format!("Stream error after retries: {e}")));
                    } else {
                        println!("[CLIENT] Error on chunk send: {e}. Retrying...");
                    }
                }
                None => {
                    if attempt == MAX_RETRIES {
                        return Err(AppError::ClientError("Stream closed unexpectedly: MAX_RETRIES".into()));
                    } else {
                        println!("[CLIENT] Stream closed unexpectedly, retrying...");
                    }
                }
            }
        }
        Err(AppError::ClientError("Exceeded max retries".into())) //This is REDUNDANT
    }

    // === Send message-only if applicable =======================================
    if let Some(msg_bytes) = standalone_message {
        let meta = Metadata {
            transfer_id: transfer_id.clone(),
            sender_bank_id: sender.unwrap().to_string(),
            receiver_bank_id: destination.unwrap().to_string(),
            receiver_bank_ip: destination_ip.unwrap().to_string(),
            chunk_index: 1,
            total_chunks: 1,
            timestamp: Utc::now().format("%Y-%m-%d %H:%M:%S%.3f %Z").to_string(),
            payload_type: payload_type.clone(),
        };
        let req = TransferRequest {
            metadata: Some(meta),
            content: msg_bytes,
        };
        // send_with_retry(&mut req_tx, req, &mut response_stream, &notifier).await?;
        send_with_retry(&mut req_tx, req, &mut response_stream).await?;
    }

    // === Send file chunks sequentially =========================================
    if !file_chunks.is_empty() {
        for (idx, chunk) in file_chunks.into_iter().enumerate() {
            let meta = Metadata {
                transfer_id: transfer_id.clone(),
                sender_bank_id: sender.unwrap().to_string(),
                receiver_bank_id: destination.unwrap().to_string(),
                receiver_bank_ip: destination_ip.unwrap().to_string(),
                chunk_index: (idx + 1) as i32,
                total_chunks,
                timestamp: Utc::now().format("%Y-%m-%d %H:%M:%S%.3f %Z").to_string(),
                payload_type: payload_type.clone(),
            };
            println!("[CLIENT] Sending chunk {}/{}", idx + 1, total_chunks);
            let req = TransferRequest {
                metadata: Some(meta),
                content: chunk.clone(),
            };

            // send_with_retry(&mut req_tx, req, &mut response_stream, &notifier).await?;
            send_with_retry(&mut req_tx, req, &mut response_stream).await?;
        }
    }

    drop(req_tx);
    /*
        THIS WAS NOT HANDLING ALL THE RESPONSES FROM TEH SERVER 
     */
    // Wait for the final SUCCESS response from the server -----------------------
    // while let Some(res) = response_stream.next().await {
    //     match res {
    //         Ok(resp) => {
    //             let origin = resp
    //                 .error_info
    //                 .as_ref()
    //                 .map(|e| e.error_code.as_str())
    //                 .unwrap_or("UNKNOWN");
    //             println!("[CLIENT] ACK received from {}. status: {}", origin, resp.status);
    //             if resp.status == ftp::Status::Success as i32 || resp.status == ftp::Status::Failure as i32 {
    //                 break;
    //             }
    //         }
    //         Err(e) => {
    //             println!("Server failed to transfer data to destination!");
    //             return Err(AppError::TonicStatus(e));
    //         }
    //     }
    // }
    /*
        THIS WILL HANDLE ALL  RESPONSES FROM THE SERVER NOW ..
     */
    let mut time_received_at = String::new();

    while let Some(res) = response_stream.next().await {
        match res {
            Ok(resp) => {
                let origin = resp
                    .error_info
                    .as_ref()
                    .map(|e| e.error_code.as_str())
                    .unwrap_or("UNKNOWN");
    
                println!("[CLIENT] ACK received from {}. status: {}", origin, resp.status);
    
                match ftp::Status::try_from(resp.status) {
                    Ok(ftp::Status::InProgress) => {
                        // Continue receiving chunks
                    }
                    Ok(ftp::Status::Success) => {
                        println!("[CLIENT] Transfer completed successfully.");
                        time_received_at = resp.time_received_at.clone();
                        break;
                    }
                    Ok(ftp::Status::Failure) => {
                        let details = resp
                            .error_info
                            .as_ref()
                            .map(|e| e.error_details.as_str())
                            .unwrap_or("No additional error details");
    
                        println!("[CLIENT] Transfer failed! Error from {}: {}", origin, details);
                        break;
                    }
                    Ok(other) => {
                        println!("[CLIENT] Unhandled status {:?} received from {}", other, origin);
                        break;
                    }
                    Err(e) => {
                        println!("[CLIENT] Unknown status code received from {}: {:?}", origin, e);
                        break;
                    }
                }
                // if let Some(tx) = &notifier {
                //     let _ = tx.send(resp.clone());
                // }
            }
    
            Err(status) => {
                // This handles tonic::Status errors like connection failures or server-side early exits
                println!(
                    "[CLIENT] Error received from server: code = {}, message = {}",
                    status.code(),
                    status.message()
                );
    
                // Decide what to do based on the error type
                match status.code() {
                    tonic::Code::NotFound => {
                        println!("[CLIENT] Bank mapping not found on server. Terminating.");
                    }
                    tonic::Code::Internal => {
                        println!("[CLIENT] Internal server error. Terminating.");
                    }
                    tonic::Code::Unavailable => {
                        println!("[CLIENT] Destination unavailable. Terminating.");
                    }
                    _ => {
                        println!("[CLIENT] Unexpected error. Terminating.");
                    }
                }
    
                // Close the connection or clean up here if needed
                return Err(AppError::TonicStatus(status));
            }
        }
    }
    

    Ok(time_received_at)
}