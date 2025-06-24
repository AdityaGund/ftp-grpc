// use std::default;

use actix_multipart::Multipart;
// use actix_web::web::Bytes;
// use actix_web::{HttpResponse, web, Responder};
use actix_web::{web, HttpResponse};
use futures_util::{TryStreamExt};
// use serde::de::value::Error;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
// use tokio::task;
// use tonic::{Response, Streaming};
// use uuid::Uuid;
// use bytes::Bytes;
// use tokio_stream::wrappers::BroadcastStream;
// use futures_util::StreamExt;
// use std::convert::Infallible;
// use crate::AppState;

use crate::error::{self, AppError};
// use crate::grpc_client::TransferResponse;
use crate::grpc_client::{self, ftp::transfer_service_client::TransferServiceClient};
use crate::services::db::Database;
use serde_json;
use actix_web::{post, get};
use mongodb::bson::oid::ObjectId;
use chrono::Utc;

// pub async fn upload(state: web::Data<AppState>, mut payload: Multipart) -> Result<HttpResponse, AppError> {
#[post("/upload")]
pub async fn upload(
    mut payload: Multipart,
    db: web::Data<std::sync::Arc<Database>>,
) -> Result<HttpResponse, AppError> {
    let mut file_path: Option<String> = None;
    let mut file_name: Option<String> = None;
    let mut message: Option<String> = None;
    let mut destination: Option<String> = None;
    let mut destination_ip: Option<String> = None;
    let mut sender: Option<String> = None;

    fs::create_dir_all("./temp").await?;

    while let Some(mut field) = payload.try_next().await? {
        if let Some(content_disposition) = field.content_disposition() {
            // println!("[DEBUG] content_disposition: {:?}", content_disposition.parameters);

            match content_disposition.get_name() {
                Some("file") => {
                    let filename = content_disposition
                        .get_filename()
                        .unwrap_or("unknown_file")
                        .to_string();

                    // let unique_id = Uuid::new_v4().to_string();
                    let temp_file_name = format!("{}", &filename);
                    let path = format!("./temp/{}", temp_file_name);

                    // store file temporarily on client-side
                    let mut f = File::create(&path).await?;

                    while let Some(chunk) = field.try_next().await? {
                        f.write_all(chunk.as_ref()).await?;
                    }
                    file_path = Some(path);
                    file_name = Some(filename);
                }
                Some("message") => {
                    let mut data = Vec::new();
                    while let Some(chunk) = field.try_next().await? {
                        data.extend_from_slice(chunk.as_ref());
                    }
                    if let Ok(s) = String::from_utf8(data) {
                        message = Some(s);
                    }
                }
                Some("destination") => {
                    let mut data = Vec::new();
                    while let Some(chunk) = field.try_next().await? {
                        data.extend_from_slice(chunk.as_ref());
                    }
                    if let Ok(s) = String::from_utf8(data) {
                        destination = Some(s);
                    }
                },
                Some("destinationIp") => {
                    let mut data = Vec::new();
                    while let Some(chunk) = field.try_next().await? {
                        data.extend_from_slice(chunk.as_ref());
                    }
                    if let Ok(s) = String::from_utf8(data) {
                        destination_ip = Some(s);
                    }
                },
                Some("sender") => {
                    let mut data = Vec::new();
                    while let Some(chunk) = field.try_next().await? {
                        data.extend_from_slice(chunk.as_ref());
                    }
                    if let Ok(s) = String::from_utf8(data) {
                        sender = Some(s);
                    }
                }
                _ => (),
            }
        }
    }

    // Perform transfer and build response depending on the outcome
    let transfer_result: Result<String, AppError>;

    // connect to B server
    // task::spawn(async move {
    let host = std::env::var("SERVER_HOST").unwrap().to_string();
    let port = std::env::var("SERVER_PORT").unwrap().to_string();
    let url = format!("http://{}:{}", host, port);

        match TransferServiceClient::connect(url).await {
            Ok(mut client) => {
                println!("[CLIENT] connected to server");
                let file_details = file_path.as_ref().zip(file_name.as_ref())
                    .map(|(p, n)| (p.as_str(), n.as_str()));

                transfer_result = grpc_client::transfer_data(
                    &mut client,
                    file_details,
                    message.as_deref(),
                    destination.as_deref(),
                    destination_ip.as_deref(),
                    sender.as_deref(),
                    // Some(state.notifier.clone()),
                ).await;

                // if let Err(e) = grpc_client::transfer_data(
                //     &mut client,
                //     file_details,
                //     message.as_deref(),
                //     destination.as_deref()
                // )
                // {
                //     eprintln!("Failed to send data via gRPC: {}", e);
                // } else {
                //     println!("Data transfer stream finished.");
                //     if let Some(path) = &file_path {
                //         if let Err(e) = fs::remove_file(path).await {
                //             eprintln!("Failed to remove temporary file '{}': {}", path, e);
                //         }
                //     }
                // }
            }
            Err(e) => {
                eprintln!("Failed to connect to gRPC server: {}", e);
                return Err(error::AppError::ClientError("error".to_string()));
            }
        }
    // });

    // Decide which HTTP response to send
    match transfer_result {
        Ok(time_received) => {
            // Persist metadata
            let now = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f %Z").to_string();
            let doc = crate::models::file_info_model::FileInfo {
                _id: ObjectId::new(),
                name: file_name.clone().unwrap_or_default(),
                path: file_path.clone().unwrap_or_default(),
                sender_bank_id: sender.clone().unwrap_or_default(),
                receiver_bank_id: destination.clone().unwrap_or_default(),
                message: message.clone().unwrap_or_default(),
                time_sent_at: now.clone(),
                time_received_at: time_received.clone(),
            };
            let _ = db.store_file_info(doc).await;

            Ok(HttpResponse::Ok().json(serde_json::json!({
                "message": "Data transfer success.",
                "file_name": &file_name,
                "sent_message": &message,
                "destination": &destination,
                "destination_ip": &destination_ip,
                "sender": &sender,
            })))
        },
        Err(e) => Err(e), // will be converted to proper HTTP error by ResponseError impl
    }
}

#[get("/file-info")]
pub async fn fetch_info(db: web::Data<std::sync::Arc<Database>>) -> Result<HttpResponse, AppError> {
    // Fetch all file information from the database
    let files = db.get_file_info().await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "data": files
    })))
}
// pub async fn events_stream(state: web::Data<AppState>) -> impl Responder {
//     let rx = state.notifier.subscribe();
//     let stream = BroadcastStream::new(rx).map(|msg| match msg {
//         Ok(resp) => {
//             let origin = resp
//                     .error_info
//                     .as_ref()
//                     .map(|e| e.error_code.as_str())
//                     .unwrap_or("UNKNOWN");
//             let json = format!("{{\"transfer_id\":\"{}\", \"origin\":\"{}\", \"status\":{}}}", resp.transfer_id, origin, resp.status);
//             Ok::<Bytes, Infallible>(Bytes::from(format!("data: {}\n\n", json)))
//         },
//         Err(_) => Ok(Bytes::from("event: ping\n\n")),
//     });

//     HttpResponse::Ok()
//         .insert_header(("Content-Type", "text/event-stream"))
//         .insert_header(("Cache-Control", "no-cache"))
//         .insert_header(("Connection", "keep-alive"))
//         .streaming(stream)
// }
