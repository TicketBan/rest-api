use actix::{Actor, ActorContext, Addr, AsyncContext, Handler, Message, StreamHandler};
use actix_web_actors::ws;
use crate::repositories::chat_repository::PgChatRepository;
use crate::repositories::message_repository::PgMessageRepository;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use crate::models::message::CreateMessageDTO;

use crate::services::message_service::MessageService;

lazy_static::lazy_static! {
    static ref SESSIONS: Mutex<HashMap<Uuid, Vec<Addr<ChatSession>>>> = Mutex::new(HashMap::new());
}


struct DropLogger(&'static str);

impl Drop for DropLogger {
    fn drop(&mut self) {
        log::info!("Dropping: {}", self.0);
    }
}


pub struct ChatSession {
    chat_uid: Uuid,
    user_uid: Uuid,
    message_service: Arc<MessageService<PgMessageRepository, PgChatRepository>>
}

#[derive(Serialize, Deserialize)]
struct WebSocketMessage {
    event: String,
    data: serde_json::Value,
}

impl ChatSession {
    pub fn new(chat_uid: Uuid, user_uid: Uuid, message_service: Arc<MessageService<PgMessageRepository, PgChatRepository>>) -> Self {
        Self {
            chat_uid,
            user_uid,
            message_service: message_service 
        }
    }
}

impl Actor for ChatSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        let mut sessions = SESSIONS.lock().unwrap();
        sessions
            .entry(self.chat_uid)
            .or_insert_with(Vec::new)
            .push(addr);
        log::info!("User {} joined chat {}", self.user_uid, self.chat_uid);
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> actix::Running {
        let mut sessions = SESSIONS.lock().unwrap();

        if let Some(users) = sessions.get_mut(&self.chat_uid) {
            users.retain(|addr| addr != &ctx.address());

            if users.is_empty() {
                sessions.remove(&self.chat_uid);
            }
        }

        log::info!("User {} left chat {}", self.user_uid, self.chat_uid);
        actix::Running::Stop
    }
}

impl ChatSession {
    fn broadcast_message(&self, ctx: &mut ws::WebsocketContext<Self>, content: &str) {
        self.save_to_message(content);

        let response = serde_json::json!({
            "event": "message",
            "data": {
                "chat_uid": self.chat_uid,
                "user_uid": self.user_uid,
                "content": content,
            }
        });

        let sessions = SESSIONS.lock().unwrap();
        if let Some(users) = sessions.get(&self.chat_uid) {
            for user in users {
                if user != &ctx.address() {
                    user.do_send(ChatMessage(response.to_string()));
                }
            }
        }
    }

    fn save_to_message(&self, content: &str) {
        let chat_uid = self.chat_uid;
        let user_uid = self.user_uid;
        let content = content.to_string();
        let message_service = self.message_service.clone();

    
        let _drop_logger = DropLogger("save_to_message variables");
    
        let message = CreateMessageDTO { 
            chat_uid: chat_uid, 
            user_uid: user_uid,
            content: content.clone(),
        };
    
        tokio::spawn(async move {
            let _inside_logger = DropLogger("inside async task");

            let result = message_service.create(message).await;
            
            match result {
                Ok(_) => log::info!(
                    "Message saved to database: chat_uid={}, user_uid={}, content={}",
                    chat_uid,
                    user_uid,
                    content
                ),
                Err(e) => log::error!("Failed to save message to database: {}", e),
            }
        });
    }
    

    fn process_ws_message(&self, text: &str, ctx: &mut ws::WebsocketContext<Self>) {
        log::info!("Received WebSocket message: {}", text);

        let ws_message: WebSocketMessage = match serde_json::from_str(text) {
            Ok(msg) => msg,
            Err(err) => {
                log::error!("Failed to parse message: {}", err);
                return;
            }
        };

        if ws_message.event == "message" {
            if let Some(content) = ws_message.data.get("content").and_then(|c| c.as_str()) {
                self.broadcast_message(ctx, content);
            }
        }
    }
}


impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ChatSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                self.process_ws_message(&text, ctx);
            }
            Ok(ws::Message::Close(reason)) => {
                log::info!("Closing the connection");
                ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct ChatMessage(String);

impl Handler<ChatMessage> for ChatSession {
    type Result = ();
    fn handle(&mut self, msg: ChatMessage, ctx: &mut Self::Context) {
        log::info!("Sending message to WebSocket: {}", msg.0);
        ctx.text(msg.0);
    }
}
