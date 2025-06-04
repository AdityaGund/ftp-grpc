use std::error::Error;
use ftp::{TransferRequest, Metadata, FileInfo};

use ftp::transfer_service_client::TransferServiceClient;

pub mod ftp {
    tonic::include_proto!("ftp");
}

async fn transfer_file(client: &mut TransferServiceClient<tonic::transport::Channel>, file_path: &str) -> Result<(), Box<dyn Error>> {
    // For grpcurl, you want to be able to send path, sender_bank_id, receiver_bank_id
    // We'll just use the file_path and hardcode the rest for now
    let file_info = FileInfo {
        path: file_path.to_string(),
        name: file_path.to_string(),
        // size: 0, // Not used in proto for grpcurl, so set to 0
    };

    let metadata = Metadata {
        transfer_id: "transfer-1".to_string(),
        sender_bank_id: "A".to_string(), // You can change this for grpcurl
        receiver_bank_id: "B".to_string(), // You can change this for grpcurl
        payload_type: Some(ftp::metadata::PayloadType::FileInfo(file_info)),
    };

    let request = TransferRequest {
        metadata: Some(metadata),
    };

    let response = client.transfer(request).await?;
    println!("Response: {:?}", response.into_inner());
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = "http://127.0.0.1:5051";
    let mut client = TransferServiceClient::connect(url).await?;
    transfer_file(&mut client, "../../ref.txt").await?;
    Ok(())
}
