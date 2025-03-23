use actix_web::{web, HttpResponse};
use crate::models::chat::CreateChatDTO;
use crate::models::response::ResponseBody;
use crate::services::chat_service::ChatService;
use crate::errors::service_error::ServiceError;
use uuid::Uuid;

pub async fn get_user_chats(
    service: web::Data<ChatService>,
    user_uid: web::Path<String>
) -> Result<HttpResponse, ServiceError> {  
    let user_uid = Uuid::parse_str(&user_uid)
        .map_err(|e| ServiceError::bad_request(&format!("Invalid user UUID: {}", e)))?;
    let chats = service.get_user_chats(user_uid).await?;
    Ok(HttpResponse::Ok().json(ResponseBody::new("The chats have been successfully received", Some(chats))))
}

pub async fn get_chat_by_id(
    service: web::Data<ChatService>,
    chat_uid: web::Path<String>
) -> Result<HttpResponse, ServiceError> {
    let chat_uid = Uuid::parse_str(&chat_uid)
        .map_err(|e| ServiceError::bad_request(&format!("Invalid user UUID: {}", e)))?;
    let chat = service.get_chat_by_id(chat_uid).await?;
    Ok(HttpResponse::Ok().json(ResponseBody::new("Chat successfully received", Some(chat))))
}

pub async fn create_chat(
    service: web::Data<ChatService>,
    chat_dto: web::Json<CreateChatDTO>
) -> Result<HttpResponse, ServiceError> {
    let chat = service.create(chat_dto.0).await?;
    Ok(HttpResponse::Created().json(ResponseBody::new("Chat has been successfully created", Some(chat))))
}

pub async fn add_participant(
    service: web::Data<ChatService>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, ServiceError> {
    let (chat_uid, user_uid) = path.into_inner();
    let chat_uid = Uuid::parse_str(&chat_uid)
        .map_err(|e| ServiceError::bad_request(&format!("Invalid user UUID: {}", e)))?;
    let user_uid = Uuid::parse_str(&user_uid)
        .map_err(|e| ServiceError::bad_request(&format!("Invalid user UUID: {}", e)))?;
    service.add_participant(chat_uid, user_uid).await?;
    Ok(HttpResponse::Ok().json(ResponseBody::new("Participant successfully added", None::<()>)))
}

pub async fn remove_participant(
    service: web::Data<ChatService>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, ServiceError> {
    let (chat_uid, user_uid) = path.into_inner();
    let chat_uid = Uuid::parse_str(&chat_uid)
        .map_err(|e| ServiceError::bad_request(&format!("Invalid user UUID: {}", e)))?;
    let user_uid = Uuid::parse_str(&user_uid)
        .map_err(|e| ServiceError::bad_request(&format!("Invalid user UUID: {}", e)))?;
    service.remove_participant(chat_uid, user_uid).await?;
    Ok(HttpResponse::Ok().json(ResponseBody::new("Member successfully deleted", None::<()>)))
}

pub async fn get_chat_participants(
    service: web::Data<ChatService>,
    chat_uid: web::Path<String>,
) -> Result<HttpResponse, ServiceError> {
    let chat_uid = Uuid::parse_str(&chat_uid)
        .map_err(|e| ServiceError::bad_request(&format!("Invalid user UUID: {}", e)))?;
    let participants = service.get_chat_participants(chat_uid).await?;
    Ok(HttpResponse::Ok().json(ResponseBody::new("Chat participants successfully received", Some(participants))))
}
