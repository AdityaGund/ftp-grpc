use std::env;
#[derive(Clone, Debug)]
pub struct Database{
    bank_user: Collection<Bank>,
    admin_user: Collection<AdminUser>,
    file_info: Collection<FileInfo>
}
use mongodb::results::{InsertOneResult, UpdateResult, DeleteResult};
use mongodb::{Client, Collection};
use crate::error::AppError;
use crate::models::file_info_model::FileInfo;
use bson::{doc};
use futures::stream::TryStreamExt;


#[derive(Debug, serde::Serialize)]
pub struct BankSummary {
    pub username: String,
    pub ip: String,
}

#[derive(Debug, serde::Serialize)]
pub struct AdminSummary {
    pub username: String,
}

use crate::models::user_model::{AdminUser, Bank};
impl Database {
    
    pub async fn init() -> Self {
        let uri = env::var("MONGO_URI").unwrap().to_string();
        let client = Client::with_uri_str(uri).await.unwrap();

        let db = client.database("admin_db");

        let bank_user: Collection<Bank> = db.collection("bank");
        let admin_user: Collection<AdminUser> = db.collection("admin");
        let file_info: Collection<FileInfo> = db.collection("fileInfo");


        println!("[ADMIN SERVER] DB Connected");
        Database{
            bank_user,
            admin_user,
            file_info
        }

    }

    pub async fn add_admin(&self, admin: AdminUser) -> Result<InsertOneResult, AppError> {
        if let Some(_) = self
            .admin_user
            .find_one(doc! { "username": &admin.username })
            .await
            .map_err(|_| AppError::DatabaseError("Failed to query admin_user".into()))?
        {
            return Err(AppError::DatabaseError("Admin user already exists".into()));
        }
    
        let result = self
            .admin_user
            .insert_one(admin)
            .await
            .map_err(|_| AppError::DatabaseError("Failed to insert admin_user".into()))?;
    
        Ok(result)
    }
    

    pub async fn add_bank(&self, bank: Bank) -> Result<InsertOneResult, AppError> {
        if let Some(_) = self
            .bank_user
            .find_one(doc! { "username": &bank.username })
            .await
            .map_err(|_| AppError::DatabaseError("Failed to query bank_user".into()))?
        {
            return Err(AppError::DatabaseError("Bank user already exists".into()));
        }
    
        let result = self
            .bank_user
            .insert_one(bank)
            .await
            .map_err(|_| AppError::DatabaseError("Failed to insert bank_user".into()))?;
    
        Ok(result)
    }

    pub async fn store_file_info(&self, file_info: FileInfo) -> Result<InsertOneResult, AppError> {
        self
            .file_info
            .insert_one(file_info)
            .await
            .map_err(|e| AppError::ClientError(e.to_string()))
    }
    
    pub async fn get_banks(&self) -> Result<Vec<BankSummary>, AppError> {
        
        let cursor = self
        .bank_user
        .find(doc! {})
        .await
        .map_err(|_| AppError::DatabaseError("Failed to query bank_user".into()))?;

        let banks: Vec<Bank> = cursor
            .try_collect()
            .await
            .map_err(|_| AppError::DatabaseError("Failed to collect banks".into()))?;

        let summaries = banks
            .into_iter()
            .map(|bank| BankSummary {
                username: bank.username,
                ip: bank.ip,
            })
            .collect();

        Ok(summaries)

    }

    pub async fn find_admin_by_username(&self, username: &str) -> Result<Option<AdminUser>, AppError> {
        let admin_opt = self
            .admin_user
            .find_one(doc! { "username": username })
            .await
            .map_err(|_| AppError::DatabaseError("Failed to query admin_user".into()))?;

        Ok(admin_opt)
    }

    pub async fn find_bank_by_username(&self, username: &str) -> Result<Option<Bank>, AppError> {
        let bank_opt = self
            .bank_user
            .find_one(doc! { "username": username })
            .await
            .map_err(|_| AppError::DatabaseError("Failed to query bank_user".into()))?;

        Ok(bank_opt)
    }

    pub async fn update_admin_password(&self, username: &str, new_password: &str) -> Result<UpdateResult, AppError> {
        let result = self
            .admin_user
            .update_one(
                doc! { "username": username },
                doc! { "$set": { "password": new_password } },
            )
            .await
            .map_err(|_| AppError::DatabaseError("Failed to update admin user".into()))?;
        Ok(result)
    }

    pub async fn update_bank(&self, username: &str, new_password: Option<&str>, new_ip: Option<&str>) -> Result<UpdateResult, AppError> {
        let mut update_doc = doc! {};
        if let Some(pw) = new_password {
            update_doc.insert("password", pw);
        }
        if let Some(ip) = new_ip {
            update_doc.insert("ip", ip);
        }
        if update_doc.is_empty() {
            return Err(AppError::ClientError("Nothing to update".into()));
        }
        let result = self
            .bank_user
            .update_one(
                doc! { "username": username },
                doc! { "$set": update_doc },
            )
            .await
            .map_err(|_| AppError::DatabaseError("Failed to update bank user".into()))?;
        Ok(result)
    }

    pub async fn delete_admin(&self, username: &str) -> Result<DeleteResult, AppError> {
        let result = self
            .admin_user
            .delete_one(doc! { "username": username })
            .await
            .map_err(|_| AppError::DatabaseError("Failed to delete admin user".into()))?;
        Ok(result)
    }

    pub async fn delete_bank(&self, username: &str) -> Result<DeleteResult, AppError> {
        let result = self
            .bank_user
            .delete_one(doc! { "username": username })
            .await
            .map_err(|_| AppError::DatabaseError("Failed to delete bank user".into()))?;
        Ok(result)
    }

    pub async fn get_admins(&self) -> Result<Vec<AdminSummary>, AppError> {
        let cursor = self
            .admin_user
            .find(doc! {})
            .await
            .map_err(|_| AppError::DatabaseError("Failed to query admin_user".into()))?;

        let admins: Vec<AdminUser> = cursor
            .try_collect()
            .await
            .map_err(|_| AppError::DatabaseError("Failed to collect admins".into()))?;

        let summaries = admins
            .into_iter()
            .map(|admin| AdminSummary {
                username: admin.username,
            })
            .collect();

        Ok(summaries)
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