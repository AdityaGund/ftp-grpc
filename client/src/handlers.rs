use actix_multipart::Multipart;
use actix_web::{HttpResponse, Responder};
use futures_util::TryStreamExt;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use tokio::task;
use uuid::Uuid;

use crate::error::AppError;
use crate::grpc_client::{self, ftp::transfer_service_client::TransferServiceClient};

pub async fn upload(mut payload: Multipart) -> Result<impl Responder, AppError> {
    let mut file_path: Option<String> = None;
    let mut file_name: Option<String> = None;
    let mut message: Option<String> = None;
    let mut destination: Option<String> = None;

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
                        f.write_all(&chunk).await?;
                    }
                    file_path = Some(path);
                    file_name = Some(filename);
                }
                Some("message") => {
                    let mut data = Vec::new();
                    while let Some(chunk) = field.try_next().await? {
                        data.extend_from_slice(&chunk);
                    }
                    if let Ok(s) = String::from_utf8(data) {
                        message = Some(s);
                    }
                }
                Some("destination") => {
                    let mut data = Vec::new();
                    while let Some(chunk) = field.try_next().await? {
                        data.extend_from_slice(&chunk);
                    }
                    if let Ok(s) = String::from_utf8(data) {
                        destination = Some(s);
                    }
                }
                _ => (),
            }
        }
    }

    // if file_path.is_none() && message.is_none() && destination.is_none() {
    //     return Ok(HttpResponse::BadRequest().body("A file/message & destination must be provided."));
    // }

    let response_json = serde_json::json!({
        "message": "Data transfer initiated.",
        "file_name": &file_name,
        "sent_message": &message,
        "destination": &destination,
    });
    
    // connect to B server
    task::spawn(async move {
        let host = std::env::var("SERVER_HOST").unwrap().to_string();
        let port = std::env::var("SERVER_PORT").unwrap().to_string();
        let url = format!("http://{}:{}", host, port);

        match TransferServiceClient::connect(url).await {
            Ok(mut client) => {
                println!("[CLIENT] connected to server");
                let file_details = file_path.as_ref().zip(file_name.as_ref())
                    .map(|(p, n)| (p.as_str(), n.as_str()));

                if let Err(e) = grpc_client::transfer_data(
                    &mut client,
                    file_details,
                    message.as_deref(),
                    destination.as_deref()
                )
                .await
                {
                    eprintln!("Failed to send data via gRPC: {}", e);
                } else {
                    println!("Data transfer stream finished.");
                    if let Some(path) = &file_path {
                        if let Err(e) = fs::remove_file(path).await {
                            eprintln!("Failed to remove temporary file '{}': {}", path, e);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to connect to gRPC server: {}", e);
            }
        }
    });

    Ok(HttpResponse::Ok().json(response_json))
}