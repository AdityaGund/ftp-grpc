use std::env;

use argon2::{password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString}, Argon2};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, Algorithm, TokenData};
use serde::{Serialize, Deserialize};

use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Clone)]
pub struct AuthService {
    secret: String,
}

impl AuthService {
    pub fn new() -> Self {
        let secret = env::var("JWT_SECRET").expect("JWT_SECRET env var required");
        Self { secret }
    }

    pub fn hash_password(&self, password: &str) -> Result<String, AppError> {
        println!("[JWT] HASHING PASSWORD");
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        Ok(argon2.hash_password(password.as_bytes(), &salt)
            .map_err(|_| AppError::ClientError("Password hash error".into()))?
            .to_string())
    }

    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, AppError> {
        println!("[JWT] VERIFYING PASSWORD");
        let parsed = PasswordHash::new(hash).map_err(|_| AppError::ClientError("Invalid hash".into()))?;
        Ok(Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok())
    }

    pub fn generate_token(&self, username: &str, role: &str, minutes: i64) -> Result<String, AppError> {
        println!("[JWT] GENERATING TOKEN");
        let now = Utc::now();
        let claims = Claims {
            sub: username.to_owned(),
            role: role.to_owned(),
            iat: now.timestamp() as usize,
            exp: (now + Duration::minutes(minutes)).timestamp() as usize,
        };
        encode(&Header::new(Algorithm::HS256), &claims, &EncodingKey::from_secret(self.secret.as_bytes()))
            .map_err(|_| AppError::ClientError("Token creation failed".into()))
    }

    pub fn verify_token(&self, token: &str) -> Result<TokenData<Claims>, AppError> {
        println!("[JWT] VERIFYING TOKEN");
        decode::<Claims>(token, &DecodingKey::from_secret(self.secret.as_bytes()), &Validation::new(Algorithm::HS256))
            .map_err(|_| AppError::ClientError("Invalid/expired token".into()))
    }
} 