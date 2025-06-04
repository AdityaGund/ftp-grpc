use ftp::transfer_service_server::{TransferService, TransferServiceServer};
use std::{env, error::Error, net::SocketAddr, path::Path};
use tokio::fs;
use tonic::transport::Server;
use dotenv::dotenv;

pub mod ftp {
    tonic::include_proto!("ftp");
}

#[derive(Debug, Default)]
pub struct FileTransferService {}

#[tonic::async_trait]
impl TransferService for FileTransferService {
    async fn transfer(
        &self,
        request: tonic::Request<ftp::TransferRequest>,
    ) -> Result<tonic::Response<ftp::TransferResponse>, tonic::Status> {
        let req = request.into_inner();
        
        // Get file info from metadata
        let file_info = req.metadata
            .as_ref()
            .and_then(|m| match &m.payload_type {
                Some(ftp::metadata::PayloadType::FileInfo(info)) => Some(info),
                _ => None,
            })
            .ok_or_else(|| tonic::Status::invalid_argument("No file info provided"))?;

        let storage_dir = "received_files";
        fs::create_dir_all(storage_dir)
            .await
            .map_err(|e| tonic::Status::internal(format!("Failed to create storage directory: {}", e)))?;

        // Create full path for the file
        let file_path = Path::new(storage_dir).join(&file_info.name);
        
        // Save the file
        fs::write(&file_path, req.content)
            .await
            .map_err(|e| tonic::Status::internal(format!("Failed to write file: {}", e)))?;

        println!("File saved to: {}", file_path.display());

        let response = ftp::TransferResponse {
            transfer_id: req.metadata.clone().map(|m| m.transfer_id).unwrap_or_default(),
            status: ftp::Status::Success as i32,
            message: format!("File {} received & saved successfully", file_info.name),
        };

        Ok(tonic::Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    
    dotenv().ok();
    

    // Get server address from environment variables
    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("SERVER_PORT").unwrap_or_else(|_| "50051".to_string());
    let addr = format!("{}:{}", host, port).parse::<SocketAddr>()?;
    
    let service = FileTransferService::default();
    println!("Server listening on {}", addr);

    Server::builder()
        .add_service(TransferServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}