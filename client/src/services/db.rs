use std::env;
pub struct Database{
    file_info: Collection<FileInfo>
}

use mongodb::results::InsertOneResult;
use mongodb::{Client, Collection};
use crate::error::AppError;
use bson::{doc};
use futures::stream::TryStreamExt;


#[derive(Debug, serde::Serialize)]
pub struct BankSummary {
    pub username: String,
    pub ip: String,
}


use crate::models::file_info_model::FileInfo;
impl Database {
    
    pub async fn init() -> Self {
        let uri = env::var("MONGO_URI").unwrap().to_string();
        let client = Client::with_uri_str(uri).await.unwrap();

        let db = client.database("client_db");

        let file_info: Collection<FileInfo> = db.collection("fileInfo");


        println!("[CLIENT SERVER] DB Connected");
        Database{
            file_info,
        }

    }
}