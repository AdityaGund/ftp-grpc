use std::env;
pub struct Database{
    file_info: Collection<FileInfo>
}

use mongodb::{Client, Collection};
use bson::{doc};
use std::path::Path;
use dotenv::dotenv;
use futures::stream::TryStreamExt;
use crate::error::AppError;


#[derive(Debug, serde::Serialize)]
pub struct BankSummary {
    pub username: String,
    pub ip: String,
}


use crate::models::file_info_model::FileInfo;
impl Database {
    
    // client never actually connects to DB - as of 19th june
    pub async fn init() -> Self {
        let client_env_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(".env");
        if dotenv::from_path(client_env_path.as_path()).is_err() {
            dotenv().ok();
        }
        let uri = env::var("MONGO_URI").unwrap().to_string();
        let client = Client::with_uri_str(uri).await.unwrap();

        let db = client.database("client_db");

        let file_info: Collection<FileInfo> = db.collection("fileInfo");


        println!("[CLIENT SERVER] DB Connected");
        Database{
            file_info,
        }
    }

    pub async fn get_file_info(&self) -> Result<Vec<FileInfo>, AppError> {
        let cursor = self
            .file_info
            .find(doc! {})
            .await
            .map_err(|_| AppError::DatabaseError("Failed to query file info".into()))?;

        let files: Vec<FileInfo> = cursor
            .try_collect()
            .await
            .map_err(|_| AppError::DatabaseError("Failed to collect file info".into()))?;

        Ok(files)
    }
}