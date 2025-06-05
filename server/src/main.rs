use ftp::transfer_service_server::{TransferService, TransferServiceServer};
use std::{collections::HashMap, env, error::Error, net::SocketAddr, path::{Path, PathBuf}};
use tokio::fs;
use tonic::{transport::Server, Request};
use dotenv::dotenv;
use crate::ftp::transfer_service_client::TransferServiceClient;

pub mod ftp {
    tonic::include_proto!("ftp");
}

#[derive(Debug)]
pub struct FileTransferService {
    bank_mappings: HashMap<String, String>, // Maps bank_id to server URL
}

impl FileTransferService {
    pub fn new() -> Self {
        let mut mappings = HashMap::new();
        mappings.insert("BANK_C".to_string(), "http://127.0.0.1:50052".to_string());
        mappings.insert("BANK_D".to_string(), "http://127.0.0.1:50053".to_string());
        
        Self { bank_mappings: mappings }
    }

    // Function to save the received file temporarily
    async fn save_file_temporarily(
        &self,
        content: &[u8],
        file_info: &ftp::FileInfo,
    ) -> Result<std::path::PathBuf, tonic::Status> {
        let storage_dir = "received_files";
        fs::create_dir_all(storage_dir)
            .await
            .map_err(|e| tonic::Status::internal(format!("Failed to create storage directory: {}", e)))?;

        let file_path = Path::new(storage_dir).join(&file_info.name);
        fs::write(&file_path, content)
            .await
            .map_err(|e| tonic::Status::internal(format!("Failed to write file: {}", e)))?;

        println!("[SERVER] File received and saved temporarily at: {}", file_path.display());
        Ok(file_path)
    }

    // Function to clean up temporary file
    async fn cleanup_temp_file(&self, file_path: &Path) {
        if let Err(e) = fs::remove_file(file_path).await {
            println!("Warning: Failed to clean up temporary file: {}", e);
        }
    }

    // Function to forward the file to the destination
    async fn forward_file(
        &self,
        file_path: &Path,
        metadata: ftp::Metadata,
        receiver_bank_id: &str,
    ) -> Result<ftp::TransferResponse, tonic::Status> {
        // Get destination URL from bank mappings
        let destination_url = self.bank_mappings
            .get(receiver_bank_id)
            .ok_or_else(|| tonic::Status::not_found(
                format!("No server mapping found for bank: {}", receiver_bank_id)
            ))?;

        println!("[SERVER] Forwarding to bank {} at {}", receiver_bank_id, destination_url);

        let content = fs::read(file_path)
            .await
            .map_err(|e| tonic::Status::internal(format!("Failed to read file: {}", e)))?;

        let mut client = TransferServiceClient::connect(destination_url.clone())
            .await
            .map_err(|e| tonic::Status::internal(format!("Failed to connect to destination: {}", e)))?;

        // Create and send request
        let request = ftp::TransferRequest {
            metadata: Some(metadata),
            content,
        };

        let response = client.transfer(request).await?;
        Ok(response.into_inner())
    }
}

#[tonic::async_trait]
impl TransferService for FileTransferService {
    async fn transfer(
        &self,
        request: tonic::Request<ftp::TransferRequest>,
    ) -> Result<tonic::Response<ftp::TransferResponse>, tonic::Status> {
        let req = request.into_inner();
        
        let metadata = req.metadata
            .as_ref()
            .ok_or_else(|| tonic::Status::invalid_argument("No metadata provided"))?;

        let receiver_bank_id = &metadata.receiver_bank_id;
        
        let file_info = match &metadata.payload_type {
            Some(ftp::metadata::PayloadType::FileInfo(info)) => info,
            _ => return Err(tonic::Status::invalid_argument("No file info provided")),
        };

        let temp_file_path = self.save_file_temporarily(&req.content, file_info).await?;

        let metadata = req.metadata.clone().unwrap();
        match self.forward_file(&temp_file_path, metadata, receiver_bank_id).await {
            Ok(response) => {
                self.cleanup_temp_file(&temp_file_path).await;
                Ok(tonic::Response::new(response))
            }
            Err(e) => {
                Err(tonic::Status::internal(format!("Failed to forward file: {}", e)))
            }
        }
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
