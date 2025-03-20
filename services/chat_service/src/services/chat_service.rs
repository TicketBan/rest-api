use std::sync::Arc;
use crate::models::chat::{Chat, CreateChatDTO};
use crate::repositories::chat_repository::{ChatRepository, PgChatRepository};
use crate::errors::service_error::ServiceError;
use sqlx::PgPool;
use uuid::Uuid;

pub struct ChatService<T: ChatRepository> {
    repository: T,
}

impl ChatService<PgChatRepository> {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            repository: PgChatRepository::new(pool),
        }
    } 
}

impl<T: ChatRepository> ChatService<T> {
    pub async fn get_user_chats(&self, user_uid: &str) -> Result<Vec<Chat>, ServiceError> {
        let user_uid = Uuid::parse_str(user_uid)
            .map_err(|e| ServiceError::bad_request(&format!("Invalid UUID format: {}", e)))?;

        self.repository.get_user_chats(&user_uid).await
    }
    
    pub async fn get_by_uid(&self, uid: &str) -> Result<Chat, ServiceError> {
        let uid = Uuid::parse_str(uid)
            .map_err(|e| ServiceError::bad_request(&format!("Invalid UUID format: {}", e)))?;

        self.repository.get_by_id(&uid).await
    }
    
    pub async fn create(&self, chat_dto: CreateChatDTO) -> Result<Chat, ServiceError> {
        if chat_dto.participants.is_empty() {
            return Err(ServiceError::bad_request("The chat must have at least one participant."));
        }

        self.repository.create(&chat_dto).await
    }
    
    pub async fn add_participant(&self, chat_uid: &str, user_uid: &str) -> Result<(), ServiceError> {
        let chat_uid = Uuid::parse_str(chat_uid)
            .map_err(|e| ServiceError::bad_request(&format!("Invalid UUID format: {}", e)))?;

        let user_uid = Uuid::parse_str(user_uid)
            .map_err(|e| ServiceError::bad_request(&format!("Invalid UUID format: {}", e)))?;

        self.repository.add_participant(&chat_uid, &user_uid).await
    }
    
    pub async fn remove_participant(&self, chat_uid: &str, user_uid: &str) -> Result<(), ServiceError> {
        let chat_uid = Uuid::parse_str(chat_uid)
            .map_err(|e| ServiceError::bad_request(&format!("Invalid UUID format: {}", e)))?;

        let user_uid = Uuid::parse_str(user_uid)
            .map_err(|e| ServiceError::bad_request(&format!("Invalid UUID format: {}", e)))?;


        self.repository.remove_participant(&chat_uid, &user_uid).await
    }
    
    pub async fn get_chat_participants(&self, chat_uid: &str) -> Result<Vec<Uuid>, ServiceError> {
        let chat_uid = Uuid::parse_str(chat_uid)
            .map_err(|e| ServiceError::bad_request(&format!("Invalid UUID format: {}", e)))?;

        self.repository.get_chat_participants(&chat_uid).await
    }
}
