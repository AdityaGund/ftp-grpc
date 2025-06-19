use std::env;
use std::fs;

use argon2::{password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString}, Argon2};
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
    decoding_key: DecodingKey,
}

impl AuthService {
    pub fn new() -> Self {
        let key_path = env::var("JWT_PUBLIC_KEY_PATH").expect("JWT_PUBLIC_KEY_PATH env var required");
        let key_bytes = fs::read(&key_path).expect("Cannot read RSA public key");
        let decoding_key = DecodingKey::from_rsa_pem(&key_bytes).expect("Invalid RSA public key");

        Self { decoding_key }
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

    // The client/bank side should never generate an admin-signed token, so calling
    // this function is now a logic error. If you truly need token generation on
    // the client, give it its *own* private key and expose its public key to any
    // verifier instead.
    // #[allow(dead_code)]
    // pub fn generate_token(&self, _username: &str, _role: &str, _minutes: i64) -> Result<String, AppError> {
    //     Err(AppError::ClientError("Token generation is not supported on client side".into()))
    // }

    pub fn verify_token(&self, token: &str) -> Result<TokenData<Claims>, AppError> {
        // println!("[JWT] VERIFYING TOKEN");
        decode::<Claims>(token, &self.decoding_key, &Validation::new(Algorithm::RS256))
            .map_err(|_| AppError::ClientError("Invalid/expired token".into()))
    }
} 