use ftp::transfer_service_server::{TransferService, TransferServiceServer};
use std::{env, error::Error, net::SocketAddr, path::{Path, PathBuf}, pin::Pin};
use tokio::{fs, io::AsyncWriteExt};
use tonic::{transport::Server, Request, Response, Status, Streaming};
use dotenv::dotenv;
use tokio_stream::{Stream, StreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use crate::ftp::{TransferResponse};
use uuid::Uuid;

pub mod ftp {
    tonic::include_proto!("ftp");
}

#[derive(Debug, Default)]
pub struct FileTransferService {}

#[tonic::async_trait]
impl TransferService for FileTransferService {
    type TransferStream = Pin<Box<dyn Stream<Item = Result<TransferResponse, Status>> + Send>>;

    async fn transfer(
        &self,
        request: Request<Streaming<ftp::TransferRequest>>,
    ) -> Result<Response<Self::TransferStream>, Status> {
        let mut in_stream = request.into_inner();
        let (tx, rx) = mpsc::channel(4);

        tokio::spawn(async move {
            let mut temp_file_path: Option<PathBuf> = None;
            let mut file: Option<fs::File> = None;
            let mut transfer_id = String::new();

            while let Some(result) = in_stream.next().await {
                 match result {
                    Ok(req) => {
                        if let Some(metadata) = &req.metadata {
                            if transfer_id.is_empty() {
                                transfer_id = metadata.transfer_id.clone();
                            }

                            if file.is_none() {
                                let file_info = match &metadata.payload_type {
                                    Some(ftp::metadata::PayloadType::FileInfo(info)) => Some(info),
                                    Some(ftp::metadata::PayloadType::AttachmentInfo(info)) => {
                                        info.file_info.as_ref()
                                    }
                                    _ => None,
                                };

                                if let Some(fi) = file_info {
                                    let storage_dir = "destination_files";
                                    let _ = fs::create_dir_all(storage_dir).await;
                                    let path = Path::new(storage_dir).join(format!("{}-{}", Uuid::new_v4(), &fi.name));
                                    temp_file_path = Some(path.clone());
                                    file = Some(fs::File::create(path).await.unwrap());
                                }
                            }

                            if let Some(f) = file.as_mut() {
                                if f.write_all(&req.content).await.is_err() {
                                    let _ = tx.send(Err(Status::internal("Failed to write file chunk"))).await;
                                    return;
                                }
                            }

                            let response = TransferResponse {
                                transfer_id: metadata.transfer_id.clone(),
                                status: ftp::Status::InProgress as i32,
                                error_info: None
                            };
                            if tx.send(Ok(response)).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        if tx.send(Err(e)).await.is_err() {
                            break;
                        }
                    }
                }
            }
            if let Some(f) = file.as_mut() {
                let _ = f.flush().await;
            }

            let response = TransferResponse {
                transfer_id,
                status: ftp::Status::Success as i32,
                error_info: None,
            };
            let _ = tx.send(Ok(response)).await;
            if let Some(path) = &temp_file_path {
                 println!("[DESTINATION] File saved to: {}", path.display());
            }
        });

        let out_stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(out_stream) as Self::TransferStream))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let host = env::var("SERVER_HOST").unwrap().to_string();
    let port = "50052".to_string();
    let addr = format!("{}:{}", host, port).parse::<SocketAddr>()?;
    
    let service = FileTransferService::default();
    println!("[DESTINATION] Server listening on {}", addr);

    Server::builder()
        .add_service(TransferServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
} 