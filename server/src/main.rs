use crate::ftp::{
    TransferRequest, TransferResponse, transfer_service_client::TransferServiceClient,
};
use chrono::Utc;
use dotenv::dotenv;
use ftp::transfer_service_server::{TransferService, TransferServiceServer};
use std::{
    collections::HashMap,
    env,
    error::Error,
    net::SocketAddr,
    path::{Path, PathBuf},
    pin::Pin,
};
use tokio::sync::mpsc;
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::{Stream, StreamExt};
use tonic::{metadata, transport::Server, Request, Response, Status, Streaming};
// use uuid::Uuid;

pub mod ftp {
    tonic::include_proto!("ftp");
}

#[derive(Debug, Clone)]
pub struct FileTransferService {
    bank_mappings: HashMap<String, String>, // Maps bank_id to server URL
}

impl FileTransferService {
    pub fn new() -> Self {
        let mut mappings = HashMap::new();
        mappings.insert("BANK_C".to_string(), "http://127.0.0.1:50052".to_string());
        mappings.insert("BANK_D".to_string(), "http://127.0.0.1:50053".to_string());

        Self {
            bank_mappings: mappings,
        }
    }

    // A new function to forward just a message
    async fn forward_message(
        &self,
        message_content: Vec<u8>,
        original_metadata: ftp::Metadata,
        receiver_bank_id: &str,
    ) -> Result<impl Stream<Item = Result<TransferResponse, Status>>, Status> {
        let destination_url = self.bank_mappings.get(receiver_bank_id).ok_or_else(|| {
            tonic::Status::not_found(format!(
                "No server mapping found for bank: {}",
                receiver_bank_id
            ))
        })?;

        println!(
            "[SERVER] Forwarding message to bank {} at {}",
            receiver_bank_id, destination_url
        );

        let mut client = TransferServiceClient::connect(destination_url.clone())
            .await
            .map_err(|e| {
                tonic::Status::internal(format!("Failed to connect to destination: {}", e))
            })?;

        let request = TransferRequest {
            metadata: Some(original_metadata),
            content: message_content,
        };

        let request_stream = tokio_stream::iter(vec![request]);
        let response = client.transfer(request_stream).await?;
        Ok(response.into_inner())
    }

    // Function to forward the file to the destination by creating a new stream
    async fn forward_file(
        &self,
        file_path: &Path,
        original_metadata: ftp::Metadata,
        receiver_bank_id: &str,
    ) -> Result<impl Stream<Item = Result<TransferResponse, Status>>, Status> {
        let destination_url = self.bank_mappings.get(receiver_bank_id).ok_or_else(|| {
            tonic::Status::not_found(format!(
                "No server mapping found for bank: {}",
                receiver_bank_id
            ))
        })?;

        println!(
            "[SERVER] Forwarding to bank {} at {}",
            receiver_bank_id, destination_url
        );

        let mut client = TransferServiceClient::connect(destination_url.clone())
            .await
            .map_err(|e: tonic::transport::Error| {
                println!("error!! {e}");
                tonic::Status::internal(format!("Failed to connect to destination: {}", e))
            })?;
            
        const CHUNK_SIZE: usize = 1024 * 1024; // 1MB

        let mut file = fs::File::open(file_path).await.map_err(|e| {
            Status::internal(format!("Failed to open temp file for forwarding: {}", e))
        })?;

        let file_size = file
            .metadata()
            .await
            .map_err(|e| Status::internal(format!("Failed to read temp file metadata: {}", e)))?
            .len();

        let total_chunks = (file_size as f64 / CHUNK_SIZE as f64).ceil() as i32;

        // write file as chunks
        let mut requests = Vec::new();
        for i in 0..total_chunks {
            let mut buffer = vec![0; CHUNK_SIZE];
            let n = file.read(&mut buffer).await.map_err(|e| {
                Status::internal(format!("Failed to read chunk from temp file: {}", e))
            })?;
            buffer.truncate(n);

            let mut metadata = original_metadata.clone();
            metadata.chunk_index = i + 1;
            metadata.total_chunks = total_chunks;
            metadata.timestamp = Utc::now().to_rfc3339();

            requests.push(TransferRequest {
                metadata: Some(metadata),
                content: buffer,
            });
        }

        // transfer to destination
        let request_stream = tokio_stream::iter(requests);
        let response = client.transfer(request_stream).await?;
        Ok(response.into_inner())
    }
}

#[tonic::async_trait]
impl TransferService for FileTransferService {
    type TransferStream = Pin<Box<dyn Stream<Item = Result<TransferResponse, Status>> + Send>>;

    async fn transfer(
        &self,
        request: Request<Streaming<TransferRequest>>,
    ) -> Result<Response<Self::TransferStream>, Status> {
        let mut in_stream = request.into_inner();
        let (tx, rx) = mpsc::channel(10);
        let self_clone = self.clone();

        tokio::spawn(async move {
            let mut temp_file_path: Option<PathBuf> = None;
            let mut file: Option<fs::File> = None;
            let mut receiver_bank_id: Option<String> = None;
            let mut full_metadata: Option<ftp::Metadata> = None;
            let mut message_only_content: Option<Vec<u8>> = None;

            while let Some(result) = in_stream.next().await {
                match result {
                    Ok(req) => {
                        // Validate and store metadata
                        let metadata = match req.metadata.as_ref().ok_or_else(|| {
                            Status::invalid_argument("Missing metadata in request")
                        }) {
                            Ok(metadata_ref) => metadata_ref,
                            Err(e) => {
                                let error_response = TransferResponse {
                                    transfer_id: full_metadata
                                        .as_ref()
                                        .map(|m| m.transfer_id.clone())
                                        .unwrap_or_default(),
                                    status: crate::ftp::Status::Failure as i32,
                                    chunk_index: 0,
                                    error_info: Some(crate::ftp::ErrorInfo {
                                        error_code: "INVALID_METADATA".to_string(),
                                        error_details: format!("Invalid metadata: {}", e),
                                    }),
                                };
                                let _ = tx.send(Ok(error_response)).await;
                                eprintln!("[SERVER] Invalid metadata: {}", e);
                                return; // Exit the spawn block, don't return Err
                            }
                        };
            
                        if full_metadata.is_none() {
                            full_metadata = Some(metadata.clone());
                            receiver_bank_id = Some(metadata.receiver_bank_id.clone());
            
                            // Handle message-only payload
                            if matches!(
                                &metadata.payload_type,
                                Some(ftp::metadata::PayloadType::MessageInfo(_))
                            ) {
                                let ack_response = TransferResponse {
                                    transfer_id: metadata.transfer_id.clone(),
                                    status: crate::ftp::Status::Success as i32,
                                    chunk_index: 1,
                                    error_info: None,
                                };
            
                                if tx.send(Ok(ack_response)).await.is_err() {
                                    eprintln!("[SERVER] Failed to send message ACK");
                                    return; // Exit the spawn block
                                }
                                message_only_content = Some(req.content);
                                break;
                            }
                        }
            
                        // Initialize file if not already done
                        if file.is_none() {
                            let file_info = match &metadata.payload_type {
                                Some(ftp::metadata::PayloadType::FileInfo(info)) => Some(info),
                                Some(ftp::metadata::PayloadType::AttachmentInfo(info)) => {
                                    info.file_info.as_ref()
                                }
                                _ => None,
                            };
            
                            if let Some(fi) = file_info {
                                let storage_dir = "received_files";
                                if let Err(e) = fs::create_dir_all(storage_dir).await {
                                    let error_response = TransferResponse {
                                        transfer_id: metadata.transfer_id.clone(),
                                        status: crate::ftp::Status::Failure as i32,
                                        chunk_index: metadata.chunk_index,
                                        error_info: Some(crate::ftp::ErrorInfo {
                                            error_code: "DIR_CREATION_FAILED".to_string(),
                                            error_details: format!("Failed to create storage dir: {}", e),
                                        }),
                                    };
                                    let _ = tx.send(Ok(error_response)).await;
                                    eprintln!("[SERVER] Failed to create storage dir: {}", e);
                                    return; // Exit the spawn block
                                }
                                
                                let path = Path::new(storage_dir).join(format!("{}", &fi.name));
                                temp_file_path = Some(path.clone());
                                
                                match fs::File::create(&path).await {
                                    Ok(f) => file = Some(f),
                                    Err(e) => {
                                        let error_response = TransferResponse {
                                            transfer_id: metadata.transfer_id.clone(),
                                            status: crate::ftp::Status::Failure as i32,
                                            chunk_index: metadata.chunk_index,
                                            error_info: Some(crate::ftp::ErrorInfo {
                                                error_code: "FILE_CREATION_FAILED".to_string(),
                                                error_details: format!("Failed to create file: {}", e),
                                            }),
                                        };
                                        let _ = tx.send(Ok(error_response)).await;
                                        eprintln!("[SERVER] Failed to create file: {}", e);
                                        return; // Exit the spawn block
                                    }
                                }
                            }
                        }
            
                        // Write chunk to file and send ACK
                        if let Some(f) = file.as_mut() {
                            if !req.content.is_empty() {
                                if let Err(e) = f.write_all(&req.content).await {
                                    let error_response = TransferResponse {
                                        transfer_id: metadata.transfer_id.clone(),
                                        status: crate::ftp::Status::Failure as i32,
                                        chunk_index: metadata.chunk_index,
                                        error_info: Some(crate::ftp::ErrorInfo {
                                            error_code: "WRITE_FAILED".to_string(),
                                            error_details: format!("Failed to write chunk: {}", e),
                                        }),
                                    };
                                    let _ = tx.send(Ok(error_response)).await;
                                    eprintln!("[SERVER] Failed to write chunk {}: {}", metadata.chunk_index, e);
                                    return; // Exit the spawn block
                                }
            
                                if let Err(e) = f.flush().await {
                                    let error_response = TransferResponse {
                                        transfer_id: metadata.transfer_id.clone(),
                                        status: crate::ftp::Status::Failure as i32,
                                        chunk_index: metadata.chunk_index,
                                        error_info: Some(crate::ftp::ErrorInfo {
                                            error_code: "FLUSH_FAILED".to_string(),
                                            error_details: format!("Failed to flush chunk: {}", e),
                                        }),
                                    };
                                    let _ = tx.send(Ok(error_response)).await;
                                    eprintln!("[SERVER] Failed to flush chunk {}: {}", metadata.chunk_index, e);
                                    return; // Exit the spawn block
                                }
            
                                // Send success ACK
                                let ack_response = TransferResponse {
                                    transfer_id: metadata.transfer_id.clone(),
                                    status: crate::ftp::Status::Success as i32,
                                    chunk_index: metadata.chunk_index,
                                    error_info: None,
                                };
                                if tx.send(Ok(ack_response)).await.is_err() {
                                    eprintln!("[SERVER] Failed to send ACK for chunk {}", metadata.chunk_index);
                                    return; // Exit the spawn block
                                }
                                println!("[SERVER] Sent ACK for chunk {}", metadata.chunk_index);
                            }
                        }
                    }
                    Err(e) => {
                        let error_response = TransferResponse {
                            transfer_id: full_metadata
                                .as_ref()
                                .map(|m| m.transfer_id.clone())
                                .unwrap_or_default(),
                            status: crate::ftp::Status::Failure as i32,
                            chunk_index: 0,
                            error_info: Some(crate::ftp::ErrorInfo {
                                error_code: "STREAM_ERROR".to_string(),
                                error_details: format!("Stream error: {}", e),
                            }),
                        };
                        let _ = tx.send(Ok(error_response)).await;
                        eprintln!("[SERVER] Stream error: {}", e);
                        return; // Exit the spawn block
                    }
                }
            }
            
            //FORWARDING LOGIC THIS ONWARDS
            if let (Some(receiver_id), Some(metadata)) = (receiver_bank_id, full_metadata.as_ref()) {
                if let Some(message_content) = message_only_content {
                    match self_clone
                        .forward_message(message_content, metadata.clone(), &receiver_id)
                        .await
                    {
                        Ok(mut forward_stream) => {
                            while let Some(item) = forward_stream.next().await {
                                if tx.send(item).await.is_err() {
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            // Send a proper error response to the client
                            let error_response = TransferResponse {
                                transfer_id: metadata.transfer_id.clone(),
                                status: crate::ftp::Status::Failure as i32,
                                chunk_index: metadata.chunk_index.clone(),
                                error_info: Some(crate::ftp::ErrorInfo {
                                    error_code: "FORWARD_FAILED".to_string(),
                                    error_details: format!("Failed to forward message to {}: {}", receiver_id, e),
                                }),
                            };
                            let _ = tx.send(Ok(error_response)).await;
                            eprintln!("[SERVER] Failed to forward message to {}: {}", receiver_id, e);
                        }
                    }
                }else if let Some(path) = temp_file_path {
                    match self_clone.forward_file(&path, metadata.clone(), &receiver_id).await {
                        Ok(mut forward_stream) => {
                            while let Some(item) = forward_stream.next().await {
                                if tx.send(item).await.is_err() {
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            // Send a proper error response to the client
                            let error_response = TransferResponse {
                                transfer_id: metadata.transfer_id.clone(),
                                status: crate::ftp::Status::Failure as i32,
                                chunk_index: metadata.chunk_index.clone(),
                                error_info: Some(crate::ftp::ErrorInfo {
                                    error_code: "FORWARD_FAILED".to_string(),
                                    error_details: format!("Failed to forward file to {}: {}", receiver_id, e),
                                }),
                            };
                            let _ = tx.send(Ok(error_response)).await;
                            eprintln!("[SERVER] Failed to forward file to {}: {}", receiver_id, e);
                        }
                    }
            
                    // Clean up temp file regardless of success/failure
                    if fs::remove_file(path).await.is_err() {
                        eprintln!("[SERVER] Warning: Failed to clean up temporary file");
                    }
                }else {
                    let error_response = TransferResponse {
                        transfer_id: full_metadata
                            .as_ref()
                            .map(|m| m.transfer_id.clone())  
                            .unwrap_or_default(),
                        status: crate::ftp::Status::Failure as i32,
                        chunk_index: metadata.chunk_index.clone(),
                        error_info: Some(crate::ftp::ErrorInfo {
                            error_code: "NO_CONTENT".to_string(),
                            error_details: "No message or file content was found in the request.".to_string(),
                        }),
                    };
                    let _ = tx.send(Ok(error_response)).await;
                }
            }else {
                let error_response = TransferResponse {
                    transfer_id: full_metadata
                        .as_ref()
                        .map(|m| m.transfer_id.clone())
                        .unwrap_or_default(),
                    status: crate::ftp::Status::Failure as i32,
                    chunk_index: 0,
                    error_info: Some(crate::ftp::ErrorInfo {
                        error_code: "MISSING_INFO".to_string(),
                        error_details: "Receiver information or metadata was missing".to_string(),
                    }),
                };
                let _ = tx.send(Ok(error_response)).await;
            } 
        });

        let out_stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(out_stream) as Self::TransferStream))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    // Get server address from environment variables
    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("SERVER_PORT").unwrap_or_else(|_| "50051".to_string());
    let addr = format!("{}:{}", host, port).parse::<SocketAddr>()?;

    let service = FileTransferService::new();
    println!("Server listening on {}", addr);

    Server::builder()
        .add_service(TransferServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
