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
use crate::grpc::server::start_grpc_server;
use tokio::task;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .init();

    let pool = init_db_pool().await;
    let grpc_pool: sqlx::Pool<sqlx::Postgres> = pool.clone();

    task::spawn(async move {
      let _ = start_grpc_server(grpc_pool).await;
    });


    let host = env::var("USER_SERVICE_HOST").unwrap();
    let port = env::var("USER_SERVICE_PORT").unwrap();
    let server_address = format!("{}:{}", host, port);

    let _ = sqlx::migrate!().run(&pool).await;
    
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Authentication::new(env::var("JWT_SECRET").unwrap()))
            .configure(config_services)
            .app_data(actix_web::web::Data::new(pool.clone()))
    })
    .bind(server_address)?
    .workers(4)
    .run()
    .await
}
