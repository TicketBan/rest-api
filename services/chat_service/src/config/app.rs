use actix_web::web;
use crate::controllers::{message_controller, chat_controller};
use crate::websocket;


pub fn config_services(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(
                web::scope("/chats")
                    .route("", web::post().to(chat_controller::create_chat))
                    .route("/user/{user_id}", web::get().to(chat_controller::get_user_chats))
                    .route("/{id}", web::get().to(chat_controller::get_chat_by_uid))
                    .route("/{id}/participants", web::get().to(chat_controller::get_chat_participants))
                    .route("/{id}/participants/{user_id}", web::post().to(chat_controller::add_participant))
                    .route("/{id}/participants/{user_id}", web::delete().to(chat_controller::remove_participant))
            )
            .service(
                web::scope("/messages")
                    .route("", web::post().to(message_controller::create_message))
                    .route("/chat/{chat_uid}", web::get().to(message_controller::get_chat_messages))
            )
    );

    cfg.service(    
        web::resource("/ws/messages/chat_uid/{chat_uid}/user_uid/{user_uid}")
            .route(web::get().to(websocket::handler::chat_ws))
    );
}