use sqlx::{PgPool,Error, postgres::PgPoolOptions};

pub async fn init_db_pool(database_url: &str) -> Result<PgPool, Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
}