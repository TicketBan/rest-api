mod config;
mod controllers;
mod errors;
mod models;
mod repositories;
mod services;
mod grpc;

use actix_web::middleware::Logger;
use actix_web::{web ,App, HttpServer};
use config::config::Config;
use config::app::config_services;
use config::db::init_db_pool;
use dotenvy::dotenv;
use env_logger;
use log::{info, error};
use std::sync::Arc;
use shared::middleware::auth::Authentication;
use crate::grpc::server::start_grpc_server;
use services::user_service::UserService;
use actix_cors::Cors;


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

    info!("Starting user_service with config: {:?}", config);

    let pool = Arc::new(init_db_pool(&config.database_url).await.map_err(|e| {
        error!("Failed to create DB pool: {}", e);
        std::io::Error::new(std::io::ErrorKind::Other, e)
    })?);
    
    let service = Arc::new(UserService::new(pool.clone(), config.jwt_secret.clone()));

    let grpc_task = tokio::spawn(start_grpc_server(config.grpc_addr.clone(), service.clone()));
    let http_server = HttpServer::new({
        move || {
            let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
            
            App::new()
                .wrap(cors)
                .wrap(Logger::default())
                .wrap(Authentication::new(config.jwt_secret.clone(), Some(["/api/auth/signup", "/api/auth/login"].into())))
                .configure(config_services)
                .app_data(web::Data::from(service.clone()))
                
        }
    })
    .bind(config.http_addr)?
    .workers(4)
    .run();

    

    tokio::select! {
        res = grpc_task => {
            let result = res.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?; 
            result.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?; 
        },
        res = http_server => res?,
    };

    Ok(())
}