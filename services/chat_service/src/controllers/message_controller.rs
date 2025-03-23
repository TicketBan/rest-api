use actix_web::{web, HttpResponse};
use std::sync::Arc;
use crate::models::message::CreateMessageDTO;
use crate::models::response::ResponseBody;
use crate::services::message_service::MessageService;
use crate::errors::service_error::ServiceError;
use sqlx::PgPool;

pub async fn get_chat_messages(
    pool: web::Data<Arc<PgPool>>,
    chat_uid: web::Path<String>
) -> Result<HttpResponse, ServiceError> {
      
    let service = MessageService::new(pool.get_ref().clone());
    let messages = service.get_all_messages_by_chat_uid(&chat_uid).await?;
    Ok(HttpResponse::Ok().json(ResponseBody::new("Messages successfully retrieved", Some(messages))))
}

pub async fn create_message(
    pool: web::Data<Arc<PgPool>>,
    message_dto: web::Json<CreateMessageDTO>
) -> Result<HttpResponse, ServiceError> {
    let service = MessageService::new(pool.get_ref().clone());
    
    let message = service.create(message_dto.0).await?;
    Ok(HttpResponse::Created().json(ResponseBody::new("Message successfully created", Some(message))))
}