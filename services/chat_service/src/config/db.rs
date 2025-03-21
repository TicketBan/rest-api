use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;
use dotenvy::dotenv;

pub async  fn init_db_pool() -> PgPool {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL_CHAT_SERVICE").unwrap()
        .expect("DATABASE_URL_CHAT_SERVICE must be set");

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool")
}