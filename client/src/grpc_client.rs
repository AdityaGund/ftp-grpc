use chrono::Utc;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio_stream::{iter, StreamExt};
use uuid::Uuid;

use crate::error::AppError;

pub mod ftp {
    tonic::include_proto!("ftp");
}

use ftp::{
    transfer_service_client::TransferServiceClient, AttachmentInfo, FileInfo, MessageInfo,
    Metadata, TransferRequest,
};

const CHUNK_SIZE: usize = 1024 * 1024; // 1 MB

pub async fn transfer_data(
    client: &mut TransferServiceClient<tonic::transport::Channel>,
    file_details: Option<(&str, &str)>, // (path, name)
    message_content: Option<&str>,
) -> Result<(), AppError> {
    if file_details.is_none() && message_content.is_none() {
        return Err(AppError::ClientError(
            "No file or message content provided.".to_string(),
        ));
    }

    let transfer_id = Uuid::new_v4().to_string();
    let mut requests = vec![];

    let has_file = file_details.is_some();
    let has_message = message_content.is_some();

    // Determine payload type based on what is being sent
    let payload_type = match (has_file, has_message) {
        (true, true) => {
            let (file_path, file_name) = file_details.unwrap();
            let file_size = Path::new(file_path).metadata()?.len();
            let message_length = message_content.unwrap().len() as u64;
            Some(ftp::metadata::PayloadType::AttachmentInfo(
                AttachmentInfo {
                    file_info: Some(FileInfo {
                        name: file_name.to_string(),
                        path: file_path.to_string(),
                        size: file_size,
                        content_type: "application/octet-stream".to_string(),
                    }),
                    message_info: Some(MessageInfo {
                        length: message_length,
                    }),
                },
            ))
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
            Some(ftp::metadata::PayloadType::MessageInfo(MessageInfo {
                length: message_length,
            }))
        }
        _ => None,
    };

    // If there's a message, create the first request for it.
    if let Some(content) = message_content {
        let metadata = Metadata {
            transfer_id: transfer_id.clone(),
            sender_bank_id: "BANK_A".to_string(),
            receiver_bank_id: "BANK_C".to_string(),
            chunk_index: 0,
            total_chunks: if has_file { 0 } else { 1 }, // Will be updated later for files
            timestamp: Utc::now().to_rfc3339(),
            payload_type: payload_type.clone(),
        };
        requests.push(TransferRequest {
            metadata: Some(metadata),
            content: content.as_bytes().to_vec(),
        });
    }

    // If there is a file, read it and create chunked requests.
    if let Some((file_path, _)) = file_details {
        let mut file = File::open(file_path).await?;
        let file_size = file.metadata().await?.len();
        let total_chunks = (file_size as f64 / CHUNK_SIZE as f64).ceil() as i32;
        let mut chunk_index = 0;

        // If there was a message, update its total_chunks metadata
        if let Some(first_req) = requests.get_mut(0) {
            if let Some(metadata) = first_req.metadata.as_mut() {
                metadata.total_chunks = total_chunks;
            }
        }
        
        loop {
            let mut buffer = vec![0; CHUNK_SIZE];
            let n = file.read(&mut buffer).await?;
            if n == 0 {
                break;
            }
            buffer.truncate(n);

            chunk_index += 1;

            let metadata = Metadata {
                transfer_id: transfer_id.clone(),
                sender_bank_id: "BANK_A".to_string(),
                receiver_bank_id: "BANK_C".to_string(),
                chunk_index,
                total_chunks,
                timestamp: Utc::now().to_rfc3339(),
                payload_type: payload_type.clone(),
            };

            requests.push(TransferRequest {
                metadata: Some(metadata),
                content: buffer,
            });
        }
    }

    let request_stream = iter(requests);
    let mut response_stream = client.transfer(request_stream).await?.into_inner();

    while let Some(response) = response_stream.next().await {
        match response {
            Ok(res) => println!("[CLIENT] Received response: {:?}", res),
            Err(err) => eprintln!("[CLIENT] Error in response stream: {}", err),
        }
    }

    Ok(())
}