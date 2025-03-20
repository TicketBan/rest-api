mod config;
mod controllers;
mod models;
mod repositories;
mod services;
mod errors;
mod websocket;
mod grpc;

use actix_web::{web, App, HttpResponse, HttpServer};
use actix_web::middleware::Logger;
use actix_cors::Cors;
use config::app::config_services;
use config::db::init_db_pool;
use std::env;
use std::sync::Arc;
use env_logger;
use log::LevelFilter;
use shared::middleware::auth::Authentication;
use sqlx;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info) 
        .init();

    let pg_pool = init_db_pool().await;

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8081".to_string());
    let server_address = format!("{}:{}", host, port);

    let _ = sqlx::migrate!().run(&pg_pool).await;


    let pool = Arc::new(pg_pool);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
            
        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(Authentication::new())
            .configure(config_services)
            .app_data(actix_web::web::Data::new(pool.clone()))
    })
    .bind(server_address)?
    .workers(4)
    .run()
    .await
}