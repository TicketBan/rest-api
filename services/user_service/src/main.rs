mod config;
mod controllers;
mod errors;
mod models;
mod repositories;
mod services;
mod grpc;

use actix_web::middleware::Logger;
use actix_web::{App, HttpServer};
use config::app::config_services;
use config::db::init_db_pool;
use dotenvy::dotenv;
use env_logger;
use log::LevelFilter;
use std::env;
use shared::middleware::auth::Authentication;
// use crate::grpc::server::start_grpc_server;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .init();

    let pool = init_db_pool().await;
    let grpc_pool = pool.clone();

    // task::spawn(async move {
    //     if let Err(e) = start_grpc_server(grpc_pool).await {
    //         eprintln!("gRPC server error: {}", e);
    //     }
    // });

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let server_address = format!("{}:{}", host, port);

    HttpServer::new(move || {
        App::new()
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
