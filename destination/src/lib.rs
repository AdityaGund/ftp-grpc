use crate::ftp::TransferResponse;
use dotenv::dotenv;
use ftp::transfer_service_server::{TransferService, TransferServiceServer};
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
use crate::services::db::Database;
use std::sync::Arc;
use crate::models::file_info_model::FileInfo as DbFileInfo;
use mongodb::bson::oid::ObjectId;
use chrono::Utc;

pub mod models;
pub mod services;
pub mod error;

pub mod ftp {
    tonic::include_proto!("ftp");
}

#[derive(Clone)]
pub struct FileTransferService {
    notifier: broadcast::Sender<TransferResponse>,
    db: Arc<Database>,
}

impl FileTransferService {
    pub fn new(notifier: broadcast::Sender<TransferResponse>, db: Arc<Database>) -> Self {
        Self { notifier, db }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub notifier: broadcast::Sender<TransferResponse>,
}

// SSE endpoint
// pub async fn events_stream(state: web::Data<AppState>) -> impl Responder {
//     let rx = state.notifier.subscribe();
//     let stream = BroadcastStream::new(rx).map(|msg| match msg {
//         Ok(resp) => {
//             let json = format!("{{\"transfer_id\":\"{}\",\"status\":{}}}", resp.transfer_id, resp.status);
//             Ok::<Bytes, std::convert::Infallible>(Bytes::from(format!("data: {}\n\n", json)))
//         }
//         Err(_) => Ok(Bytes::from("event: ping\n\n")),
//     });

//     HttpResponse::Ok()
//         .insert_header(("Content-Type", "text/event-stream"))
//         .insert_header(("Cache-Control", "no-cache"))
//         .insert_header(("Connection", "keep-alive"))
//         .streaming(stream)
// }

#[tonic::async_trait]
impl TransferService for FileTransferService {
    type TransferStream = Pin<Box<dyn Stream<Item = Result<TransferResponse, Status>> + Send>>;

    async fn transfer(
        &self,
        request: Request<Streaming<ftp::TransferRequest>>,
    ) -> Result<Response<Self::TransferStream>, Status> {
        println!("[DESTINATION] Notification: Receiving file");

        let notifier = self.notifier.clone();
        let db_clone = self.db.clone();
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
            let mut sender_bank_id = String::new();
            let mut receiver_bank_id = String::new();
            let mut receiver_bank_ip = String::new();
            let mut time_sent_at = String::new();
            let mut file_name = String::new();
            let mut file_path = String::new();

            while let Some(result) = in_stream.next().await {
                match result {
                    Ok(mut req) => {
                        println!("received a chunk ({} bytes)", req.content.len());

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
                                            println!("extracting message from first chunk");
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
                            // Extract common metadata fields for ALL payload types
                            if transfer_id.is_empty() {
                                transfer_id = metadata.transfer_id.clone();
                            }
                            
                            // Extract common fields that should be available for all payload types
                            if sender_bank_id.is_empty() {
                                sender_bank_id = metadata.sender_bank_id.clone();
                            }
                            if receiver_bank_id.is_empty() {
                                receiver_bank_id = metadata.receiver_bank_id.clone();
                            }
                            if time_sent_at.is_empty() {
                                time_sent_at = metadata.timestamp.clone();
                            }

                            // only check for files (file-specific logic)
                            if !matches!(
                                &metadata.payload_type,
                                Some(ftp::metadata::PayloadType::MessageInfo(_))
                            ) {
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

                                    // create directory and file handling code...
                                    if let Some(fi) = file_info {
                                        println!("creating file {}", &fi.name);
                                        let storage_dir = "destination_files";
                                        let _ = fs::create_dir_all(storage_dir).await;
                                        let path = Path::new(storage_dir).join(format!("{}", &fi.name));
                                        temp_file_path = Some(path.clone());

                                        let append_mode = fi.content_type.contains("log");

                                        let file_handle = if append_mode {
                                            
                                            if !path.exists(){
                                                println!("[DEST] Creating {}", fi.name);
                                                tokio::fs::File::create(&path).await.unwrap()
                                            } else {
                                                println!("[DEST] Appending to {}", fi.name);
                                                tokio::fs::OpenOptions::new()
                                                    .append(true)
                                                    .open(&path)
                                                    .await
                                                    .unwrap()
                                            }
                                        } else {
                                            println!("[DEST] Creating {}", fi.name);
                                            tokio::fs::File::create(&path).await.unwrap()
                                        };
                                        file = Some(file_handle);

                                        // store file-specific metadata details
                                        file_name = fi.name.clone();
                                        file_path = match std::fs::canonicalize(&path) {
                                            Ok(abs) => abs.to_string_lossy().to_string(),
                                            Err(_) => path.clone().to_string_lossy().to_string(),
                                        };
                                        sender_bank_id = metadata.sender_bank_id.clone();
                                        receiver_bank_id = metadata.receiver_bank_id.clone();
                                        receiver_bank_ip = metadata.receiver_bank_ip.clone();
                                        time_sent_at = metadata.timestamp.clone();
                                    }
                                }

                                // write file (after first chunk)
                                if let Some(f) = file.as_mut() {
                                    println!("writing chunk to file ({} bytes)", req.content.len());
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

                        let now = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f %Z").to_string();
                        let response = TransferResponse {
                            transfer_id: req
                                .metadata
                                .as_ref()
                                .map_or_else(String::new, |m| m.transfer_id.clone()),
                            status: ftp::Status::InProgress as i32,
                            time_received_at: now,
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
                println!("flushing file to disk");
                let _ = f.flush().await;
            }


            let now = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f %Z").to_string();
            let response = TransferResponse {
                transfer_id,
                status: ftp::Status::Success as i32,
                time_received_at: now.clone(),
                error_info: Some(ErrorInfo {
                    error_code: "DESTINATION".to_string(),
                    error_details: "DONE".to_string(),
                }),
            };

            let db_doc = DbFileInfo {
                _id: ObjectId::new(),
                name: file_name,
                path: file_path,
                sender_bank_id,
                receiver_bank_id,
                receiver_bank_ip,
                message,
                time_sent_at,
                time_received_at: now.clone(),
            };

            let _ = db_clone.store_file_info(db_doc).await;

            println!("[DESTINATION] Final ACK sent for transfer {} (SUCCESS)", response.transfer_id);
            let _ = tx.send(Ok(response.clone())).await;
            let _ = notifier.send(response.clone());
            if let Some(_path) = &temp_file_path {
                println!("file saved to disk and metadata stored");
            }
        });

        let out_stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(out_stream) as Self::TransferStream))
    }
}

pub async fn run_destination() -> Result<(), Box<dyn Error>> {
    let dest_env_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(".env");
    if dotenv::from_path(dest_env_path.as_path()).is_err() {
        dotenv().ok();
    }


    // gRPC address
    let host = env::var("DESTINATION_HOST").unwrap();
    let port = env::var("DESTINATION_PORT").unwrap();
    let grpc_addr: SocketAddr = format!("{}:{}", host, port).parse()?;

    let db = Arc::new(Database::init().await);
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
        .max_frame_size(Some(8 * 1024 * 1024))
        .add_service(
            TransferServiceServer::new(FileTransferService::new(tx, db.clone()))
                .max_encoding_message_size(8 * 1024 * 1024)
                .max_decoding_message_size(8 * 1024 * 1024),
        )
        .serve(grpc_addr)
        .await?;


    // Wait for HTTP server (if it finishes)
    // let _ = http_handle.await;

    Ok(())
}
