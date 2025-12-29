use sqlx::PgPool;
use std::sync::Arc;

pub type DbPool = Arc<PgPool>;

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPool::connect(database_url).await
}

