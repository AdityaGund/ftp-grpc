use chrono::Utc;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio_stream::{iter, StreamExt};
use tokio::time::{self, Duration};
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
const MAX_RETRIES: usize = 3;
const ACK_TIMEOUT: Duration = Duration::from_secs(10);


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
    // let mut requests = vec![];

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
        // if let Some(msg_content) = message_content {
        //     requests.push(TransferRequest {
        //         metadata: Some(Metadata {
        //             transfer_id: transfer_id.clone(),
        //             sender_bank_id: "BANK_A".to_string(),
        //             receiver_bank_id: destination.unwrap().to_string(),
        //             chunk_index: 1,
        //             total_chunks: 1,
        //             timestamp: Utc::now().to_rfc3339(),
        //             payload_type: payload_type.clone(),
        //         }),
        //         content: msg_content.as_bytes().to_vec(),
        //     });
        // }
        if let Some(msg_content) = message_content {
            file_chunks.push(msg_content.as_bytes().to_vec());
            total_chunks = 1;
        }
    }

    // turn all our file chunks into requests
    // if !file_chunks.is_empty() {
    //     for (i, chunk) in file_chunks.into_iter().enumerate() {
    //         let metadata = Metadata {
    //             transfer_id: transfer_id.clone(),
    //             sender_bank_id: "BANK_A".to_string(),
    //             receiver_bank_id: destination.unwrap().to_string(),
    //             chunk_index: (i + 1) as i32,
    //             total_chunks,
    //             timestamp: Utc::now().to_rfc3339(),
    //             payload_type: payload_type.clone(),
    //         };
    //         requests.push(TransferRequest {
    //             metadata: Some(metadata),
    //             content: chunk,
    //         });
    //     }
    // }

    // /*
    //     Below part  we are sending the requests and handling the responses which will be acks 
    //  */
    // let request_stream = iter(requests);
    // let mut response_stream = client
    //     .transfer(request_stream)
    //     .await
    //     .map_err(|e| AppError::ClientError(e.to_string()))?
    //     .into_inner();

    // // print out what the server says back
    // while let Some(response) = response_stream.next().await {
    //     match response {
    //         Ok(res) => {
    //             println!(
    //                 "[CLIENT] Received ACK  as server got msg: transfer_id={}, chunk_index={}, status={}",
    //                 res.transfer_id, res.chunk_index, res.status
    //             );
    //         }
    //         Err(err) => {
    //             eprintln!("[CLIENT] Error in response stream: {}", err);
    //             return Err(AppError::ClientError(err.to_string()));
    //         }
    //     }
    // }

    
    for (i, chunk) in file_chunks.into_iter().enumerate() {
        let chunk_index = (i + 1) as i32;
        let mut retries = 0;
        let mut success = false;
    
        while retries < MAX_RETRIES && !success {
            let request = TransferRequest {
                metadata: Some(Metadata {
                    transfer_id: transfer_id.clone(),
                    sender_bank_id: "BANK_A".to_string(),
                    receiver_bank_id: destination.unwrap().to_string(),
                    chunk_index,
                    total_chunks,
                    timestamp: Utc::now().to_rfc3339(),
                    payload_type: payload_type.clone(),
                }),
                content: chunk.clone(),
            };
    
            let request_stream = tokio_stream::iter(vec![request]);
            let mut response_stream = client
                .transfer(request_stream)
                .await
                .map_err(|e| AppError::TonicStatus(e))?
                .into_inner();
    
            // Read ALL responses for this chunk until the stream ends
            let mut chunk_success = false;
            let mut final_error: Option<String> = None;
            
            // Set a timeout for receiving all responses for this chunk
            let timeout_future = time::sleep(ACK_TIMEOUT);
            tokio::pin!(timeout_future);
            
            loop {
                tokio::select! {
                    response = response_stream.next() => {
                        match response {
                            Some(Ok(res)) => {
                                println!(
                                    "[CLIENT] Received response: transfer_id={}, chunk_index={}, status={}",
                                    res.transfer_id, res.chunk_index, res.status
                                );
    
                                if res.chunk_index == chunk_index || res.chunk_index == 0 {
                                    match res.status {
                                        status if status == ftp::Status::Success as i32 => {
                                            if res.chunk_index == chunk_index {
                                                println!("[CLIENT] Chunk {} acknowledged successfully", chunk_index);
                                                chunk_success = true;
                                            } else {
                                                println!("[CLIENT] Transfer completed successfully");
                                                chunk_success = true;
                                            }
                                        }
                                        status if status == ftp::Status::Failure as i32 => {
                                            let error_info = res.error_info.unwrap_or_default();
                                            final_error = Some(format!(
                                                "Server error - Code: {}, Details: {}",
                                                error_info.error_code,
                                                error_info.error_details
                                            ));
                                            println!(
                                                "[CLIENT] Server reported failure for chunk {}: {}",
                                                chunk_index,
                                                final_error.as_ref().unwrap()
                                            );
                                            break; // Exit the response reading loop
                                        }
                                        status if status == ftp::Status::Retry as i32 => {
                                            let error_info = res.error_info.unwrap_or_default();
                                            println!(
                                                "[CLIENT] Server requested retry for chunk {}: Code: {}, Details: {}",
                                                chunk_index,
                                                error_info.error_code,
                                                error_info.error_details
                                            );
                                            break; // Exit the response reading loop to retry
                                        }
                                        _ => {
                                            final_error = Some(format!(
                                                "Unexpected status {} for chunk {}",
                                                res.status, chunk_index
                                            ));
                                            break;
                                        }
                                    }
                                }
                            }
                            Some(Err(e)) => {
                                final_error = Some(format!("Response stream error: {}", e));
                                println!("[CLIENT] Error in response stream: {}", e);
                                break;
                            }
                            None => {
                                // Stream ended
                                println!("[CLIENT] Response stream ended for chunk {}", chunk_index);
                                break;
                            }
                        }
                    }
                    _ = &mut timeout_future => {
                        println!("[CLIENT] Timeout waiting for responses for chunk {}", chunk_index);
                        final_error = Some("Timeout waiting for server response".to_string());
                        break;
                    }
                }
            }
    
            // Determine if we should retry or consider this chunk successful
            if let Some(error) = final_error {
                retries += 1;
                if retries >= MAX_RETRIES {
                    return Err(AppError::ClientError(format!(
                        "Failed to send chunk {} after {} retries. Last error: {}",
                        chunk_index, MAX_RETRIES, error
                    )));
                }
                println!(
                    "[CLIENT] Retrying chunk {} (attempt {}/{}): {}",
                    chunk_index, retries, MAX_RETRIES, error
                );
                time::sleep(Duration::from_secs(1)).await;
            } else if chunk_success {
                success = true;
            } else {
                // No explicit error but also no success - treat as timeout/failure
                retries += 1;
                if retries >= MAX_RETRIES {
                    return Err(AppError::ClientError(format!(
                        "Failed to send chunk {} after {} retries: No clear success confirmation",
                        chunk_index, MAX_RETRIES
                    )));
                }
                println!(
                    "[CLIENT] No clear success for chunk {} (attempt {}/{})",
                    chunk_index, retries, MAX_RETRIES
                );
                time::sleep(Duration::from_secs(1)).await;
            }
        }
    
        if !success {
            return Err(AppError::ClientError(format!(
                "Failed to send chunk {} after {} retries",
                chunk_index, MAX_RETRIES
            )));
        }
    }
    

    Ok(())
}