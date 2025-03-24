mod config;
mod controllers;
mod models;
mod repositories;
mod services;
mod errors;
mod websocket;
mod grpc;

use actix_web::{web, App, HttpServer};
use actix_web::middleware::Logger;
use actix_cors::Cors;
use config::app::config_services;
use config::db::init_db_pool;
use services::message_service::MessageService;
use std::sync::Arc;
use env_logger;
use shared::middleware::auth::Authentication;
use dotenvy::dotenv;
use config::config::Config;
use log::{info, error};
use services::chat_service::ChatService;
use grpc::client::init_grpc_client;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let config = Config::from_env().map_err(|e| {
        error!("Config error: {}", e);
        std::io::Error::new(std::io::ErrorKind::InvalidInput, e)
    })?;

    env_logger::Builder::new()
    .filter_level(config.log_level) 
    .init();
    info!("Starting chat_service with config: {:?}", config);

    let pool = Arc::new(init_db_pool(&config.database_url).await.map_err(|e| {
        error!("Failed to create DB pool: {}", e);
        std::io::Error::new(std::io::ErrorKind::Other, e)
    })?);

    let grpc_client = match init_grpc_client(config.grpc_addr.clone(), std::time::Duration::from_secs(5)).await {
        Ok(client) => client,
        Err(e) => {
            log::error!("Failed to initialize gRPC client: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()));
        }
    };

    let message_service = Arc::new(MessageService::new(pool.clone()));
    let chat_service = Arc::new(ChatService::new(pool.clone(), grpc_client.clone()));

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
            
        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(Authentication::new(config.jwt_secret.clone(), None))
            .configure(config_services)
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::from(message_service.clone()))
            .app_data(web::Data::from(chat_service.clone()))
    })
    .bind(config.http_addr)?
    .workers(4)
    .run()
    .await
}