use std::{env, error::Error};
use dotenv::dotenv;
use ftp::{TransferRequest, Metadata, FileInfo};
use tokio::fs;
use ftp::transfer_service_client::TransferServiceClient;

pub mod ftp {
    tonic::include_proto!("ftp");
}

async fn send_file(client: &mut TransferServiceClient<tonic::transport::Channel>, file_path: &str) -> Result<(), Box<dyn Error>> {
    // Read file content
    let content = fs::read(file_path).await?;
    
    // Get file name from path
    let file_name = std::path::Path::new(file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(file_path)
        .to_string();

    println!("[DEBUG] file_name: {}", file_name);
    
    // Create file info
    let file_info = FileInfo {
        name: file_name.clone(),
        path: file_path.to_string(),
        size: content.len() as u64,
    };

    println!("[DEBUG] file_name:{file_info:?}");
    
    // metadata
    let metadata = Metadata {
        transfer_id: uuid::Uuid::new_v4().to_string(),
        sender_bank_id: "CLIENT".to_string(),
        receiver_bank_id: "SERVER".to_string(),
        payload_type: Some(ftp::metadata::PayloadType::FileInfo(file_info)),
    };

    println!("[DEBUG] file_info:{metadata:?}");
    
    // Create and send request
    let request = TransferRequest {
        metadata: Some(metadata),
        content,
    };

    // println!("[DEBUG] file_name:{request:?}");

    // invoke transfer function
    let response = client.transfer(request).await?;
    println!("[SERVER] Server response: {:?}", response.into_inner());
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    dotenv().ok();
    
    // Get server address from environment variables
    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("SERVER_PORT").unwrap_or_else(|_| "50051".to_string());
    let url = format!("http://{}:{}", host, port);
    
    let mut client = TransferServiceClient::connect(url).await?;
    
    // Send a test file
    send_file(&mut client, r"send_files\JAVA Notes.pdf").await?;
    
    Ok(())
}