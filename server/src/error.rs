use actix_web::{ResponseError, HttpResponse};
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    IoError(std::io::Error),
    TonicStatus(tonic::Status),
    ClientError(String),
    DatabaseError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::IoError(e) => write!(f, "IO error: {}", e),
            AppError::TonicStatus(e) => write!(f, "gRPC error: {}", e),
            AppError::ClientError(e) => write!(f, "Client error: {}", e),
            AppError::DatabaseError(e) => write!(f, "Database error: {}", e),
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::IoError(_) => HttpResponse::InternalServerError().json("Internal server error"),
            AppError::TonicStatus(_) => HttpResponse::InternalServerError().json("File transfer service unavailable"),
            AppError::ClientError(msg) => HttpResponse::BadRequest().json(msg),
            AppError::DatabaseError(msg) => HttpResponse::InternalServerError().json(msg),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IoError(err)
    }
}

impl From<tonic::Status> for AppError {
    fn from(err: tonic::Status) -> Self {
        AppError::TonicStatus(err)
    }
}