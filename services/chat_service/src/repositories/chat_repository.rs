use async_trait::async_trait;
use sqlx::PgPool;
use chrono::Utc;
use uuid::Uuid;
use std::sync::Arc;
use crate::models::chat::{Chat, CreateChatDTO, ChatParticipant};
use crate::errors::service_error::ServiceError;
use reqwest::Client;

#[async_trait]
pub trait ChatRepository {
    async fn get_user_chats(&self, user_uid: &Uuid) -> Result<Vec<Chat>, ServiceError>;
    async fn get_by_id(&self, uid: &Uuid) -> Result<Chat, ServiceError>;
    async fn create(&self, chat_dto: &CreateChatDTO) -> Result<Chat, ServiceError>;
    async fn add_participant(&self, chat_uid: &Uuid, user_uid: &Uuid) -> Result<(), ServiceError>;
    async fn remove_participant(&self, chat_uid: &Uuid, user_uid: &Uuid) -> Result<(), ServiceError>;
    async fn get_chat_participants(&self, chat_uid: &Uuid) -> Result<Vec<Uuid>, ServiceError>;
}

pub struct PgChatRepository {
    pub pool: Arc<PgPool>,
}

impl PgChatRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ChatRepository for PgChatRepository {
    async fn get_user_chats(&self, user_uid: &Uuid) -> Result<Vec<Chat>, ServiceError> {
        let chats = sqlx::query_as::<_, Chat>("
            SELECT c.uid, c.name, c.created_at, c.updated_at
            FROM chats c
            JOIN chat_participants cp ON c.uid = cp.chat_uid
            WHERE cp.user_uid = $1"
        )
        .bind(user_uid)
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| ServiceError::internal_error(&format!("Database error: {}", e)))?;

        Ok(chats) 
    }
    
    async fn get_by_id(&self, uid: &Uuid) -> Result<Chat, ServiceError> {
        let chat = sqlx::query_as::<_, Chat>("
            SELECT * FROM chats WHERE uid = $1"
        )
        .bind(uid)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| ServiceError::internal_error(&format!("Database error: {}", e)))?
        .ok_or_else(|| ServiceError::not_found(&format!("Chat with uid {} not found", uid)))?;

        Ok(chat)
    }
    
    async fn create(&self, chat_dto: &CreateChatDTO) -> Result<Chat, ServiceError> {
        let mut tx = self.pool.begin().await
            .map_err(|e| ServiceError::internal_error(&format!("Transaction error: {}", e)))?;
    
        let chat_uid = Uuid::new_v4();
        let now = Utc::now();
        
        let chat = sqlx::query_as::<_, Chat>("
            INSERT INTO chats (uid, name, created_at, updated_at)
            VALUES ($1, $2, $3, $4)
            RETURNING uid, name, created_at, updated_at"
        )
        .bind(chat_uid)
        .bind(&chat_dto.name)
        .bind(now)
        .bind(now)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| ServiceError::internal_error(&format!("Error creating a chat room: {}", e)))?;

        let client = Client::new();
        let user_service_url = "http://127.0.0.1:8080/api/users";

        for user_uid in &chat_dto.participants {
            let response = client
                .get(format!("{}/{}", user_service_url, user_uid))
                .send()
                .await
                .map_err(|e| ServiceError::internal_error(&format!("User service error: {}", e)))?;

            if !response.status().is_success() {
                return Err(ServiceError::bad_request(&format!("User with uid {} does not exist", user_uid)));
            }
        }
    
        for user_uid in &chat_dto.participants {
            sqlx::query(
                "INSERT INTO chat_participants (chat_uid, user_uid, joined_at)
                 VALUES ($1, $2, $3)"
            )
            .bind(chat_uid)
            .bind(user_uid)
            .bind(now)
            .execute(&mut *tx)
            .await
            .map_err(|e| ServiceError::internal_error(&format!("Error adding participant {}: {}", user_uid, e)))?;
        }
    
        tx.commit().await
            .map_err(|e| ServiceError::internal_error(&format!("Transaction commit error: {}", e)))?;
    
        Ok(chat)
    }

    async fn add_participant(&self, chat_uid: &Uuid, user_uid: &Uuid) -> Result<(), ServiceError> {
        let chat_exists = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM chats WHERE uid = $1) as exists"
        )
        .bind(chat_uid)
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| ServiceError::internal_error(&format!("Database error: {}", e)))?;

        if !chat_exists {
            return Err(ServiceError::not_found(&format!("Chat with uid {} not found", chat_uid)));
        }
        
        let already_participant = sqlx::query_scalar::<_, bool>("
            SELECT EXISTS(
                SELECT 1 FROM chat_participants 
                WHERE chat_uid = $1 AND user_uid = $2
            ) as exists
            "
        )
        .bind(chat_uid)
        .bind(user_uid)   
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| ServiceError::internal_error(&format!("Database error: {}", e)))?;

        if already_participant {
            return Err(ServiceError::bad_request("The user is already a member of the chat room"));
        }

        sqlx::query("
            INSERT INTO chat_participants (chat_uid, user_uid, joined_at)
            VALUES ($1, $2, $3)"
        )
        .bind(chat_uid)
        .bind(user_uid)
        .bind(Utc::now())
        .execute(&*self.pool)
        .await
        .map_err(|e| ServiceError::internal_error(&format!("Error adding a participant: {}", e)))?;

        Ok(())
    }

    async fn remove_participant(&self, chat_uid: &Uuid, user_uid: &Uuid) -> Result<(), ServiceError> {
        let result = sqlx::query("
            DELETE FROM chat_participants
            WHERE chat_uid = $1 AND user_uid = $2"
        )
        .bind(chat_uid)
        .bind(user_uid)
        .execute(&*self.pool)
        .await
        .map_err(|e| ServiceError::internal_error(&format!("Participant deletion error {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(ServiceError::not_found("Participant not found in chat"));
        }

        Ok(())
    }

    async fn get_chat_participants(&self, chat_uid: &Uuid) -> Result<Vec<Uuid>, ServiceError> {
        let participants = sqlx::query_scalar::<_, Uuid>(
            "SELECT user_uid FROM chat_participants WHERE chat_uid = $1"
        )
        .bind(chat_uid)
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| ServiceError::internal_error(&format!("Error fetching participants: {}", e)))?;
    
        Ok(participants)
    }
}