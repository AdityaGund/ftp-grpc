use std::env;
use std::fs;

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
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl AuthService {
    pub fn new() -> Self {
        // Read RSA private key from the file whose path is provided through the
        // `JWT_PRIVATE_KEY_PATH` env var. This key MUST be in PEM PKCS#8 or PKCS#1
        // format accepted by `jsonwebtoken`.
        let key_path = env::var("JWT_PRIVATE_KEY_PATH")
            .expect("JWT_PRIVATE_KEY_PATH env var required (should point to RSA private key PEM file)");
        let key_bytes = fs::read(&key_path)
            .expect("Cannot read RSA private key");

        // Build encoding/decoding keys once and reuse them.
        let encoding_key = EncodingKey::from_rsa_pem(&key_bytes)
            .expect("Invalid RSA private key (encoding)");
        let decoding_key = DecodingKey::from_rsa_pem(&key_bytes)
            .expect("Invalid RSA private key (decoding)");

        Self { encoding_key, decoding_key }
    }

    pub fn hash_password(&self, password: &str) -> Result<String, AppError> {
        // println!("[JWT] HASHING PASSWORD");
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        Ok(argon2.hash_password(password.as_bytes(), &salt)
            .map_err(|_| AppError::ClientError("Password hash error".into()))?
            .to_string())
    }

    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, AppError> {
        // println!("[JWT] VERIFYING PASSWORD");
        let parsed = PasswordHash::new(hash).map_err(|_| AppError::ClientError("Invalid hash".into()))?;
        Ok(Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok())
    }

    pub fn generate_token(&self, username: &str, role: &str, minutes: i64) -> Result<String, AppError> {
        // println!("[JWT] GENERATING TOKEN");
        let now = Utc::now();
        let claims = Claims {
            sub: username.to_owned(),
            role: role.to_owned(),
            iat: now.timestamp() as usize,
            exp: (now + Duration::minutes(minutes)).timestamp() as usize,
        };
        encode(&Header::new(Algorithm::RS256), &claims, &self.encoding_key)
            .map_err(|_| AppError::ClientError("Token creation failed".into()))
    }

    pub fn verify_token(&self, token: &str) -> Result<TokenData<Claims>, AppError> {
        // println!("[JWT] VERIFYING TOKEN");
        decode::<Claims>(token, &self.decoding_key, &Validation::new(Algorithm::RS256))
            .map_err(|_| AppError::ClientError("Invalid/expired token".into()))
    }
} 