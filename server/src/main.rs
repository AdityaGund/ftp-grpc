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
use tonic::{Request, Response, Status, Streaming, transport::Server};
use uuid::Uuid;

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
        mappings.insert("BANK_C".to_string(), "192.168.164.16:50052".to_string());
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
                        if full_metadata.is_none() {
                            if let Some(metadata) = &req.metadata {
                                full_metadata = Some(metadata.clone());
                                receiver_bank_id = Some(metadata.receiver_bank_id.clone());

                                if matches!(
                                    &metadata.payload_type,
                                    Some(ftp::metadata::PayloadType::MessageInfo(_))
                                ) {
                                    message_only_content = Some(req.content);
                                    break;
                                }
                            }
                        }

                        if file.is_none() {
                            if let Some(metadata) = &req.metadata {
                                let file_info = match &metadata.payload_type {
                                    Some(ftp::metadata::PayloadType::FileInfo(info)) => Some(info),
                                    Some(ftp::metadata::PayloadType::AttachmentInfo(info)) => {
                                        info.file_info.as_ref()
                                    }
                                    _ => None,
                                };

                                if let Some(fi) = file_info {
                                    let storage_dir = "received_files";
                                    if fs::create_dir_all(storage_dir).await.is_err() {
                                        let _ = tx
                                            .send(Err(Status::internal(
                                                "Could not create storage dir",
                                            )))
                                            .await;
                                        return;
                                    }
                                    let path = Path::new(storage_dir).join(format!("{}", &fi.name));
                                    temp_file_path = Some(path.clone());
                                    file = Some(fs::File::create(path).await.unwrap());
                                }
                            }
                        }

                        if let Some(f) = file.as_mut() {
                            if !req.content.is_empty() {
                                if f.write_all(&req.content).await.is_err() {
                                    let _ = tx
                                        .send(Err(Status::internal(
                                            "Failed to write chunk to temp file",
                                        )))
                                        .await;
                                    return;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e)).await;
                        return;
                    }
                }
            }

            if let Some(f) = file.as_mut() {
                if f.flush().await.is_err() {
                    let _ = tx
                        .send(Err(Status::internal("Failed to flush temp file")))
                        .await;
                    return;
                }
            }

            if let (Some(receiver_id), Some(metadata)) = (receiver_bank_id, full_metadata) {
                if let Some(message_content) = message_only_content {
                    match self_clone
                        .forward_message(message_content, metadata, &receiver_id)
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
                            let _ = tx.send(Err(e)).await;
                        }
                    }
                } else if let Some(path) = temp_file_path {
                    match self_clone.forward_file(&path, metadata, &receiver_id).await {
                        Ok(mut forward_stream) => {
                            while let Some(item) = forward_stream.next().await {
                                if tx.send(item).await.is_err() {
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(Err(e)).await;
                        }
                    }

                    if fs::remove_file(path).await.is_err() {
                        eprintln!("[SERVER] Warning: Failed to clean up temporary file");
                    }
                } else {
                    let _ = tx
                        .send(Err(Status::invalid_argument(
                            "No message or file content was found in the request.",
                        )))
                        .await;
                }
            } else {
                let _ = tx
                    .send(Err(Status::invalid_argument(
                        "Receiver information or metadata was missing",
                    )))
                    .await;
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
