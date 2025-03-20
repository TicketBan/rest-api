use actix_web::{web, HttpResponse};
use std::sync::Arc;
use crate::models::chat::{CreateChatDTO};
use crate::models::response::ResponseBody;
use crate::services::chat_service::ChatService;
use crate::repositories::chat_repository::PgChatRepository;
use crate::errors::service_error::ServiceError;
use sqlx::PgPool;

pub async fn get_user_chats(
    pool: web::Data<Arc<PgPool>>,
    user_uid: web::Path<String>
) -> Result<HttpResponse, ServiceError> {
    let service = ChatService::<PgChatRepository>::new(pool.get_ref().clone());
    
    let chats = service.get_user_chats(&user_uid.into_inner()).await?;
    Ok(HttpResponse::Ok().json(ResponseBody::new("The chats have been successfully received", Some(chats))))
}

pub async fn get_chat_by_id(
    pool: web::Data<Arc<PgPool>>,
    chat_uid: web::Path<String>
) -> Result<HttpResponse, ServiceError> {
    let service = ChatService::<PgChatRepository>::new(pool.get_ref().clone());

    let chat = service.get_by_uid(&chat_uid).await?;
    Ok(HttpResponse::Ok().json(ResponseBody::new("Chat successfully received", Some(chat))))
}

pub async fn create_chat(
    pool: web::Data<Arc<PgPool>>,
    chat_dto: web::Json<CreateChatDTO>
) -> Result<HttpResponse, ServiceError> {
    let service = ChatService::<PgChatRepository>::new(pool.get_ref().clone());
    
    let chat = service.create(chat_dto.0).await?;
    Ok(HttpResponse::Created().json(ResponseBody::new("Chat has been successfully created", Some(chat))))
}

pub async fn add_participant(
    pool: web::Data<Arc<PgPool>>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, ServiceError> {
    let service = ChatService::<PgChatRepository>::new(pool.get_ref().clone());
    let (chat_uid, user_uid) = path.into_inner();
    
    service.add_participant(&chat_uid, &user_uid).await?;
    Ok(HttpResponse::Ok().json(ResponseBody::new("Participant successfully added", None::<()>)))
}

pub async fn remove_participant(
    pool: web::Data<Arc<PgPool>>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, ServiceError> {
    let service = ChatService::<PgChatRepository>::new(pool.get_ref().clone());
    let (chat_uid, user_uid) = path.into_inner();
    
    service.remove_participant(&chat_uid, &user_uid).await?;
    Ok(HttpResponse::Ok().json(ResponseBody::new("Member successfully deleted", None::<()>)))
}

pub async fn get_chat_participants(
    pool: web::Data<Arc<PgPool>>,
    chat_uid: web::Path<String>,
) -> Result<HttpResponse, ServiceError> {
    let service = ChatService::<PgChatRepository>::new(pool.get_ref().clone());
    
    let participants = service.get_chat_participants(&chat_uid).await?;
    Ok(HttpResponse::Ok().json(ResponseBody::new("Chat participants successfully received", Some(participants))))
}
