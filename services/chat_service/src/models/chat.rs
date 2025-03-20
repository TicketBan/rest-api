use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sqlx::FromRow;

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct Chat {
    pub uid: Uuid,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateChatDTO {
    pub participants: Vec<Uuid>,
    pub name: Option<String>,  
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ChatParticipant {
    pub chat_uid: Uuid,
    pub user_uid: Uuid,
    pub joined_at: DateTime<Utc>,
}