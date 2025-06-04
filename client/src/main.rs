use std::error::Error;

use ftp::transfer_service_client::TransferServiceClient;

pub mod ftp {
    tonic::include_proto!("ftp");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    let url = "http://[::1]:50051";
    let mut client = TransferServiceClient::connect(url).await?;
    

    Ok(())
}
