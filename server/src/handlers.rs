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

#[post("/login")]
pub async fn login(req: HttpRequest, db: web::Data<Database>) -> Result<HttpResponse, AppError> {
    // println!("[ADMIN SERVER] login request received");
    let auth = AuthService::new();
     println!("[ADMIN SERVER] login request received, extracting values");
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
    println!("token generatec {token}");
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