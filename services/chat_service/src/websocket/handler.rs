use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use uuid::Uuid;
use std::sync::Arc;
use crate::websocket::session::ChatSession;
use crate::errors::service_error::ServiceError;
use log;
use crate::services::message_service::MessageService;
use crate::repositories::chat_repository::PgChatRepository;
use crate::repositories::message_repository::PgMessageRepository;

pub async fn chat_ws(
    req: HttpRequest,
    stream: web::Payload,
    path: web::Path<(String, String)>,
    message_service: web::Data<Arc<MessageService<PgMessageRepository, PgChatRepository>>>
) -> Result<HttpResponse, Error> {
    log::info!("Received connection request: {:?}", req);
    
    let (chat_uid, user_uid) = path.into_inner();


    log::info!("Parsing chat_uid: {} and user_uid: {}", chat_uid, user_uid);

     let chat_uid = Uuid::parse_str(&chat_uid)
      .map_err(|_| ServiceError::bad_request("Invalid chat UUID"))?;
     let user_uid = Uuid::parse_str(&user_uid)
         .map_err(|_| ServiceError::bad_request("Invalid user UUID"))?;

    log::info!("UUIDs parsed successfully: chat_uid: {:?}, user_uid: {:?}", chat_uid, user_uid);

    let session = ChatSession::new(
        chat_uid,
        user_uid,
        message_service.get_ref().clone()
    );
    
    match ws::start(session, &req, stream) {
        Ok(resp) => {
            log::info!("WebSocket connection established successfully.");
            Ok(resp)
        }
        Err(e) => {
            log::error!("Failed to start WebSocket: {}", e);
            Err(ServiceError::internal_error(&format!("Failed to start WebSocket: {}", e)).into())
        }
    }
}