use crate::errors::service_error::ServiceError;
use crate::models::message::{CreateMessageDTO, Message};
use crate::repositories::chat_repository::{ChatRepository, PgChatRepository};
use crate::repositories::message_repository::{MessageRepository, PgMessageRepository};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

pub struct MessageService<M: MessageRepository, C: ChatRepository> {
    repository: M,
    chat_repository: C,
}

impl MessageService<PgMessageRepository, PgChatRepository> {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            repository: PgMessageRepository::new(pool.clone()),
            chat_repository: PgChatRepository::new(pool),
        }
    }
}

impl<M: MessageRepository, C: ChatRepository> MessageService<M, C> {
    pub async fn get_all_messages_by_chat_uid(
        &self,
        chat_uid: String,
    ) -> Result<Vec<Message>, ServiceError> {

        let chat_uid = Uuid::parse_str(&chat_uid)
            .map_err(|e| ServiceError::bad_request(&format!("Invalid UUID format: {}", e)))?;

        let _ = self
            .chat_repository
            .get_by_uid(&chat_uid)
            .await?;

        self.repository.get_all_by_chat_uid(&chat_uid).await
    }

    pub async fn create(&self, message_dto: CreateMessageDTO) -> Result<Message, ServiceError> {
        if message_dto.content.trim().is_empty() {
            return Err(ServiceError::bad_request("Message content cannot be empty"));
        }

        if message_dto.content.len() > 5000 {
            return Err(ServiceError::bad_request(
                "Message content too long (maximum 5000 characters)",
            ));
        }

        self.repository.create(&message_dto).await
    }
}
