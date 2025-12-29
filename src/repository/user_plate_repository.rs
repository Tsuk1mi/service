use uuid::Uuid;
use crate::db::DbPool;
use crate::error::AppResult;
use crate::models::user_plate::UserPlate;

/// Трейт для работы с автомобилями пользователя (DIP)
#[async_trait::async_trait]
pub trait UserPlateRepository: Send + Sync {
    async fn create(&self, user_id: Uuid, plate: &str, is_primary: bool, departure_time: Option<chrono::NaiveTime>) -> AppResult<UserPlate>;
    async fn find_by_user_id(&self, user_id: Uuid) -> AppResult<Vec<UserPlate>>;
    async fn find_primary_by_user_id(&self, user_id: Uuid) -> AppResult<Option<UserPlate>>;
    async fn find_by_plate(&self, plate: &str) -> AppResult<Vec<UserPlate>>;
    async fn delete(&self, id: Uuid, user_id: Uuid) -> AppResult<()>;
    async fn set_primary(&self, id: Uuid, user_id: Uuid) -> AppResult<()>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<UserPlate>>;
    async fn update_departure_time(&self, id: Uuid, user_id: Uuid, time: Option<chrono::NaiveTime>) -> AppResult<UserPlate>;
}

/// Реализация репозитория автомобилей пользователя
#[derive(Clone)]
pub struct PostgresUserPlateRepository {
    db: DbPool,
}

impl PostgresUserPlateRepository {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl UserPlateRepository for PostgresUserPlateRepository {
    async fn create(&self, user_id: Uuid, plate: &str, is_primary: bool, departure_time: Option<chrono::NaiveTime>) -> AppResult<UserPlate> {
        let plate_id = uuid::Uuid::new_v4();

        // Если это основной автомобиль, убираем флаг is_primary у других автомобилей
        // Используем более эффективный UPDATE с WHERE EXISTS
        if is_primary {
            sqlx::query(
                r#"
                UPDATE user_plates
                SET is_primary = false, updated_at = NOW()
                WHERE user_id = $1 AND is_primary = true
                "#
            )
            .bind(user_id)
            .execute(&*self.db)
            .await?;
        }

        // Используем RETURNING для получения созданной записи
        // Если конфликт, возвращаем существующую запись
        let user_plate = sqlx::query_as::<_, UserPlate>(
            r#"
            INSERT INTO user_plates (id, user_id, plate, is_primary, departure_time, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
            ON CONFLICT (user_id, plate) 
            DO UPDATE SET 
                is_primary = EXCLUDED.is_primary,
                departure_time = COALESCE(EXCLUDED.departure_time, user_plates.departure_time),
                updated_at = NOW()
            RETURNING id, user_id, plate, is_primary, departure_time, created_at, updated_at
            "#,
        )
        .bind(plate_id)
        .bind(user_id)
        .bind(plate)
        .bind(is_primary)
        .bind(departure_time)
        .fetch_one(&*self.db)
        .await?;

        Ok(user_plate)
    }

    async fn find_by_user_id(&self, user_id: Uuid) -> AppResult<Vec<UserPlate>> {
        let plates = sqlx::query_as::<_, UserPlate>(
            r#"
            SELECT id, user_id, plate, is_primary, departure_time, created_at, updated_at
            FROM user_plates
            WHERE user_id = $1
            ORDER BY is_primary DESC, created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&*self.db)
        .await?;

        Ok(plates)
    }

    async fn find_primary_by_user_id(&self, user_id: Uuid) -> AppResult<Option<UserPlate>> {
        let plate = sqlx::query_as::<_, UserPlate>(
            r#"
            SELECT id, user_id, plate, is_primary, departure_time, created_at, updated_at
            FROM user_plates
            WHERE user_id = $1 AND is_primary = true
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&*self.db)
        .await?;

        Ok(plate)
    }

    async fn find_by_plate(&self, plate: &str) -> AppResult<Vec<UserPlate>> {
        // Используем нормализованное сравнение для использования индекса
        let plates = sqlx::query_as::<_, UserPlate>(
            r#"
            SELECT id, user_id, plate, is_primary, departure_time, created_at, updated_at
            FROM user_plates
            WHERE UPPER(TRIM(plate)) = UPPER(TRIM($1))
            "#,
        )
        .bind(plate)
        .fetch_all(&*self.db)
        .await?;

        Ok(plates)
    }

    async fn delete(&self, id: Uuid, user_id: Uuid) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM user_plates
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(id)
        .bind(user_id)
        .execute(&*self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(crate::error::AppError::NotFound(
                "User plate not found or you don't have permission to delete it".to_string(),
            ));
        }

        Ok(())
    }

    async fn set_primary(&self, id: Uuid, user_id: Uuid) -> AppResult<()> {
        // Сначала проверяем, что автомобиль принадлежит пользователю
        let plate = sqlx::query_as::<_, UserPlate>(
            r#"
            SELECT id, user_id, plate, is_primary, departure_time, created_at, updated_at
            FROM user_plates
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&*self.db)
        .await?;

        if plate.is_none() {
            return Err(crate::error::AppError::NotFound(
                "User plate not found".to_string(),
            ));
        }

        // Убираем флаг is_primary у всех автомобилей пользователя
        sqlx::query(
            r#"
            UPDATE user_plates
            SET is_primary = false, updated_at = NOW()
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .execute(&*self.db)
        .await?;

        // Устанавливаем этот автомобиль как основной
        sqlx::query(
            r#"
            UPDATE user_plates
            SET is_primary = true, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&*self.db)
        .await?;

        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<UserPlate>> {
        let plate = sqlx::query_as::<_, UserPlate>(
            r#"
            SELECT id, user_id, plate, is_primary, departure_time, created_at, updated_at
            FROM user_plates
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&*self.db)
        .await?;

        Ok(plate)
    }

    async fn update_departure_time(&self, id: Uuid, user_id: Uuid, time: Option<chrono::NaiveTime>) -> AppResult<UserPlate> {
        let plate = sqlx::query_as::<_, UserPlate>(
            r#"
            UPDATE user_plates
            SET departure_time = $1, updated_at = NOW()
            WHERE id = $2 AND user_id = $3
            RETURNING id, user_id, plate, is_primary, departure_time, created_at, updated_at
            "#
        )
        .bind(time)
        .bind(id)
        .bind(user_id)
        .fetch_optional(&*self.db)
        .await?;

        plate.ok_or_else(|| crate::error::AppError::NotFound("User plate not found".to_string()))
    }
}
