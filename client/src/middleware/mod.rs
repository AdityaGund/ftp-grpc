use actix_web::dev::ServiceRequest;
use actix_web::{Error, HttpMessage};
use actix_web_httpauth::extractors::bearer::BearerAuth;

use crate::services::auth::AuthService;

pub async fn validator(mut req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let auth_service = AuthService::new();

    match auth_service.verify_token(credentials.token()) {
        Ok(token_data) => {
            // Ensure the authenticated user is a bank user
            if token_data.claims.role != "bank" {
                return Err((actix_web::error::ErrorUnauthorized("Only bank users are allowed"), req));
            }

            // Attach claims to request for downstream handlers
            req.extensions_mut().insert(token_data.claims);
            Ok(req)
        }
        Err(e) => Err((actix_web::error::ErrorUnauthorized(e.to_string()), req)),
    }
} 