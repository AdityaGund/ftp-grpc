use std::env;
pub struct Database{
    bank_user: Collection<Bank>,
    admin_user: Collection<AdminUser>
}
use mongodb::results::InsertOneResult;
use mongodb::{Client, Collection};
use crate::error::AppError;
use bson::doc;

use crate::models::user_model::{AdminUser, Bank};
impl Database {
    
    pub async fn init() -> Self {
        let uri = env::var("MONGO_URI").unwrap().to_string();
        let client = Client::with_uri_str(uri).await.unwrap();

        let db = client.database("admin_db");

        let bank_user: Collection<Bank> = db.collection("bank");
        let admin_user: Collection<AdminUser> = db.collection("admin");


        println!("[ADMIN SERVER] DB Connected");
        Database{
            bank_user,
            admin_user
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

}