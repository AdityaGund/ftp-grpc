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
use tokio_util::io::StreamReader;
use futures_util::stream::StreamExt;

use crate::error::{self, AppError};
// use crate::grpc_client::TransferResponse;
use crate::grpc_client::{self, ftp::transfer_service_client::TransferServiceClient};
use crate::services::db::Database;
use serde_json;
use actix_web::{post, get};
use futures::future::join_all; //for parallel processing of multiple file transfers

use mongodb::bson::oid::ObjectId;
use chrono::Utc;

// pub async fn upload(state: web::Data<AppState>, mut payload: Multipart) -> Result<HttpResponse, AppError> {
#[post("/upload")]
pub async fn upload(
    mut payload: Multipart,
    db: web::Data<std::sync::Arc<Database>>,
) -> Result<HttpResponse, AppError> {
    println!("upload called");
    let mut file_path: Option<String> = None;
    let mut file_name: Option<String> = None;
    let mut message: Option<String> = None;
    let mut destinations: Option<String> = None;
    // let mut destination_ip: Option<String> = None;
    let mut sender: Option<String> = None;
    // Timestamp when the upload was initiated
    let sent_time = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f %Z").to_string();
    
    fs::create_dir_all("./temp").await?;
    
    println!("starting while");
    while let Some(mut field) = payload.try_next().await? {
        println!("here");
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
                        f.write_all(&chunk).await?;
                    }
                    f.flush().await?;
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
                Some("destinations") => {
                    let mut data = Vec::new();
                    while let Some(chunk) = field.try_next().await? {
                        data.extend_from_slice(chunk.as_ref());
                    }
                    if let Ok(s) = String::from_utf8(data) {
                        destinations = Some(s);
                    }
                },
                // Some("destinationIp") => {
                //     let mut data = Vec::new();
                //     while let Some(chunk) = field.try_next().await? {
                //         data.extend_from_slice(chunk.as_ref());
                //     }
                //     if let Ok(s) = String::from_utf8(data) {
                //         destination_ip = Some(s);
                //     }
                // },
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

    println!("parsing destination");
    let destinations :Vec<serde_json::Value> = match &destinations {
        Some(s) => serde_json::from_str(&s).map_err(|_| AppError::ClientError("Invalid destination format".to_string()))?,
        None => vec![],
    };
    let file_details = file_path.as_ref().zip(file_name.as_ref())
    .map(|(p, n)| (p.clone(), n.clone())); // clone for move into tasks

    let message = message.clone();
    let sender = sender.clone();
    // Spawn each transfer in its own Tokio task so the runtime can schedule them in parallel on
    // the multithreaded executor. We collect the JoinHandles so we can await all of them later
    // and aggregate the individual results for the HTTP response.
    let mut tasks: Vec<tokio::task::JoinHandle<(String, String, bool, Option<String>)>> = Vec::new();

    println!("outside for");
    for dest in &destinations {
        println!("inside for");
        let username = dest
            .get("username")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let ip = dest
            .get("ip")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let file_details = file_details.clone();
        let message = message.clone();
        let sender = sender.clone();
        let db = db.clone();
        let sent_time_clone = sent_time.clone();
        let file_name = file_name.clone();
        let file_path = file_path.clone();

        println!("spawning transfer task");
        let handle = tokio::spawn(async move {
            println!("starting");
            let port = std::env::var("SERVER_PORT").unwrap();
            let host = std::env::var("SERVER_HOST").unwrap();
            let url = format!("http://{}:{}", host, port);
            
            println!("connecting to server");
            match TransferServiceClient::connect(url).await {
                Ok(mut client) => {
                    println!("[CLIENT] connected to server");
                    let file_details_ref = file_details
                        .as_ref()
                        .map(|(p, n)| (p.as_str(), n.as_str()));

                    match grpc_client::transfer_data(
                        &mut client,
                        file_details_ref,
                        message.as_deref(),
                        Some(&username),
                        Some(&ip),
                        sender.as_deref(),
                    )
                    .await
                    .map_err(|e| e.to_string())
                    {
                        Ok(time_received) => {
                            let doc = crate::models::file_info_model::FileInfo {
                                _id: ObjectId::new(),
                                name: file_name.clone().unwrap_or_default(),
                                path: file_path.clone().unwrap_or_default(),
                                sender_bank_id: sender.clone().unwrap_or_default(),
                                receiver_bank_id: username.clone(),
                                message: message.clone().unwrap_or_default(),
                                time_sent_at: sent_time_clone.clone(),
                                time_received_at: time_received.clone(),
                            };
                            let _ = db.store_file_info(doc).await.ok();

                            (username, ip, true, None::<String>)
                        }
                        Err(err_msg) => (username, ip, false, Some(err_msg)),
                    }
                }
                Err(e) => (username, ip, false, Some(e.to_string())),
            }
        });
        tasks.push(handle);
    }

    // Await all parallel transfer tasks. A JoinHandle may error if the task panicked; convert
    // such errors into a regular failed-transfer entry so the frontend can still display it.
    let mut results = Vec::with_capacity(tasks.len());
    for handle in join_all(tasks).await {
        match handle {
            Ok(r) => results.push(r),
            Err(e) => results.push(("".into(), "".into(), false, Some(format!("task join error: {e}"))))
        }
    }

    let all_ok = results.iter().all(|(_, _, ok, _)| *ok);

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": if all_ok { "Data transfer success." } else { "Partial or full failure." },
        "file_name": &file_name,
        "sent_message": &message,
        "sent_time": &sent_time,
        "sender": &sender,
        "results": results.iter().map(|(username, ip, ok, err)| {
            serde_json::json!({
                "destination": username,
                "destination_ip": ip,
                "status": if *ok { "success" } else { "failed" },
                "error": err
            })
        }).collect::<Vec<_>>()
    })))

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
