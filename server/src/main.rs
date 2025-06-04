use std::{error::Error, net::SocketAddr};
use tonic::transport::Server;
use ftp::transfer_service_server::{TransferService, TransferServiceServer};
use ftp::{TransferResponse, Chunk, Status};

pub mod ftp {
    tonic::include_proto!("ftp");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("ftp_descriptor");
}

#[derive(Debug, Default)]
pub struct TransferServiceImpl {}

#[tonic::async_trait]
impl TransferService for TransferServiceImpl {
    async fn transfer(
        &self,
        request: tonic::Request<tonic::Streaming<Chunk>>,
    ) -> Result<tonic::Response<TransferResponse>, tonic::Status> {
        let response = TransferResponse {
            transfer_id: "test".to_string(),
            status: Status::Success as i32,
            message: "Success".to_string(),
        };
        Ok(tonic::Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // println!("Hello, world!");

    let addr: SocketAddr = "[::1]:50051".parse()?;
    let service = TransferServiceImpl::default();

    Server::builder()
        .add_service(TransferServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
