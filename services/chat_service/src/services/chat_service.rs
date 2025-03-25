use std::sync::Arc;
use crate::grpc::client::UserGrpcClient;
use crate::models::chat::{Chat, CreateChatDTO};
use crate::repositories::chat_repository::{ChatRepository, PgChatRepository};
use crate::errors::service_error::ServiceError;
use sqlx::PgPool;
use uuid::Uuid;
use futures::future::join_all;


pub struct ChatService<T: ChatRepository> {
    repository: T,
    user_client: Arc<UserGrpcClient>,
}

impl ChatService<PgChatRepository> {
    pub fn new(pool: PgPool, user_client: Arc<UserGrpcClient>) -> Self {
        Self {
            repository: PgChatRepository::new(pool),
            user_client,
        }
    }
}

impl<T: ChatRepository> ChatService<T> {

    pub async fn get_user_chats(&self, user_uid: String) -> Result<Vec<Chat>, ServiceError> {
        let user_uid = parse_uuid(&user_uid)?;
        self.repository.get_user_chats(&user_uid).await
    }
    pub async fn get_chat_by_uid(&self, chat_uid: String) -> Result<Chat, ServiceError> {
        let chat_uid = parse_uuid(&chat_uid)?;
        self.repository.get_by_uid(&chat_uid).await
    }

    pub async fn create(&self, chat_dto: CreateChatDTO) -> Result<Chat, ServiceError> {

        if chat_dto.participants.is_empty() {
            return Err(ServiceError::bad_request("Chat must have at least one participant"));
        }

        let unique_participants: std::collections::HashSet<Uuid> = chat_dto.participants.iter().copied().collect();
        if unique_participants.len() < chat_dto.participants.len() {
            return Err(ServiceError::bad_request("Duplicate participants are not allowed"));
        }

        let user_checks = join_all(unique_participants.iter().map(|&user_uid| {
            self.user_client.get_user_by_uid(user_uid)
        })).await;

        let mut errors = Vec::new();
        for (idx, result) in user_checks.into_iter().enumerate() {
            if let Err(e) = result {
                let user_uid = chat_dto.participants[idx]; 
                errors.push(format!("User {} not found: {}", user_uid, e));
            }
        }
        if !errors.is_empty() {
            return Err(ServiceError::not_found(&errors.join("; ")));
        }

        self.repository.create(&chat_dto).await
    }

    pub async fn add_participant(&self, chat_uid: String, user_uid: String) -> Result<(), ServiceError> {
        let user_uid = parse_uuid(&user_uid)?;
        let chat_uid = parse_uuid(&chat_uid)?;

        self.user_client.get_user_by_uid(user_uid)
            .await
            .map_err(|e| ServiceError::not_found(&format!("User {} not found: {}", user_uid, e)))?;
        self.repository.add_participant(&chat_uid, &user_uid).await
    }

    pub async fn remove_participant(&self, chat_uid: String, user_uid: String) -> Result<(), ServiceError> {
        let chat_uid = parse_uuid(&chat_uid)?;
        let user_uid = parse_uuid(&user_uid)?;
        
        self.repository.remove_participant(&chat_uid, &user_uid).await
    }

    pub async fn get_chat_participants(&self, chat_uid: String) -> Result<Vec<Uuid>, ServiceError> {
        let chat_uid = parse_uuid(&chat_uid)?;
        self.repository.get_chat_participants(&chat_uid).await
    }

}


fn parse_uuid(uuid_str: &str) -> Result<Uuid, ServiceError> {
    Uuid::parse_str(uuid_str)
        .map_err(|e| ServiceError::bad_request(&format!("Invalid UUID: {}", e)))
}