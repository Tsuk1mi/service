use sqlx::PgPool;
use std::sync::Arc;

/// Тип-алиас для пула соединений с базой данных
pub type DbPool = Arc<PgPool>;

/// Создает пул соединений с PostgreSQL
pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPool::connect(database_url).await
}
