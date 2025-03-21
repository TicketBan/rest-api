use std::sync::Arc;
use sqlx::PgPool;
use uuid::Uuid;
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, SaltString, PasswordVerifier
    },
    Argon2
};
use chrono::{Utc, NaiveDateTime};
use crate::models::user::{LoginDTO, User, UserDTO, LoginResponse};
use shared::models::user_token::UserToken;
use crate::repositories::user_repository::{UserRepository, PgUserRepository};
use crate::errors::service_error::ServiceError;


pub struct UserService<T: UserRepository> {
    repository: T,
}

impl UserService<PgUserRepository> {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            repository: PgUserRepository::new(pool),
        }
    }
}

impl<T: UserRepository> UserService<T> {
    pub async fn get_all(&self) -> Result<Vec<User>, ServiceError> {
        self.repository.get_all().await
    }
    
    pub async fn get_by_id(&self, uid: &str) -> Result<User, ServiceError> {
        let uid = Uuid::parse_str(uid)
            .map_err(|_| ServiceError::bad_request("Uuid error"))?;

        self.repository.get_by_id(&uid).await
    }
    
    pub async fn signup(&self, user_dto: UserDTO) -> Result<String, ServiceError> {
        if user_dto.username.is_empty() {
            return Err(ServiceError::bad_request("Username cannot be empty"));
        }
        
        if user_dto.email.is_empty() || !user_dto.email.contains('@') {
            return Err(ServiceError::bad_request("Invalid email format"));
        }
        
        if user_dto.password.len() < 8 {
            return Err(ServiceError::bad_request("Password must be at least 8 characters long"));
        }

        let user = self.repository.get_by_email(&user_dto.email);
        
        if user.await.is_ok() {
            return Err(ServiceError::bad_request(&format!("User with email {} already exists", &user_dto.email)));
        }
    
        let password_hash = self.hash_password(&user_dto.password)?;

        let new_user_dto = UserDTO {
            username: user_dto.username,
            email: user_dto.email,
            password: password_hash,
        };
        
        let user = self.repository.create(&new_user_dto).await?;
        
        Ok(format!("User {} successfully created", user.username))
    }

    pub async fn login(&self, login_dto: LoginDTO) -> Result<LoginResponse, ServiceError>{
        let user = self.repository.get_by_email(&login_dto.email)
            .await?;
        
        self.verify_password(&login_dto.password, &user.password_hash)?;

        let user_token = UserToken::new(user.uid);
        
        let token = user_token.generate_token(env::var("JWT_SECRET"))
              .map_err(|e| ServiceError::internal_error(&format!("Error generating token: {:?}", e)))?;
            
        let expires_at = DateTime::<Utc>::from_timestamp(user_token.exp, 0)
              .expect("Invalid timestamp");

        let response = LoginResponse {
            user: user,
            token,
            expires_at: expires_at
        };

        Ok(response)

    }

    fn verify_password(&self, password: &str, hashed_password: &str) -> Result<(), ServiceError> {
        let password_hash = PasswordHash::new(hashed_password)
            .map_err(|e| ServiceError::internal_error(&format!("Error parsing password hash:: {}", e)))?;

        let argon2 = Argon2::default();
        argon2.verify_password(password.as_bytes(), &password_hash)
            .map_err(|_| ServiceError::bad_request("Incorrect email or password"))?;
        
        Ok(())
    }
    
    fn hash_password(&self, password: &str) -> Result<String, ServiceError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)
            .map_err(|e| ServiceError::internal_error(&format!("Password hashing error: {}", e)))?; 

        Ok(password_hash.to_string())
    }

}