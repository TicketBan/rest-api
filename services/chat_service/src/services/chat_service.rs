use std::sync::Arc;
use crate::grpc::client::UserGrpcClient;
use crate::models::chat::{Chat, CreateChatDTO};
use crate::repositories::chat_repository::{ChatRepository, PgChatRepository};
use crate::errors::service_error::ServiceError;
use sqlx::PgPool;
use uuid::Uuid;
use futures::future::join_all;

#[derive(Clone)]
pub struct ChatService {
    repository: Arc<dyn ChatRepository>,
    user_client: Arc<UserGrpcClient>,
}

impl ChatService {
    pub fn new(pool: Arc<PgPool>, user_client: Arc<UserGrpcClient>) -> Self {
        Self {
            repository: Arc::new(PgChatRepository::new(pool)),
            user_client,
        }
    }

    pub async fn get_user_chats(&self, user_id: Uuid) -> Result<Vec<Chat>, ServiceError> {
        self.repository.get_user_chats(&user_id).await
    }
    pub async fn get_chat_by_id(&self, chat_id: Uuid) -> Result<Chat, ServiceError> {
        self.repository.get_by_id(&chat_id).await
    }

    pub async fn create(&self, chat_dto: CreateChatDTO) -> Result<Chat, ServiceError> {

        if chat_dto.participants.is_empty() {
            return Err(ServiceError::bad_request("Chat must have at least one participant"));
        }

        let unique_participants: std::collections::HashSet<Uuid> = chat_dto.participants.iter().copied().collect();
        if unique_participants.len() < chat_dto.participants.len() {
            return Err(ServiceError::bad_request("Duplicate participants are not allowed"));
        }

        let user_checks = join_all(unique_participants.iter().map(|&user_id| {
            self.user_client.get_user_by_uid(user_id)
        })).await;

        let mut errors = Vec::new();
        for (idx, result) in user_checks.into_iter().enumerate() {
            if let Err(e) = result {
                let user_id = chat_dto.participants[idx]; 
                errors.push(format!("User {} not found: {}", user_id, e));
            }
        }
        if !errors.is_empty() {
            return Err(ServiceError::not_found(&errors.join("; ")));
        }

        self.repository.create(&chat_dto).await
    }

    pub async fn add_participant(&self, chat_id: Uuid, user_id: Uuid) -> Result<(), ServiceError> {
        self.user_client.get_user_by_uid(user_id)
            .await
            .map_err(|e| ServiceError::not_found(&format!("User {} not found: {}", user_id, e)))?;
        self.repository.add_participant(&chat_id, &user_id).await
    }

    pub async fn remove_participant(&self, chat_id: Uuid, user_id: Uuid) -> Result<(), ServiceError> {
        self.repository.remove_participant(&chat_id, &user_id).await
    }

    pub async fn get_chat_participants(&self, chat_id: Uuid) -> Result<Vec<Uuid>, ServiceError> {
        self.repository.get_chat_participants(&chat_id).await
    }
}