use std::sync::Arc;
use actix_web::Result;
use sqlx::PgPool;
use uuid::Uuid;
use argon2::{password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, SaltString, PasswordVerifier}, Argon2};
use chrono::{Utc, DateTime};
use crate::models::user::{LoginDTO, User, UserDTO, LoginResponse};
use shared::models::user_token::UserToken;
use crate::repositories::user_repository::{UserRepository, PgUserRepository};
use crate::errors::service_error::ServiceError;
use log::{info, error};
use validator::Validate;

pub struct UserService<T: UserRepository> {
    repository: T,
    jwt_secret: Arc<String>,
}

impl UserService<PgUserRepository> {
    pub fn new(pool: Arc<PgPool>, jwt_secret: Arc<String>) -> Self {
        Self {
            repository: PgUserRepository::new(pool),
            jwt_secret,
        }
    }
}

impl<T: UserRepository> UserService<T> {
    pub async fn get_all(&self) -> Result<Vec<User>, ServiceError> {
        info!("Fetching all users");
        self.repository.get_all().await
    }
    
    pub async fn get_by_id(&self, uid: &str) -> Result<User, ServiceError> {
        let uid = Uuid::parse_str(uid).map_err(|_| ServiceError::bad_request("Invalid UUID"))?;

        self.repository.get_by_id(&uid).await
    }
    
    pub async fn signup(&self, user_dto: UserDTO) -> Result<User, ServiceError> {
        info!("Signing up user with email: {}", user_dto.email);
        user_dto.validate().map_err(|e| {
            let errors = e.to_string();
            ServiceError::bad_request(&errors)
        })?;
        if user_dto.username.is_empty() { return Err(ServiceError::bad_request("Username cannot be empty")); }
        if !user_dto.email.contains('@') { return Err(ServiceError::bad_request("Invalid email")); }
        if user_dto.password.len() < 8 { return Err(ServiceError::bad_request("Password too short")); }

        let password_hash = self.hash_password(&user_dto.password)?;
        let new_user_dto = UserDTO {
            username: user_dto.username,
            email: user_dto.email,
            password: password_hash,
        };

        self.repository.create(&new_user_dto).await.map_err(|e| {
            error!("Signup error: {}", e);
            if e.message.contains("duplicate key") {
                ServiceError::bad_request("Email already exists")
            } else {
                e
            }
        })
    }

    pub async fn login(&self, login_dto: LoginDTO) -> Result<LoginResponse, ServiceError> {
        info!("Login attempt for email: {}", login_dto.email);
        login_dto.validate().map_err(|e| {
            let errors = e.to_string();
            ServiceError::bad_request(&errors)
        })?;

        let user = self.repository.get_by_email(&login_dto.email).await?;
        self.verify_password(&login_dto.password, &user.password_hash)?;
        
        let ttl = chrono::Duration::weeks(1);
        let user_token = UserToken::new(user.uid, ttl);
        let token = user_token.generate_token(&self.jwt_secret).map_err(|e| {
            error!("Token generation failed: {:?}", e);
            ServiceError::internal_error(&format!("Error generating token: {:?}", e))
        })?;
        let expires_at = DateTime::<Utc>::from_timestamp(user_token.exp, 0).unwrap();

        info!("User {} logged in successfully", user.email);
        Ok(LoginResponse { user, token, expires_at })
    }

    fn hash_password(&self, password: &str) -> Result<String, ServiceError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        argon2.hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| ServiceError::internal_error(&format!("Password hashing error: {}", e)))
    }

    fn verify_password(&self, password: &str, hashed_password: &str) -> Result<(), ServiceError> {
        let password_hash = PasswordHash::new(hashed_password)
            .map_err(|e| ServiceError::internal_error(&format!("Error parsing password hash: {}", e)))?;
        Argon2::default().verify_password(password.as_bytes(), &password_hash)
            .map_err(|_| ServiceError::bad_request("Incorrect email or password"))
    }
}