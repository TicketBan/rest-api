use log;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::env;

#[derive(Serialize, Deserialize, FromRow, Debug, Clone)]
pub struct UserToken {
    pub exp: i64,
    pub iat: i64,
    pub sub: String,
}

#[derive(Debug)]
pub enum TokenError {
    Creation(String),
    Validation(String),
    Expired,
    Invalid,
}

impl UserToken {
    pub fn new(user_id: Uuid) -> Self {
        let now = Utc::now();
        let expiration = now + Duration::weeks(1);
        
        Self {
            exp: expiration.timestamp(),
            iat: now.timestamp(),
            sub: user_id.to_string(),
        }
    }
    
    pub fn generate_token(&self, secret: &str) -> Result<String, TokenError> {
        let key = encode(
            &Header::default(),
            &self,
            &EncodingKey::from_secret(secret.as_bytes()),
        ).map_err(|e| TokenError::Creation(e.to_string()));
        log::info!("{:?}", key);
        key
    }
    
    pub fn validate_token(token: &str, secret: &str) -> Result<Self, TokenError> {
        let decoded = decode::<UserToken>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => TokenError::Expired,
            _ => TokenError::Invalid,
        })?;
        
        Ok(decoded.claims)
    }
    
    pub fn is_valid(&self) -> bool {
        let now = Utc::now().timestamp();
        self.exp > now
    }
    
    pub fn get_user_id(&self) -> Result<Uuid, uuid::Error> {
        Uuid::parse_str(&self.sub)
    }
}
