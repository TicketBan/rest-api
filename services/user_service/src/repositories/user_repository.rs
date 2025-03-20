// repositories/user_repository.rs
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::user::{User, UserDTO};
use shared::models::user_token::UserToken;
use crate::errors::service_error::ServiceError;
use std::sync::Arc;

#[async_trait::async_trait]
pub trait UserRepository {
    async fn get_all(&self) -> Result<Vec<User>, ServiceError>;
    async fn get_by_id(&self, uid: &Uuid) -> Result<User, ServiceError>;
    async fn get_by_email(&self, email: &str) -> Result<User, ServiceError>;
    async fn create(&self, user: &UserDTO) -> Result<User, ServiceError>;
    async fn crate_token(&self, token_dto: &UserToken) -> Result<UserToken, ServiceError>;
}

pub struct PgUserRepository {
    pub pool: Arc<PgPool>,
}

impl PgUserRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserRepository for PgUserRepository {
    async fn get_all(&self) -> Result<Vec<User>, ServiceError> {
        let users= sqlx::query_as::<_, User>("SELECT * FROM users")
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| ServiceError::internal_error(&format!("Database error: {}", e)))?;

        Ok(users)
    }
    
    async fn get_by_id(&self, uid: &Uuid) -> Result<User, ServiceError> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE uid = $1")
            .bind(uid)
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| ServiceError::internal_error(&format!("Database error: {}", e)))
            .and_then(|user_opt| {
                user_opt.ok_or_else(|| ServiceError::not_found(&format!("User with uid {} not found", uid)))
            })
    }
    
    async fn get_by_email(&self, email: &str) -> Result<User, ServiceError> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| ServiceError::internal_error(&format!("Database error: {}", e)))
            .and_then(|user_opt| {
                user_opt.ok_or_else(|| ServiceError::not_found(&format!("User with email {} not found", email)))
            })
    }
    
    async fn create(&self, user_dto: &UserDTO) -> Result<User, ServiceError> {        
        sqlx::query_as::<_, User>(
            "INSERT INTO users (username, email, password_hash, created_at, updated_at) 
             VALUES ($1, $2, $3, NOW(), NOW()) 
             RETURNING *"
        )
        .bind(&user_dto.username)
        .bind(&user_dto.email)
        .bind(&user_dto.password)
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| ServiceError::internal_error(&format!("Failed to create user: {}", e)))
    }

    async fn crate_token(&self, user_token: &UserToken) -> Result<UserToken, ServiceError>{
        sqlx::query_as::<_, UserToken>(
            "INSERT INTO users_tokens ( exp, iat, sub)
             VALUE ($1, $2, $3, &4)
             RETURNING *"
        )
        .bind(&user_token.exp)
        .bind(&user_token.iat)
        .bind(&user_token.sub)
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| ServiceError::internal_error(&format!("Failed to create user token: {}", e)))
    }
}