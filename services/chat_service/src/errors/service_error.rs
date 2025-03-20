use actix_web::{HttpResponse, ResponseError};
use derive_more::Display;
use serde::Serialize;

#[derive(Debug, Display, Serialize)]
#[display(fmt = "{}", message)]
pub struct ServiceError {
    pub message: String,
    pub status_code: u16,
}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code().into())
            .json(self)
    }
}

impl ServiceError {
    pub fn new(message: &str, status_code: u16) -> Self {
        Self {
            message: message.to_string(),
            status_code,
        }
    }
    
    pub fn bad_request(message: &str) -> Self {
        Self::new(message, 400)
    }
    
    pub fn not_found(message: &str) -> Self {
        Self::new(message, 404)
    }
    
    pub fn internal_error(message: &str) -> Self {
        Self::new(message, 500)
    }
}