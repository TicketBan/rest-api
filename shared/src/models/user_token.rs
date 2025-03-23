use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use uuid::Uuid;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, errors::Error as JwtError};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserToken {
   pub exp: i64,
   pub iat: i64,
   pub sub: String,
}

#[derive(Debug)]
pub enum TokenError {
    Creation(JwtError),
    Expired,
    Invalid(JwtError),
}

impl UserToken {
    pub fn new(user_id: Uuid, ttl: Duration) -> Self {
        let now = Utc::now();
        Self {
            exp: (now + ttl).timestamp(),
            iat: now.timestamp(),
            sub: user_id.to_string(),
        }
    }
    
    pub fn generate_token(&self, secret: &str) -> Result<String, TokenError> {
        encode(&Header::default(), self, &EncodingKey::from_secret(secret.as_bytes()))
            .map_err(TokenError::Creation)
    }
    
    pub fn validate_token(token: &str, secret: &str) -> Result<Self, TokenError> {
        let decoded = decode::<Self>(token, &DecodingKey::from_secret(secret.as_bytes()), &Validation::default())
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => TokenError::Expired,
                _ => TokenError::Invalid(e),
            })?;
        Ok(decoded.claims)
    }
    
    pub fn is_valid(&self) -> bool {
        let now = Utc::now().timestamp();
        self.iat <= now && self.exp > now
    }
    
    pub fn get_user_id(&self) -> Result<Uuid, uuid::Error> {
        Uuid::parse_str(&self.sub)
    }
}