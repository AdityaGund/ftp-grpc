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

pub use ftp::{
    transfer_service_client::TransferServiceClient, AttachmentInfo, FileInfo, MessageInfo,
    Metadata, TransferRequest,
};

const CHUNK_SIZE: usize = 1024 * 1024; // 1 MB

pub async fn transfer_data(
    client: &mut TransferServiceClient<tonic::transport::Channel>,
    file_details: Option<(&str, &str)>, // (path, name)
    message_content: Option<&str>,
    destination: Option<&str>,
) -> Result<(), AppError> {

    // can't do much without a file or a message
    if file_details.is_none() && message_content.is_none() {
        return Err(AppError::ClientError(
            "No file or message content provided.".to_string(),
        ));
    }

    let transfer_id = Uuid::new_v4().to_string();
    let mut requests = vec![];

    let has_file = file_details.is_some();
    let has_message = message_content.is_some();

    // figure out what kind of data we're sending
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

    let mut file_chunks: Vec<Vec<u8>> = Vec::new();
    let mut total_chunks = 0;
    // if there's a file, chop it up into chunks
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
    
    // if we have a message and a file, merge the message and the first chunk together
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
            requests.push(TransferRequest {
                metadata: Some(Metadata {
                    transfer_id: transfer_id.clone(),
                    sender_bank_id: "BANK_A".to_string(),
                    receiver_bank_id: destination.unwrap().to_string(),
                    chunk_index: 1,
                    total_chunks: 1,
                    timestamp: Utc::now().to_rfc3339(),
                    payload_type: payload_type.clone(),
                }),
                content: msg_content.as_bytes().to_vec(),
            });
        }
    }

    // turn all our file chunks into requests
    if !file_chunks.is_empty() {
        for (i, chunk) in file_chunks.into_iter().enumerate() {
            let metadata = Metadata {
                transfer_id: transfer_id.clone(),
                sender_bank_id: "BANK_A".to_string(),
                receiver_bank_id: destination.unwrap().to_string(),
                chunk_index: (i + 1) as i32,
                total_chunks,
                timestamp: Utc::now().to_rfc3339(),
                payload_type: payload_type.clone(),
            };
            requests.push(TransferRequest {
                metadata: Some(metadata),
                content: chunk,
            });
        }
    }

    let request_stream = iter(requests);
    let mut response_stream = client.transfer(request_stream).await?.into_inner();

    // print out what the server says back
    while let Some(response) = response_stream.next().await {
        match response {
            Ok(res) => println!("[CLIENT] Received response: {:?}", res),
            Err(err) => eprintln!("[CLIENT] Error in response stream: {}", err),
        }
    }

    Ok(())
}