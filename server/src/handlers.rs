use actix_web::{web, HttpRequest, HttpResponse};
use actix_web::HttpMessage;
// use actix_web::HttpMessage;
use mongodb::bson::oid::ObjectId;
// use mongodb::results::InsertOneResult;
use crate::error::AppError;
use crate::models::user_model::{AdminUser, Bank};
use crate::services::db::Database;
use actix_web::{get, post};
use crate::services::auth::AuthService;
use serde_json::json;
use actix_multipart::Multipart;
use futures_util::TryStreamExt;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use crate::models::file_info_model::FileInfo;
use chrono::Utc;
use tokio_util::io::StreamReader;
use futures_util::stream::StreamExt;

#[post("/login")]
pub async fn login(req: HttpRequest, db: web::Data<Database>) -> Result<HttpResponse, AppError> {
    // println!("[ADMIN SERVER] login request received");
    let auth = AuthService::new();
    // println!("[ADMIN SERVER] login request received, extracting values");
    // Helper to extract header value as &str
    let header_str = |name: &str| -> Result<&str, AppError> {
        req.headers()
            .get(name)
            .ok_or_else(|| AppError::ClientError(format!("Missing `{}` header", name)))?
            .to_str()
            .map_err(|_| AppError::ClientError(format!("Invalid `{}` header", name)))
    };
    // println!("[ADMIN SERVER] login request received extracting values ");
    let username = header_str("username")?.to_owned();
    let password = header_str("password")?.to_owned();
    // let role     = header_str("role")?.to_lowercase();
    let role = if username.starts_with('A') {
        "admin".to_string()
    } else if username.starts_with('B') {
        "bank".to_string()
    } else {
        return Err(AppError::ClientError("Invalid username format".into()));
    };
    // println!("[ADMIN SERVER] {username}, {password}, {role}");

    // Fetch stored user
    let stored_hash_opt = match role.as_str() {
        "bank" => db.find_bank_by_username(&username).await?.map(|b| b.password),
        "admin" => db.find_admin_by_username(&username).await?.map(|a| a.password),
        _ => None,
    };

    let Some(stored_hash) = stored_hash_opt else {
        return Err(AppError::ClientError("Invalid username or password".into()));
    };

    // verify
    if !auth.verify_password(&password, &stored_hash)? {
        return Err(AppError::ClientError("Invalid password".into()));
    }

    let token = auth.generate_token(&username, &role, 60)?;
    // println!("token generatec {token}");
    Ok(HttpResponse::Ok().json(json!({"token": token,"role": role, "username": username})))
}

#[post("/add")]
pub async fn add_user(req: HttpRequest, db: web::Data<Database>) -> Result<HttpResponse, AppError> {
    let auth = AuthService::new();
    // Helper to extract header value as &str
    let header_str = |name: &str| -> Result<&str, AppError> {
        req.headers()
            .get(name)
            .ok_or_else(|| AppError::ClientError(format!("Missing `{}` header", name)))?
            .to_str()
            .map_err(|_| AppError::ClientError(format!("Invalid `{}` header", name)))
    };

    let username = header_str("username")?.to_owned();
    let password = header_str("password")?.to_owned();
    let ip     = header_str("ip")?.to_owned();

    // println!("[ADMIN SERVER] {username}, {password}, {role}");

    // access claims to ensure admin role
    if let Some(claims) = req.extensions().get::<crate::services::auth::Claims>() {
        if claims.role != "admin" {
            return Err(AppError::ClientError("Only admin can add user".into()));
        }
    } else {
        return Err(AppError::ClientError("Unauthorized".into()));
    }

    let hashed_pw = auth.hash_password(&password)?;

    
    let a = username.chars().nth(0);
    let role: String;

    println!("{a:?}");
    if a.unwrap() == 'B' {
        role = "bank".to_string();
    } else {
        role = "admin".to_string();
    }

    match role.as_str() {
        "bank"=> {
            let bank_user = Bank {
                _id: ObjectId::new(),
                username: username.clone(),
                password: hashed_pw.clone(),
                ip: ip.clone(),
            };
            let _ = db.add_bank(bank_user).await?;
        },
        "admin" => {
            let admin_user = AdminUser {
                _id: ObjectId::new(),
                username: username.clone(),
                password: hashed_pw.clone(),
            };
            let _ = db.add_admin(admin_user).await;
        },
        _ => {
            return Err(AppError::ClientError(
                "`role` header must be either `Bank` or `Admin`".into(),
            ))
        }
    }

    println!("[ADMIN SERVER] Added user `{}` as `{}`", username, role);

    Ok(HttpResponse::Ok().json(format!("Successfuly added {} as {}", username, role)))
}


#[get("/available")]
pub async fn available_banks(db: web::Data<Database>) -> Result<HttpResponse, AppError> {

    let banks = db.get_banks().await?;

    Ok(HttpResponse::Ok().json(banks))
}

#[post("/delete")]
pub async fn delete_user(req: HttpRequest, db: web::Data<Database>) -> Result<HttpResponse, AppError> {
    // Only admins can delete users
    if let Some(claims) = req.extensions().get::<crate::services::auth::Claims>() {
        if claims.role != "admin" {
            return Err(AppError::ClientError("Only admin can delete users".into()));
        }
    } else {
        return Err(AppError::ClientError("Unauthorized".into()));
    }

    let header_str = |name: &str| -> Result<&str, AppError> {
        req.headers()
            .get(name)
            .ok_or_else(|| AppError::ClientError(format!("Missing `{}` header", name)))?
            .to_str()
            .map_err(|_| AppError::ClientError(format!("Invalid `{}` header", name)))
    };

    let username = header_str("username")?.to_owned();

    // Determine role based on first char again
    let role = if username.starts_with('B') { "bank" } else { "admin" };

    match role {
        "bank" => {
            db.delete_bank(&username).await?;
        },
        "admin" => {
            db.delete_admin(&username).await?;
        },
        _ => {
            return Err(AppError::ClientError("Invalid username format".into()));
        }
    }

    Ok(HttpResponse::Ok().json(format!("Deleted user {}", username)))
}

#[post("/update")]
pub async fn update_user(req: HttpRequest, db: web::Data<Database>) -> Result<HttpResponse, AppError> {
    // Only admins can update users
    if let Some(claims) = req.extensions().get::<crate::services::auth::Claims>() {
        if claims.role != "admin" {
            return Err(AppError::ClientError("Only admin can update users".into()));
        }
    } else {
        return Err(AppError::ClientError("Unauthorized".into()));
    }

    let header_str = |name: &str| -> Option<String> {
        req.headers().get(name).and_then(|v| v.to_str().ok()).map(|s| s.to_owned())
    };

    let username = header_str("username").ok_or_else(|| AppError::ClientError("Missing `username` header".into()))?;
    let new_password = header_str("password");
    let new_ip = header_str("ip");

    if new_password.is_none() && new_ip.is_none() {
        return Err(AppError::ClientError("Nothing to update".into()));
    }

    let auth = AuthService::new();
    let hashed_pw = if let Some(pw) = &new_password { Some(auth.hash_password(pw)?)} else { None };

    let role = if username.starts_with('B') { "bank" } else { "admin" };

    match role {
        "bank" => {
            db.update_bank(&username, hashed_pw.as_deref(), new_ip.as_deref()).await?;
        },
        "admin" => {
            if let Some(pw) = hashed_pw {
                db.update_admin_password(&username, &pw).await?;
            } else {
                return Err(AppError::ClientError("Password required to update admin user".into()));
            }
        },
        _ => {
            return Err(AppError::ClientError("Invalid username format".into()));
        }
    }

    Ok(HttpResponse::Ok().json(format!("Updated user {}", username)))
}

#[get("/users")]
pub async fn list_users(db: web::Data<Database>) -> Result<HttpResponse, AppError> {
    let banks = db.get_banks().await?;
    let admins = db.get_admins().await?;

    Ok(HttpResponse::Ok().json(json!({
        "banks": banks,
        "admins": admins,
    })))
}

#[get("/file-info")]
pub async fn fetch_file_info(db: web::Data<Database>) -> Result<HttpResponse, AppError> {
    let files = db.get_file_info().await?;

    Ok(HttpResponse::Ok().json(json!({
        "data": files
    })))
}

#[post("/admin-upload")]
pub async fn admin_upload(
    mut payload: Multipart,
    db: web::Data<crate::services::db::Database>,
) -> Result<HttpResponse, AppError> {
    // Similar to client upload handler but operates from admin server context
    let mut file_path: Option<String> = None;
    let mut file_name: Option<String> = None;
    let mut message: Option<String> = None;
    let mut destinations_field: Option<String> = None;
    let mut sender: Option<String> = None;

    println!("admin_upload called");
    // Ensure temp directory exists
    fs::create_dir_all("./temp").await?;

    while let Some(mut field) = payload.try_next().await? {
        println!("here");
        if let Some(cd) = field.content_disposition() {
            match cd.get_name() {
                Some("file") => {
                    let filename = cd.get_filename().unwrap_or("unknown_file").to_string();
                    let temp_path = format!("./temp/{}", &filename);
                    let mut f = File::create(&temp_path).await?;
                    // Convert the multipart field (a Stream of Bytes) into an AsyncRead
                    // so we can leverage Tokio's highly-optimised copy implementation.
                    let mut stream_reader = StreamReader::new(
                        field
                            .map_ok(|bytes| bytes)
                            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string())),
                    );
                    tokio::io::copy(&mut stream_reader, &mut f).await?;
                    file_path = Some(temp_path);
                    file_name = Some(filename);
                }
                Some("message") => {
                    let mut data = Vec::<u8>::new();
                    while let Some(chunk) = field.try_next().await? {
                        data.extend_from_slice(chunk.as_ref());
                    }
                    if let Ok(s) = String::from_utf8(data) {
                        message = Some(s);
                    }
                }
                Some("destinations") => {
                    let mut data = Vec::<u8>::new();
                    while let Some(chunk) = field.try_next().await? {
                        data.extend_from_slice(chunk.as_ref());
                    }
                    if let Ok(s) = String::from_utf8(data) {
                        destinations_field = Some(s);
                    }
                }
                Some("sender") => {
                    let mut data = Vec::<u8>::new();
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

    println!("parsing destinations");
    // Parse destinations JSON array
    let destinations: Vec<serde_json::Value> = match &destinations_field {
        Some(s) => serde_json::from_str(&s).map_err(|_| AppError::ClientError("Invalid destination format".into()))?,
        None => vec![],
    };
    println!("parsed");
    
    // Prepare shared data for concurrent transfers
    let file_details = file_path.as_ref().zip(file_name.as_ref())
    .map(|(p, n)| (p.clone(), n.clone())); // clone for move into tasks

    let message_clone = message.clone();
    let sender_clone = sender.clone();

    let mut tasks = Vec::new();

    println!("parsed");
    println!("FOR LOOP started");
    for dest in &destinations {
        println!("inside for loop");
        let username = dest.get("username").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let ip = dest.get("ip").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let file_details = file_details.clone();
        let message = message_clone.clone();
        let sender = sender_clone.clone();
        let db = db.clone();
        let file_name = file_name.clone();
        let file_path = file_path.clone();

        println!("sending to destination");
        tasks.push(async move {
            // Build gRPC URL for each destination
            let url = if ip.starts_with("http") { ip.clone() } else { format!("http://{}:50053", ip) };
            
            println!("connecting to grpc destination");
            match crate::grpc_client::TransferServiceClient::connect(url).await {
                Ok(mut client) => {
                    let file_details_ref = file_details.as_ref().map(|(p, n)| (p.as_str(), n.as_str()));

                    let now = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f %Z").to_string();
                    println!("starting transfer");
                    match crate::grpc_client::transfer_data(
                        &mut client,
                        file_details_ref,
                        message.as_deref(),
                        Some(&username),
                        Some(&ip),
                        sender.as_deref(),
                    ).await {
                        Ok(time_received) => {
                            // Store metadata
                            let doc = FileInfo {
                                _id: ObjectId::new(),
                                name: file_name.clone().unwrap_or_default(),
                                path: file_path.clone().unwrap_or_default(),
                                sender_bank_id: sender.clone().unwrap_or_default(),
                                receiver_bank_id: username.clone(),
                                message: message.clone().unwrap_or_default(),
                                time_sent_at: now,
                                time_received_at: time_received.clone(),
                            };
                            let _ = db.store_file_info(doc).await;

                            (username, ip, true, None::<String>)
                        }
                        Err(e) => (username, ip, false, Some(e.to_string())),
                    }
                }
                Err(e) => (username, ip, false, Some(e.to_string())),
            }
        });
    }

    let results = futures::future::join_all(tasks).await;

    let all_ok = results.iter().all(|(_, _, ok, _)| *ok);

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": if all_ok { "Data transfer success." } else { "Partial or full failure." },
        "file_name": &file_name,
        "sent_message": &message,
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