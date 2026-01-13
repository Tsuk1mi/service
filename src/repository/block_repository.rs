use crate::db::DbPool;
use crate::error::AppResult;
use crate::models::block::Block;
use uuid::Uuid;

/// Трейт для работы с блокировками в БД (DIP)
#[async_trait::async_trait]
pub trait BlockRepository: Send + Sync {
    async fn create(
        &self,
        blocker_id: Uuid,
        blocker_plate: &str,
        blocked_plate: &str,
    ) -> AppResult<Block>;
    async fn find_by_blocker_id(&self, blocker_id: Uuid) -> AppResult<Vec<Block>>;
    async fn find_by_blocker_plates(&self, plates: &[String]) -> AppResult<Vec<Block>>;
    async fn find_by_blocked_plate(&self, blocked_plate: &str) -> AppResult<Vec<Block>>;
    async fn delete(&self, block_id: Uuid, blocker_plate: &str) -> AppResult<()>;
    async fn find_by_id(&self, block_id: Uuid) -> AppResult<Option<Block>>;
    /// Проверяет существование блокировки по номерам (оптимизированная проверка дубликатов)
    async fn exists(&self, blocker_plate: &str, blocked_plate: &str) -> AppResult<bool>;
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
    async fn create(
        &self,
        blocker_id: Uuid,
        blocker_plate: &str,
        blocked_plate: &str,
    ) -> AppResult<Block> {
        let block_id = uuid::Uuid::new_v4();

        // Используем RETURNING для избежания дополнительного SELECT
        let block = sqlx::query_as::<_, Block>(
            r#"
            INSERT INTO blocks (id, blocker_id, blocker_plate, blocked_plate, created_at)
            VALUES ($1, $2, $3, $4, NOW())
            RETURNING id, blocker_id, blocker_plate, blocked_plate, created_at
            "#,
        )
        .bind(block_id)
        .bind(blocker_id)
        .bind(blocker_plate)
        .bind(blocked_plate)
        .fetch_one(&*self.db)
        .await?;

        Ok(block)
    }

    async fn find_by_blocker_id(&self, blocker_id: Uuid) -> AppResult<Vec<Block>> {
        let blocks = sqlx::query_as::<_, Block>(
            r#"
            SELECT id, blocker_id, blocker_plate, blocked_plate, created_at
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

    async fn find_by_blocker_plates(&self, plates: &[String]) -> AppResult<Vec<Block>> {
        if plates.is_empty() {
            return Ok(Vec::new());
        }
        let normalized: Vec<String> = plates.iter().map(|p| p.trim().to_uppercase()).collect();
        let blocks = sqlx::query_as::<_, Block>(
            r#"
            SELECT id, blocker_id, blocker_plate, blocked_plate, created_at
            FROM blocks
            WHERE UPPER(TRIM(blocker_plate)) = ANY($1)
            ORDER BY created_at DESC
            "#,
        )
        .bind(&normalized)
        .fetch_all(&*self.db)
        .await?;

        Ok(blocks)
    }

    async fn find_by_blocked_plate(&self, blocked_plate: &str) -> AppResult<Vec<Block>> {
        // Используем нормализованное сравнение для использования индекса
        let blocks = sqlx::query_as::<_, Block>(
            r#"
            SELECT id, blocker_id, blocker_plate, blocked_plate, created_at
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
            SELECT id, blocker_id, blocker_plate, blocked_plate, created_at
            FROM blocks
            WHERE id = $1
            "#,
        )
        .bind(block_id)
        .fetch_optional(&*self.db)
        .await?;

        Ok(block)
    }

    async fn delete(&self, block_id: Uuid, blocker_plate: &str) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM blocks
            WHERE id = $1 AND UPPER(TRIM(blocker_plate)) = UPPER(TRIM($2))
            "#,
        )
        .bind(block_id)
        .bind(blocker_plate)
        .execute(&*self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(crate::error::AppError::NotFound(
                "Block not found or you don't have permission to delete it".to_string(),
            ));
        }

        Ok(())
    }

    async fn exists(&self, blocker_plate: &str, blocked_plate: &str) -> AppResult<bool> {
        // Оптимизированная проверка существования с использованием EXISTS
        // Проверяем по номерам, а не по пользователю
        let exists: (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM blocks
                WHERE UPPER(TRIM(blocker_plate)) = UPPER(TRIM($1))
                AND UPPER(TRIM(blocked_plate)) = UPPER(TRIM($2))
                LIMIT 1
            ) as exists
            "#,
        )
        .bind(blocker_plate)
        .bind(blocked_plate)
        .fetch_one(&*self.db)
        .await?;

        Ok(exists.0)
    }
}
