use crate::errors::service_error::ServiceError;
use crate::models::message::{CreateMessageDTO, Message};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait MessageRepository {
    async fn create(&self, create_message_dto: &CreateMessageDTO) -> Result<Message, ServiceError>;
    async fn get_all_by_chat_uid(&self, chat_uid: &Uuid) -> Result<Vec<Message>, ServiceError>;
}

pub struct PgMessageRepository {
    pub pool: PgPool,
}

impl PgMessageRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MessageRepository for PgMessageRepository {
    async fn get_all_by_chat_uid(&self, chat_uid: &Uuid) -> Result<Vec<Message>, ServiceError> {
        sqlx::query_as::<_, Message>(
            "SELECT uid, chat_uid, user_uid, content, created_at 
             FROM messages 
             WHERE chat_uid = $1
             ORDER BY created_at ASC",
        )
        .bind(chat_uid)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ServiceError::internal_error(&format!("Database error: {}", e)))
    }
    async fn create(&self, create_message_dto: &CreateMessageDTO) -> Result<Message, ServiceError> {
        sqlx::query_as::<_, Message>(
            "INSERT INTO messages (chat_uid, user_uid, content)
             VALUES ($1, $2, $3)
             RETURNING uid, chat_uid, user_uid, content, created_at
            ",
        )
        .bind(&create_message_dto.chat_uid)
        .bind(&create_message_dto.user_uid)
        .bind(&create_message_dto.content)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ServiceError::internal_error(&format!("Database error: {}", e)))

    }
}
