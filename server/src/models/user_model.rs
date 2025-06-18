// pub use self::{AdminUser, BankUser};

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bank {
    /// MongoDB automatically generated primary key.
    // #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub _id: ObjectId,

    pub username: String,
    pub password: String,
    pub ip: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminUser {
    // #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub _id: ObjectId,

    pub username: String,
    pub password: String,

    // You can extend this with additional privilege flags, etc.
    // #[serde(default)]
    // pub permissions: Vec<String>,
}
