#[derive(Clone)]
pub struct Database{
    file_info: Collection<FileInfo>
}

use std::path::Path;
use mongodb::results::InsertOneResult;
use mongodb::{Client, Collection};
use crate::error::AppError;
use bson::{doc};
use dotenv::dotenv;
use std::env;

#[derive(Debug, serde::Serialize)]
pub struct BankSummary {
    pub username: String,
    pub ip: String,
}


use crate::models::file_info_model::FileInfo;
impl Database {
    
    pub async fn init() -> Self {
        // let dest_env_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(".env");
        // if dotenv::from_path(dest_env_path.as_path()).is_err() {
        //     dotenv().ok();
        // };
        let uri = env::var("MONGO_URI").unwrap().to_string();
        let client = Client::with_uri_str(uri).await.unwrap();

        let db = client.database("client_db");

        let file_info: Collection<FileInfo> = db.collection("fileInfo");


        println!("[DESTINATION SERVER] DB Connected");
        Database{
            file_info,
        }

    }

    pub async fn store_file_info(&self, file_info: FileInfo) -> Result<InsertOneResult, AppError> {
        self
            .file_info
            .insert_one(file_info)
            .await
            .map_err(|e| AppError::ClientError(e.to_string()))
    }
}