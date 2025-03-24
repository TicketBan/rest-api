use actix_web::{web, HttpResponse};
use crate::models::message::CreateMessageDTO;
use crate::models::response::ResponseBody;
use crate::services::message_service::MessageService;
use crate::repositories::chat_repository::PgChatRepository;
use crate::repositories::message_repository::PgMessageRepository;
use crate::errors::service_error::ServiceError;

pub async fn get_chat_messages(
    service: web::Data<MessageService<PgMessageRepository, PgChatRepository>>,
    chat_uid: web::Path<String>
) -> Result<HttpResponse, ServiceError> {
    let messages = service.get_all_messages_by_chat_uid(chat_uid.into_inner()).await?;
    Ok(HttpResponse::Ok().json(ResponseBody::new("Messages successfully retrieved", Some(messages))))
}

pub async fn create_message(
    service: web::Data<MessageService<PgMessageRepository,PgChatRepository>>,
    message_dto: web::Json<CreateMessageDTO>
) -> Result<HttpResponse, ServiceError> {  
    let message = service.create(message_dto.0).await?;
    Ok(HttpResponse::Created().json(ResponseBody::new("Message successfully created", Some(message))))
}

