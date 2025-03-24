use actix_web::{web, HttpResponse};
use crate::models::chat::CreateChatDTO;
use crate::models::response::ResponseBody;
use crate::repositories::chat_repository::PgChatRepository;
use crate::services::chat_service::ChatService;
use crate::errors::service_error::ServiceError;

pub async fn get_user_chats(
    service: web::Data<ChatService<PgChatRepository>>,
    user_uid: web::Path<String>
) -> Result<HttpResponse, ServiceError> {  
    let chats = service.get_user_chats(user_uid.into_inner()).await?;
    Ok(HttpResponse::Ok().json(ResponseBody::new("The chats have been successfully received", Some(chats))))
}

pub async fn get_chat_by_uid(
    service: web::Data<ChatService<PgChatRepository>>,
    chat_uid: web::Path<String>
) -> Result<HttpResponse, ServiceError> {
    let chat = service.get_chat_by_uid(chat_uid.into_inner()).await?;
    Ok(HttpResponse::Ok().json(ResponseBody::new("Chat successfully received", Some(chat))))
}

pub async fn create_chat(
    service: web::Data<ChatService<PgChatRepository>>,
    chat_dto: web::Json<CreateChatDTO>
) -> Result<HttpResponse, ServiceError> {
    let chat = service.create(chat_dto.0).await?;
    Ok(HttpResponse::Created().json(ResponseBody::new("Chat has been successfully created", Some(chat))))
}

pub async fn add_participant(
    service: web::Data<ChatService<PgChatRepository>>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, ServiceError> {
    let (chat_uid, user_uid) = path.into_inner();
    service.add_participant(chat_uid, user_uid).await?;
    Ok(HttpResponse::Ok().json(ResponseBody::new("Participant successfully added", None::<()>)))
}

pub async fn remove_participant(
    service: web::Data<ChatService<PgChatRepository>>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, ServiceError> {
    let (chat_uid, user_uid) = path.into_inner();
    service.remove_participant(chat_uid, user_uid).await?;
    Ok(HttpResponse::Ok().json(ResponseBody::new("Member successfully deleted", None::<()>)))
}

pub async fn get_chat_participants(
    service: web::Data<ChatService<PgChatRepository>>,
    chat_uid: web::Path<String>,
) -> Result<HttpResponse, ServiceError> {
    let participants = service.get_chat_participants(chat_uid.into_inner()).await?;
    Ok(HttpResponse::Ok().json(ResponseBody::new("Chat participants successfully received", Some(participants))))
}
