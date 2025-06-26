use crate::ftp::{
    TransferRequest, TransferResponse, transfer_service_client::TransferServiceClient,
};
use crate::services::db::Database;
use actix_web::web::Data;
use actix_web::{http, App, HttpServer};
use actix_cors::Cors;
use actix_web_httpauth::middleware::HttpAuthentication;
use chrono::Utc;
use dotenv::dotenv;
use ftp::transfer_service_server::{TransferService, TransferServiceServer};
use ftp::ErrorInfo;
// use std::fs::File;
// use std::sync::mpsc::Sender;
use std::{
    env,
    error::Error,
    net::SocketAddr,
    path::{Path, PathBuf},
    pin::Pin,
};
use tokio::sync::mpsc;
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Status, Streaming, transport::Server};
use crate::models::file_info_model::FileInfo as DbFileInfo;
use mongodb::bson::oid::ObjectId;
use std::sync::Arc;

// const MAX_RETRIES: u8 = 3;
const CHUNK_SIZE: usize = (1024 * 1024) * 5;

pub mod routes;
pub mod handlers;
pub mod error;
pub mod models;
pub mod services;
pub mod middleware;
pub mod grpc_client;

pub mod ftp {
    tonic::include_proto!("ftp");
}

#[derive(Debug, Clone)]
pub struct FileTransferService{
    db: Arc<Database>,
}

impl FileTransferService {
    pub fn new( db: Arc<Database>) -> Self {
        Self { db }
    }

    async fn forward_message(
        &self,
        message_content: Vec<u8>,
        original_metadata: ftp::Metadata,
        _receiver_bank_id: &str,
        receiver_bank_ip: &str,
    ) -> Result<impl Stream<Item = Result<TransferResponse, Status>>, Status> {
        // The client now sends the destination IP/URL directly in the header (metadata)
        // so we no longer rely on a server-side mapping table.
        // Accept either a full URL (starting with http) or a bare IP/host.
        let destination_url = if receiver_bank_ip.starts_with("http") {
            receiver_bank_ip.to_string()
        } else {
            format!("http://{}:50053", receiver_bank_ip)
        };

        println!(
            "[SERVER] Forwarding message to destination {}",
            destination_url
        );

        let mut client = TransferServiceClient::connect(destination_url.clone())
            .await
            .map_err(|e| {
                tonic::Status::internal(format!("Failed to connect to destination: {}", e))
            })?;

        let request = TransferRequest {
            metadata: Some(original_metadata),
            content: message_content,
        };

        let request_stream = tokio_stream::iter(vec![request]);
        let response = client.transfer(request_stream).await?;
        Ok(response.into_inner())
    }

    async fn forward_file(
        &self,
        file_path: &Path,
        original_metadata: ftp::Metadata,
        _receiver_bank_id: &str,
        // Sender used to propagate ACKs back to the original client connection. -> tx
        ack_sender: mpsc::Sender<Result<TransferResponse, Status>>,
        receiver_bank_ip: &str,
        msg: Option<Vec<u8>>,
    ) -> Result<(), Status> {
        
        let db_clone = self.db.clone();

        let message: String = msg
            .as_ref()
            .map(|bytes| String::from_utf8_lossy(bytes).to_string())
            .unwrap_or_default();
            
        let destination_url = if receiver_bank_ip.starts_with("http") {
            receiver_bank_ip.to_string()
        } else {
            format!("http://{}:50053", receiver_bank_ip)
        };

        println!(
            "[SERVER] Forwarding to destination {}",
            destination_url
        );

        let mut destination = TransferServiceClient::connect(destination_url.clone())
            .await
            .map_err(|e: tonic::transport::Error| {
                tonic::Status::internal(format!("Failed to connect to destination: {}", e))
            })?;

        // const CHUNK_SIZE: usize = (1024 * 1024) * 4; // 1MB
        const MAX_RETRIES: u8 = 3;

        let mut file = fs::File::open(file_path).await.map_err(|e| {
            Status::internal(format!("Failed to open temp file for forwarding: {}", e))
        })?;

        let file_size = file
            .metadata()
            .await
            .map_err(|e| Status::internal(format!("Failed to read temp file metadata: {}", e)))?
            .len();

        let total_chunks = (file_size as f64 / CHUNK_SIZE as f64).ceil() as i32;

        // req_tx: push chunks, req_rx: outbound request stream
        let (req_tx, req_rx) = mpsc::channel::<TransferRequest>(1);
        let mut response_stream= destination
            // When tonic polls this stream it blocks until our code drops the next chunk into req_tx.
            .transfer(ReceiverStream::new(req_rx))
            .await?
            .into_inner();


        // Loop through file chunks sequentially, waiting for ACK from destination before sending next chunk
        for i in 0..total_chunks {
            let mut buffer = vec![0; CHUNK_SIZE];
            let n = file.read(&mut buffer).await.map_err(|e| {
                Status::internal(format!("Failed to read chunk from temp file: {}", e))
            })?;
            buffer.truncate(n);

            let mut metadata = original_metadata.clone();
            metadata.chunk_index = i + 1;
            metadata.total_chunks = total_chunks;
            metadata.timestamp = Utc::now().to_rfc3339();

            let req = TransferRequest {
                metadata: Some(metadata),
                content: buffer,
            };

            println!("[SERVER] Forwarding chunk {}/{}", i + 1, total_chunks);
            for attempt in 1..=MAX_RETRIES {
                if attempt > 1 {
                    println!("[SERVER] Retry attempt {} for chunk {}", attempt, i + 1);
                }

                // Send the chunk
                req_tx
                    .send(req.clone())
                    .await
                    .map_err(|e| Status::internal(format!("Failed to send chunk to destination: {}", e)))?;

                // Wait for ACK before proceeding
                match response_stream.next().await {
                    Some(Ok(ack)) => {
                        println!("[SERVER] ACK received from destination for transfer {} (status = {})", ack.transfer_id, ack.status);
                        let _ = ack_sender.send(Ok(ack)).await;
                        println!("[SERVER] ACK forwarded to client");
                        break; // success, move to next chunk
                    }
                    Some(Err(e)) => {
                        println!("[SERVER] Error while waiting for ACK: {e}");
                        if attempt == MAX_RETRIES {
                            let _ = ack_sender.send(Err(e.clone())).await;
                            return Err(e);
                        } else {
                            continue; // retry
                        }
                    }
                    None => {
                        if attempt == MAX_RETRIES {
                            return Err(Status::internal("Destination closed stream unexpectedly"));
                        } else {
                            println!("[SERVER] Stream closed unexpectedly, retrying chunk {}", i + 1);
                            continue;
                        }
                    }
                }
            }
        }

        // All chunks sent, close sender side
        drop(file); // ensure file handle closed
        drop(req_tx);

        // Await the final SUCCESS response from destination and propagate it
        while let Some(res) = response_stream.next().await {
            // Propagate whatever we received to the original client
            let terminate = matches!(
                &res,
                Ok(ok_res)
                    if ok_res.status == ftp::Status::Success as i32
                        || ok_res.status == ftp::Status::Failure as i32
            );

            if let Ok(ok_res) = &res {
                if ok_res.status == ftp::Status::Success as i32 {
                    // Store metadata in DB
                    if let Some(file_name_os) = file_path.file_name() {
                        let db_doc = DbFileInfo {
                            _id: ObjectId::new(),
                            name: file_name_os.to_string_lossy().to_string(),
                            path: match std::fs::canonicalize(file_path) {
                                Ok(abs) => abs.to_string_lossy().to_string(),
                                Err(_) => file_path.to_string_lossy().to_string(),
                            },
                            sender_bank_id: original_metadata.sender_bank_id.clone(),
                            receiver_bank_id: original_metadata.receiver_bank_id.clone(),
                            message: message.clone(),
                            time_sent_at: original_metadata.timestamp.clone(),
                            time_received_at: if !ok_res.time_received_at.is_empty() {
                                ok_res.time_received_at.clone()
                            } else {
                                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f %Z").to_string()
                            },
                        };
                        let _ = db_clone.store_file_info(db_doc).await;
                    }
                }
            }

            let _ = ack_sender.send(res).await;

            if terminate {
                break;
            }
        }

        Ok(())
    }
}

#[tonic::async_trait]
impl TransferService for FileTransferService {
    type TransferStream = Pin<Box<dyn Stream<Item = Result<TransferResponse, Status>> + Send>>;

    async fn transfer(
        &self,
        request: Request<Streaming<TransferRequest>>,
    ) -> Result<Response<Self::TransferStream>, Status> {
        // tokio_stream::wrappers::ReceiverStream::new(req_rx) from grpc_client.rs
        let mut in_stream = request.into_inner();
        // here tx sends the ACKs, rx receives it, converts it a type that the client understands and then sends it to client (return)
        let (tx, rx) = mpsc::channel(10);
        let self_clone = self.clone();

        tokio::spawn(async move {
            let mut temp_file_path: Option<PathBuf> = None;
            let mut file: Option<fs::File> = None;
            let mut receiver_bank_id: Option<String> = None;
            let mut receiver_bank_ip: Option<String> = None;
            let mut full_metadata: Option<ftp::Metadata> = None;
            let mut message_only_content: Option<Vec<u8>> = None;
            let mut attachment_message: Option<Vec<u8>> = None;
            const SEPARATOR: &[u8] = b"---MESSAGE_END---";

            while let Some(result) = in_stream.next().await {
                match result {
                    Ok(mut req) => {
                        if full_metadata.is_none() {
                            if let Some(metadata) = &req.metadata {
                                full_metadata = Some(metadata.clone());
                                receiver_bank_id = Some(metadata.receiver_bank_id.clone());
                                receiver_bank_ip = Some(metadata.receiver_bank_ip.clone());

                                if matches!(
                                    &metadata.payload_type,
                                    Some(ftp::metadata::PayloadType::MessageInfo(_))
                                ) {
                                    message_only_content = Some(req.content);
                                    break;
                                }
                            }
                        }

                        if file.is_none() {
                            if let Some(metadata) = &req.metadata {
                                let file_info = match &metadata.payload_type {
                                    Some(ftp::metadata::PayloadType::FileInfo(info)) => Some(info),
                                    Some(ftp::metadata::PayloadType::AttachmentInfo(info)) => {
                                        // Attempt to extract inline message on the very first chunk for AttachmentInfo
                                        if attachment_message.is_none() {
                                            if let Some(pos) = req
                                                .content
                                                .windows(SEPARATOR.len())
                                                .position(|window| window == SEPARATOR)
                                            {
                                                attachment_message = Some(req.content[..pos].to_vec());
                                                // DO NOT modify req.content so that destination still receives the message
                                            }
                                        }
                                        info.file_info.as_ref()
                                    }
                                    _ => None,
                                };

                                if let Some(fi) = file_info {
                                    let storage_dir = "received_files";
                                    if fs::create_dir_all(storage_dir).await.is_err() {
                                        let _ = tx
                                            .send(Err(Status::internal(
                                                "Could not create storage dir",
                                            )))
                                            .await;
                                        return;
                                    }
                                    let path = Path::new(storage_dir).join(format!("{}", &fi.name));
                                    temp_file_path = Some(path.clone());
                                    file = Some(fs::File::create(path).await.unwrap());
                                }
                            }
                        }

                        if let Some(f) = file.as_mut() {
                            if !req.content.is_empty() {
                                if f.write_all(&req.content).await.is_err() {
                                    let _ = tx
                                        .send(Err(Status::internal(
                                            "Failed to write chunk to temp file",
                                        )))
                                        .await;
                                    return;
                                }
                                // After successfully persisting the chunk, send an ACK back to the client.
                                let ack = TransferResponse {
                                    transfer_id: req
                                        .metadata
                                        .as_ref()
                                        .map_or_else(String::new, |m| m.transfer_id.clone()),
                                    status: ftp::Status::InProgress as i32,
                                    time_received_at: String::new(),
                                    error_info: Some(ErrorInfo {
                                        error_code: "SERVER".to_string(),
                                        error_details: String::new(),
                                    }),
                                };

                                println!("[SERVER] ACK sent to client for transfer {}", ack.transfer_id);
                                let _ = tx.send(Ok(ack)).await;
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e)).await;
                        return;
                    }
                }
            }

            if let Some(f) = file.as_mut() {
                if f.flush().await.is_err() {
                    let _ = tx
                        .send(Err(Status::internal("Failed to flush temp file")))
                        .await;
                    return;
                }
            }

            // forward the msg/file to destination
            if let (Some(receiver_id), Some(metadata)) = (receiver_bank_id, full_metadata) {
                if let Some(message_content) = message_only_content {
                    // Convert message bytes to String for DB storage
                    let message_str = String::from_utf8_lossy(&message_content).to_string();
                    let metadata_clone = metadata.clone();

                    match self_clone
                        .forward_message(message_content, metadata_clone.clone(), &receiver_id, &receiver_bank_ip.unwrap())
                        .await
                    {
                        Ok(mut forward_stream) => {
                            let db_ref = self_clone.db.clone();
                            while let Some(item) = forward_stream.next().await {
                                let terminate = matches!(
                                    &item,
                                    Ok(resp) if resp.status == ftp::Status::Success as i32 || resp.status == ftp::Status::Failure as i32
                                );

                                if let Ok(resp) = &item {
                                    if resp.status == ftp::Status::Success as i32 {
                                        // Store metadata for message-only transfer
                                        let db_doc = DbFileInfo {
                                            _id: ObjectId::new(),
                                            name: String::new(),
                                            path: String::new(),
                                            sender_bank_id: metadata_clone.sender_bank_id.clone(),
                                            receiver_bank_id: metadata_clone.receiver_bank_id.clone(),
                                            message: message_str.clone(),
                                            time_sent_at: metadata_clone.timestamp.clone(),
                                            time_received_at: if !resp.time_received_at.is_empty() {
                                                resp.time_received_at.clone()
                                            } else {
                                                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f %Z").to_string()
                                            },
                                        };
                                        let _ = db_ref.store_file_info(db_doc).await;
                                    }
                                }

                                if tx.send(item).await.is_err() {
                                    break;
                                }

                                if terminate {
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(Err(e)).await;
                        }
                    }
                } else if let Some(path) = temp_file_path {
                    let msg_for_file = attachment_message.or(message_only_content.clone());

                    match self_clone.forward_file(&path, metadata, &receiver_id, tx.clone(), &receiver_bank_ip.unwrap(), msg_for_file).await {
                           Ok(_) => {
                            // No need to send response back to client as the file transfer is complete
                        }
                        Err(e) => {
                            let _ = tx.send(Err(e)).await;
                        }
                    }

                    if fs::remove_file(path).await.is_err() {
                        eprintln!("[SERVER] Warning: Failed to clean up temporary file");
                    }
                } else {
                    let _ = tx
                        .send(Err(Status::invalid_argument(
                            "No message or file content was found in the request.",
                        )))
                        .await;
                }
            } else {
                let _ = tx
                    .send(Err(Status::invalid_argument(
                        "Receiver information or metadata was missing",
                    )))
                    .await;
            }
        });

        let out_stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(out_stream) as Self::TransferStream))
    }
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let grpc_handle = actix_web::rt::spawn(async move {
        let host = env::var("SERVER_HOST").unwrap();
        let port = env::var("SERVER_PORT").unwrap();
        
        let addr = format!("{}:{}", host, port).parse::<SocketAddr>().expect("[GRPC ADMIN] failed to parse address");
        
        let db = Arc::new(Database::init().await);


        let service = FileTransferService::new(db.clone());
        println!("[GRPC ADMIN] Server listening on {}", addr);
        
        Server::builder()
            .max_frame_size(Some(8 * 1024 * 1024))
            .add_service(
                TransferServiceServer::new(service)
                    .max_encoding_message_size(8 * 1024 * 1024)
                    .max_decoding_message_size(8 * 1024 * 1024),
            )
            .serve(addr)
            .await
            .expect("[GRPC ADMIN] failed to create GRPC ADMIN server");

    });
    
    
    let http_handle = actix_web::rt::spawn(async move {
        let host = env::var("SERVER_HTTP_HOST").unwrap();
        let port = env::var("SERVER_HTTP_PORT").unwrap();
        let addr = format!("{}:{}", host, port).parse::<SocketAddr>().expect("[ADMIN SERVER] failed to parse address");

        
        let db = Database::init().await;
        let db_data = Data::new(db);
        
        println!("[ADMIN SERVER] Starting Actix-web server at http://{}", addr);

        let _ = HttpServer::new(move || {
            let cors = Cors::default()
            .allowed_origin("http://localhost:5173")
            .allowed_origin("http://127.0.0.1:5173")
            .allowed_methods(vec!["GET", "POST"])
            // .allowed_headers(vec![
            //         http::header::AUTHORIZATION,
            //         http::header::ACCEPT,
            //         http::header::CONTENT_TYPE,
            //         http::header::HeaderName::from_static("username"),
            //         http::header::HeaderName::from_static("password"),
            //         http::header::HeaderName::from_static("ip"),
            // ])
            .allow_any_header()
            .supports_credentials()
            .max_age(3600);
    
    
            App::new()
                // .app_data(web::Data::new(app_state.clone()))
                .wrap(cors)
                .app_data(db_data.clone())
                // // .app_data(grpc_client.clone())
                // .configure(routes::configure_routes)
                // PUBLIC routes
                .service(handlers::login)
                // everything under /api requires JWT
                .service(
                    actix_web::web::scope("/api")
                        .wrap(HttpAuthentication::bearer(middleware::validator))
                        .configure(routes::configure_routes)
                )
        })
        .bind(&addr)
        .expect("[ADMIN SERVER] Failed to bind")
        .run()
        .await;


    });
    
    
    let _ = tokio::join!(grpc_handle, http_handle);

    Ok(())
}
