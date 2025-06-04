use ftp::transfer_service_server::{TransferService, TransferServiceServer};
use ftp::{Status, TransferRequest, TransferResponse};
use std::{error::Error, net::SocketAddr};
use tonic::transport::Server;

pub mod ftp {
    tonic::include_proto!("ftp");

    pub(crate) const _FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("ftp_descriptor");
}

#[derive(Debug, Default)]
pub struct ClientTransfer {}

#[tonic::async_trait]
impl TransferService for ClientTransfer {
    async fn transfer(
        &self,
        request: tonic::Request<TransferRequest>,
    ) -> Result<tonic::Response<TransferResponse>, tonic::Status> {
        println!("[SERVER] Transfer function invoked!");
        let req = request.into_inner();
        let transfer_id = req
            .metadata
            .as_ref()
            .map(|m| m.transfer_id.clone())
            .unwrap_or_default();

        let file_name = req
            .metadata
            .as_ref()
            .and_then(|m| match &m.payload_type {
                Some(ftp::metadata::PayloadType::FileInfo(info)) => Some(info.name.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "no_file_name_in_metadata".to_string());

        let file_path = req
            .metadata
            .as_ref()
            .and_then(|m| match &m.payload_type {
                Some(ftp::metadata::PayloadType::FileInfo(info)) => Some(info.path.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "no_path_in_metadata".to_string());

        println!(
            "[SERVER] Received transfer_id: {transfer_id}, file_name: {file_name}, file_path: {file_path}"
        );
        
        let response = TransferResponse {
            transfer_id,
            status: Status::Success as i32,
            message: format!("Received file info: name={}, path={}", file_name, file_path),
        };
        Ok(tonic::Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("[SERVER] Starting gRPC server on 127.0.0.1:5051");
    let addr: SocketAddr = "127.0.0.1:5051".parse()?;
    let service = ClientTransfer::default();

    Server::builder()
        .add_service(TransferServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
