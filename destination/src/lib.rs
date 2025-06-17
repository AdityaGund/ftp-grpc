use crate::ftp::TransferResponse;
use dotenv::dotenv;
use ftp::transfer_service_server::{TransferService, TransferServiceServer};
// use uuid::serde;
use std::{
    env,
    error::Error,
    net::SocketAddr,
    path::{Path, PathBuf},
    pin::Pin,
};
use tokio::sync::mpsc;
use tokio::{fs, io::AsyncWriteExt};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Status, Streaming, transport::Server};
use ftp::ErrorInfo;
use tokio::sync::broadcast;
use bytes::Bytes;
use tokio_stream::wrappers::BroadcastStream;
use actix_web::{HttpResponse, web, Responder};
// use actix_cors::Cors;
// use serde_json;
// use uuid::Uuid;

pub mod ftp {
    tonic::include_proto!("ftp");
}

#[derive(Clone)]
pub struct FileTransferService {
    notifier: broadcast::Sender<TransferResponse>,
}

impl FileTransferService {
    pub fn new(notifier: broadcast::Sender<TransferResponse>) -> Self {
        Self { notifier }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub notifier: broadcast::Sender<TransferResponse>,
}

// SSE endpoint
pub async fn events_stream(state: web::Data<AppState>) -> impl Responder {
    let rx = state.notifier.subscribe();
    let stream = BroadcastStream::new(rx).map(|msg| match msg {
        Ok(resp) => {
            let json = format!("{{\"transfer_id\":\"{}\",\"status\":{}}}", resp.transfer_id, resp.status);
            Ok::<Bytes, std::convert::Infallible>(Bytes::from(format!("data: {}\n\n", json)))
        }
        Err(_) => Ok(Bytes::from("event: ping\n\n")),
    });

    HttpResponse::Ok()
        .insert_header(("Content-Type", "text/event-stream"))
        .insert_header(("Cache-Control", "no-cache"))
        .insert_header(("Connection", "keep-alive"))
        .streaming(stream)
}

#[tonic::async_trait]
impl TransferService for FileTransferService {
    type TransferStream = Pin<Box<dyn Stream<Item = Result<TransferResponse, Status>> + Send>>;

    async fn transfer(
        &self,
        request: Request<Streaming<ftp::TransferRequest>>,
    ) -> Result<Response<Self::TransferStream>, Status> {
        println!("[DESTINATION] Notification: Receiving file");

        let notifier = self.notifier.clone();
        let mut in_stream = request.into_inner();
        let (tx, rx) = mpsc::channel(4);

        // let self_clone = self.clone();

        tokio::spawn(async move {
            let mut temp_file_path: Option<PathBuf> = None;
            let mut file: Option<fs::File> = None;
            let mut transfer_id = String::new();
            let mut is_first_chunk = true;
            let mut message = String::new();
            const SEPARATOR: &[u8] = b"---MESSAGE_END---";

            while let Some(result) = in_stream.next().await {
                match result {
                    Ok(mut req) => {

                        // first chunk to check for message
                        if is_first_chunk {
                            is_first_chunk = false;
                            if let Some(metadata) = &req.metadata {
                                match &metadata.payload_type {
                                    // if metadata is msg+file
                                    Some(ftp::metadata::PayloadType::AttachmentInfo(_)) => {
                                        if let Some(pos) = req
                                            .content
                                            .windows(SEPARATOR.len())
                                            .position(|window| window == SEPARATOR)
                                        {
                                            message = String::from_utf8_lossy(&req.content[..pos])
                                                .to_string();
                                            println!("[DESTINATION] Received message: {}", &message);
                                            req.content =
                                                req.content[pos + SEPARATOR.len()..].to_vec();
                                        }
                                    }

                                    // if metadata is only msg
                                    Some(ftp::metadata::PayloadType::MessageInfo(_)) => {
                                        message = String::from_utf8_lossy(&req.content).to_string();
                                        println!(
                                            "[DESTINATION] Received message: {}",
                                            &message
                                        );
                                        req.content.clear();
                                    }

                                    // only file, no msg
                                    _ => {}
                                }
                            }
                        }

                        // after first chunk
                        if let Some(metadata) = &req.metadata {

                            // only check for files
                            if !matches!(
                                &metadata.payload_type,
                                Some(ftp::metadata::PayloadType::MessageInfo(_))
                            ) {
                                if transfer_id.is_empty() {
                                    transfer_id = metadata.transfer_id.clone();
                                }

                                // write file on disk (for first chunk)
                                if file.is_none() {

                                    // file name
                                    let file_info = match &metadata.payload_type {
                                        Some(ftp::metadata::PayloadType::FileInfo(info)) => {
                                            Some(info)
                                        }
                                        Some(ftp::metadata::PayloadType::AttachmentInfo(info)) => {
                                            info.file_info.as_ref()
                                        }
                                        _ => None,
                                    };

                                    // create directory
                                    if let Some(fi) = file_info {
                                        let storage_dir = "destination_files";
                                        let _ = fs::create_dir_all(storage_dir).await;
                                        let path =
                                            Path::new(storage_dir).join(format!("{}", &fi.name));
                                        temp_file_path = Some(path.clone());
                                        file = Some(fs::File::create(path).await.unwrap());
                                    }
                                }

                                // write file (after first chunk)
                                if let Some(f) = file.as_mut() {
                                    if !req.content.is_empty() {
                                        if f.write_all(&req.content).await.is_err() {
                                            let _ = tx
                                                .send(Err(Status::internal(
                                                    "Failed to write file chunk",
                                                )))
                                                .await;
                                            return;
                                        }
                                    }
                                }
                            }
                        }

                        let response = TransferResponse {
                            transfer_id: req
                                .metadata
                                .as_ref()
                                .map_or_else(String::new, |m| m.transfer_id.clone()),
                            status: ftp::Status::InProgress as i32,
                            error_info: Some(ErrorInfo {
                                error_code: "DESTINATION".to_string(),
                                error_details: req.metadata.as_ref().map(|m| format!("{}/{}", m.chunk_index, m.total_chunks)).unwrap_or_default(),
                            }),
                        };

                        println!("[DESTINATION] ACK sent for transfer {} (IN_PROGRESS)", response.transfer_id);

                        if tx.send(Ok(response.clone())).await.is_err() {
                            break;
                        }

                        let _ = notifier.send(response.clone());
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
                error_info: Some(ErrorInfo {
                    error_code: "DESTINATION".to_string(),
                    error_details: "DONE".to_string(),
                }),
            };
            println!("[DESTINATION] Final ACK sent for transfer {} (SUCCESS)", response.transfer_id);
            let _ = tx.send(Ok(response.clone())).await;
            let _ = notifier.send(response.clone());
            if let Some(path) = &temp_file_path {
                println!("[DESTINATION] File saved to: {}", path.display());
            }
        });

        let out_stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(out_stream) as Self::TransferStream))
    }
}

pub async fn run_destination() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    // gRPC address
    let host = env::var("DESTINATION_HOST").unwrap();
    let port = env::var("DESTINATION_PORT").unwrap();
    let grpc_addr: SocketAddr = format!("{}:{}", host, port).parse()?;

    // HTTP address for SSE
    // let http_addr = env::var("DESTINATION_HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:5174".to_string());

    // broadcast channel shared between gRPC & HTTP
    let (tx, _rx) = broadcast::channel::<TransferResponse>(100);
    // let app_state = AppState { notifier: tx.clone() };

    // ---- spawn Actix-web HTTP server (SSE) ------------------------------
    // let http_handle = actix_web::rt::spawn({
    //     let app_state = app_state.clone();
    //     async move {
    //         println!("[DESTINATION HTTP] Listening on {}", http_addr);
    //         HttpServer::new(move || {
    //             let cors = Cors::default().allow_any_origin();
    //             App::new()
    //                 .app_data(web::Data::new(app_state.clone()))
    //                 .wrap(cors)
    //                 .route("/events", web::get().to(events_stream))
    //         })
    //         .bind(&http_addr)?
    //         .run()
    //         .await
    //     }
    // });

    // ---- start gRPC server ---------------------------------------------
    println!("[DESTINATION GRPC] Listening on {}", grpc_addr);
    Server::builder()
        .add_service(TransferServiceServer::new(FileTransferService::new(tx)))
        .serve(grpc_addr)
        .await?;

    // Wait for HTTP server (if it finishes)
    // let _ = http_handle.await;

    Ok(())
}
