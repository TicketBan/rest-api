use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub uid: Uuid, 
    pub chat_uid: Uuid,
    pub user_uid: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
} 

#[derive(Debug, Deserialize)]
pub struct CreateMessageDTO {
    pub chat_uid: Uuid,
    pub user_uid: Uuid,
    pub content: String,
}