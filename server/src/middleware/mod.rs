use actix_web::dev::ServiceRequest;
use actix_web::{Error, HttpMessage};
use actix_web_httpauth::extractors::bearer::BearerAuth;

use crate::services::auth::AuthService;

pub async fn validator(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    // public paths
    // println!("[ADMIN SERVER DEBUG] HERE 1");
    let path = req.path();
    if path == "/login" {
        // println!("[ADMIN SERVER DEBUG] HERE 2");
        return Ok(req);
    }

    let auth_service = AuthService::new();
    match auth_service.verify_token(credentials.token()) {
        Ok(token_data) => {
            let req = req;
            req.extensions_mut().insert(token_data.claims);
            Ok(req)
        }
        Err(e) => Err((actix_web::error::ErrorUnauthorized(e.to_string()), req)),
    }
} 