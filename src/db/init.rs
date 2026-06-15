use crate::error::{AppError, AppResult};
use sqlx::PgPool;

/// Применяет SQL-миграции из каталога migrations/
pub async fn run_migrations(pool: &PgPool) -> AppResult<()> {
    tracing::info!("Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Migration failed: {}", e)))?;
    tracing::info!("Database migrations completed");
    Ok(())
}
