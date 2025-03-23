use sqlx::PgPool;
use uuid::Uuid;
use crate::models::user::{User, UserDTO};
use crate::errors::service_error::ServiceError;
use std::sync::Arc;
use log::{info, error};

#[async_trait::async_trait]
pub trait UserRepository {
    async fn get_all(&self) -> Result<Vec<User>, ServiceError>;
    async fn get_by_id(&self, uid: &Uuid) -> Result<User, ServiceError>;
    async fn get_by_email(&self, email: &str) -> Result<User, ServiceError>;
    async fn create(&self, user: &UserDTO) -> Result<User, ServiceError>;
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
        info!("Executing get_all query");
        sqlx::query_as::<_, User>("SELECT * FROM users")
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| {
                error!("Database error in get_all: {}", e);
                ServiceError::internal_error(&format!("Database error: {}", e))
            })
    }
    
    async fn get_by_id(&self, uid: &Uuid) -> Result<User, ServiceError> {
        info!("Fetching user by ID: {}", uid);
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE uid = $1")
            .bind(uid)
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| {
                error!("Database error in get_by_id: {}", e);
                ServiceError::internal_error(&format!("Database error: {}", e))
            })?
            .ok_or_else(|| ServiceError::not_found(&format!("User with uid {} not found", uid)))
    }
    
    async fn get_by_email(&self, email: &str) -> Result<User, ServiceError> {
        info!("Fetching user by email: {}", email);
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| {
                error!("Database error in get_by_email: {}", e);
                ServiceError::internal_error(&format!("Database error: {}", e))
            })?
            .ok_or_else(|| ServiceError::not_found(&format!("User with email {} not found", email)))
    }

    async fn create(&self, user_dto: &UserDTO) -> Result<User, ServiceError> {      
        info!("Creating user with email: {}", user_dto.email);  
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

}