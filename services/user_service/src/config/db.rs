use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;

pub async  fn init_db_pool() -> PgPool {

    let database_url = env::var("DATABASE_URL_USER_SERVICE")
        .expect("DATABASE_URL_USER_SERVICE must be set");

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool")
}