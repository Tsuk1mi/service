use crate::db::DbPool;
use crate::error::AppResult;
use crate::models::block::Block;
use uuid::Uuid;

/// Трейт для работы с блокировками в БД (DIP)
#[async_trait::async_trait]
pub trait BlockRepository: Send + Sync {
    async fn create(&self, blocker_id: Uuid, blocked_plate: &str) -> AppResult<Block>;
    async fn find_by_blocker_id(&self, blocker_id: Uuid) -> AppResult<Vec<Block>>;
    async fn find_by_blocked_plate(&self, blocked_plate: &str) -> AppResult<Vec<Block>>;
    async fn delete(&self, block_id: Uuid, blocker_id: Uuid) -> AppResult<()>;
    async fn find_by_id(&self, block_id: Uuid) -> AppResult<Option<Block>>;
    /// Проверяет существование блокировки (оптимизированная проверка дубликатов)
    async fn exists(&self, blocker_id: Uuid, blocked_plate: &str) -> AppResult<bool>;
}

/// Реализация репозитория блокировок
#[derive(Clone)]
pub struct PostgresBlockRepository {
    db: DbPool,
}

impl PostgresBlockRepository {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl BlockRepository for PostgresBlockRepository {
    async fn create(&self, blocker_id: Uuid, blocked_plate: &str) -> AppResult<Block> {
        let block_id = uuid::Uuid::new_v4();

        // Используем RETURNING для избежания дополнительного SELECT
        let block = sqlx::query_as::<_, Block>(
            r#"
            INSERT INTO blocks (id, blocker_id, blocked_plate, created_at)
            VALUES ($1, $2, $3, NOW())
            RETURNING id, blocker_id, blocked_plate, created_at
            "#,
        )
        .bind(block_id)
        .bind(blocker_id)
        .bind(blocked_plate)
        .fetch_one(&*self.db)
        .await?;

        Ok(block)
    }

    async fn find_by_blocker_id(&self, blocker_id: Uuid) -> AppResult<Vec<Block>> {
        let blocks = sqlx::query_as::<_, Block>(
            r#"
            SELECT id, blocker_id, blocked_plate, created_at
            FROM blocks
            WHERE blocker_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(blocker_id)
        .fetch_all(&*self.db)
        .await?;

        Ok(blocks)
    }

    async fn find_by_blocked_plate(&self, blocked_plate: &str) -> AppResult<Vec<Block>> {
        // Используем нормализованное сравнение для использования индекса
        let blocks = sqlx::query_as::<_, Block>(
            r#"
            SELECT id, blocker_id, blocked_plate, created_at
            FROM blocks
            WHERE UPPER(TRIM(blocked_plate)) = UPPER(TRIM($1))
            ORDER BY created_at DESC
            "#,
        )
        .bind(blocked_plate)
        .fetch_all(&*self.db)
        .await?;

        Ok(blocks)
    }

    async fn find_by_id(&self, block_id: Uuid) -> AppResult<Option<Block>> {
        let block = sqlx::query_as::<_, Block>(
            r#"
            SELECT id, blocker_id, blocked_plate, created_at
            FROM blocks
            WHERE id = $1
            "#,
        )
        .bind(block_id)
        .fetch_optional(&*self.db)
        .await?;

        Ok(block)
    }

    async fn delete(&self, block_id: Uuid, blocker_id: Uuid) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM blocks
            WHERE id = $1 AND blocker_id = $2
            "#,
        )
        .bind(block_id)
        .bind(blocker_id)
        .execute(&*self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(crate::error::AppError::NotFound(
                "Block not found or you don't have permission to delete it".to_string(),
            ));
        }

        Ok(())
    }

    async fn exists(&self, blocker_id: Uuid, blocked_plate: &str) -> AppResult<bool> {
        // Оптимизированная проверка существования с использованием EXISTS
        let exists: (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM blocks
                WHERE blocker_id = $1 
                AND UPPER(TRIM(blocked_plate)) = UPPER(TRIM($2))
                LIMIT 1
            ) as exists
            "#,
        )
        .bind(blocker_id)
        .bind(blocked_plate)
        .fetch_one(&*self.db)
        .await?;

        Ok(exists.0)
    }
}
