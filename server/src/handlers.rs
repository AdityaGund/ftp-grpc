use actix_web::{web, HttpRequest, HttpResponse};
// use actix_web::HttpMessage;
use mongodb::bson::oid::ObjectId;
// use mongodb::results::InsertOneResult;
use crate::error::AppError;
use crate::models::user_model::{AdminUser, Bank};
use crate::services::db::Database;
use actix_web::{get, post};
use crate::services::auth::AuthService;
use serde_json::json;

#[post("/login")]
pub async fn login(req: HttpRequest, db: web::Data<Database>) -> Result<HttpResponse, AppError> {
    // println!("[ADMIN SERVER] login request received");
    let auth = AuthService::new();
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
    Ok(HttpResponse::Ok().json(json!({"token": token})))
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
    // if let Some(claims) = req.extensions().get::<crate::services::auth::Claims>() {
    //     if claims.role != "admin" {
    //         return Err(AppError::ClientError("Only admin can add user".into()));
    //     }
    // } else {
    //     return Err(AppError::ClientError("Unauthorized".into()));
    // }

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