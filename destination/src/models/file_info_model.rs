use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileInfo {
    pub _id: ObjectId,

    pub name: String,
    pub path: String,
    pub sender_bank_id: String,
    pub receiver_bank_id: String,
    pub message: String,
    pub time_sent_at: String,
    pub time_received_at: String,
}
