use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// Модель для регистрации пользователя в Telegram боте
#[derive(Debug, Clone, FromRow)]
pub struct TelegramBotUser {
    pub id: Uuid,
    pub phone_hash: String,
    pub chat_id: i64,
    pub telegram_username: Option<String>,
    pub user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Трейт для работы с регистрациями Telegram бота
#[async_trait::async_trait]
pub trait TelegramBotRepository: Send + Sync {
    /// Находит регистрацию по phone_hash
    async fn find_by_phone_hash(&self, phone_hash: &str) -> AppResult<Option<TelegramBotUser>>;

    /// Находит регистрацию по chat_id
    async fn find_by_chat_id(&self, chat_id: i64) -> AppResult<Option<TelegramBotUser>>;

    /// Создает или обновляет регистрацию
    async fn upsert(
        &self,
        phone_hash: &str,
        chat_id: i64,
        telegram_username: Option<&str>,
        user_id: Option<Uuid>,
    ) -> AppResult<TelegramBotUser>;

    /// Обновляет user_id для существующей регистрации
    async fn update_user_id(
        &self,
        phone_hash: &str,
        chat_id: i64,
        user_id: Uuid,
    ) -> AppResult<TelegramBotUser>;

    /// Обновляет telegram_username для существующей регистрации
    async fn update_telegram_username(
        &self,
        phone_hash: &str,
        chat_id: i64,
        telegram_username: Option<&str>,
    ) -> AppResult<Option<TelegramBotUser>>;

    /// Находит временные регистрации (с phone_hash вида "temp_*") для указанного user_id
    async fn find_temp_by_user_id(&self, user_id: Uuid) -> AppResult<Vec<TelegramBotUser>>;

    /// Обновляет phone_hash для существующей регистрации
    async fn update_phone_hash(
        &self,
        old_phone_hash: &str,
        new_phone_hash: &str,
        chat_id: i64,
    ) -> AppResult<Option<TelegramBotUser>>;

    /// Удаляет временные записи для указанного user_id, кроме указанной записи
    async fn delete_temp_except(&self, user_id: Uuid, except_id: Uuid) -> AppResult<()>;

    /// Находит регистрацию по telegram_username
    async fn find_by_telegram_username(
        &self,
        telegram_username: &str,
    ) -> AppResult<Option<TelegramBotUser>>;
}

/// Реализация репозитория для PostgreSQL
#[derive(Clone)]
pub struct PostgresTelegramBotRepository {
    db: DbPool,
}

impl PostgresTelegramBotRepository {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl TelegramBotRepository for PostgresTelegramBotRepository {
    async fn find_by_phone_hash(&self, phone_hash: &str) -> AppResult<Option<TelegramBotUser>> {
        let result = sqlx::query_as::<_, TelegramBotUser>(
            r#"
            SELECT 
                id, phone_hash, chat_id, telegram_username, user_id, created_at, updated_at
            FROM telegram_bot_users
            WHERE phone_hash = $1
            ORDER BY updated_at DESC
            LIMIT 1
            "#,
        )
        .bind(phone_hash)
        .fetch_optional(&*self.db)
        .await?;

        Ok(result)
    }

    async fn find_by_chat_id(&self, chat_id: i64) -> AppResult<Option<TelegramBotUser>> {
        let result = sqlx::query_as::<_, TelegramBotUser>(
            r#"
            SELECT 
                id, phone_hash, chat_id, telegram_username, user_id, created_at, updated_at
            FROM telegram_bot_users
            WHERE chat_id = $1
            ORDER BY updated_at DESC
            LIMIT 1
            "#,
        )
        .bind(chat_id)
        .fetch_optional(&*self.db)
        .await?;

        Ok(result)
    }

    async fn upsert(
        &self,
        phone_hash: &str,
        chat_id: i64,
        telegram_username: Option<&str>,
        user_id: Option<Uuid>,
    ) -> AppResult<TelegramBotUser> {
        let result = sqlx::query_as::<_, TelegramBotUser>(
            r#"
            INSERT INTO telegram_bot_users (phone_hash, chat_id, telegram_username, user_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, NOW(), NOW())
            ON CONFLICT (phone_hash, chat_id)
            DO UPDATE SET
                telegram_username = COALESCE(EXCLUDED.telegram_username, telegram_bot_users.telegram_username),
                user_id = COALESCE(EXCLUDED.user_id, telegram_bot_users.user_id),
                updated_at = NOW()
            RETURNING id, phone_hash, chat_id, telegram_username, user_id, created_at, updated_at
            "#,
        )
        .bind(phone_hash)
        .bind(chat_id)
        .bind(telegram_username)
        .bind(user_id)
        .fetch_one(&*self.db)
        .await?;

        Ok(result)
    }

    async fn update_user_id(
        &self,
        phone_hash: &str,
        chat_id: i64,
        user_id: Uuid,
    ) -> AppResult<TelegramBotUser> {
        let result = sqlx::query_as::<_, TelegramBotUser>(
            r#"
            UPDATE telegram_bot_users
            SET user_id = $1, updated_at = NOW()
            WHERE phone_hash = $2 AND chat_id = $3
            RETURNING id, phone_hash, chat_id, telegram_username, user_id, created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(phone_hash)
        .bind(chat_id)
        .fetch_optional(&*self.db)
        .await?;

        result.ok_or_else(|| {
            AppError::NotFound(format!(
                "Telegram bot registration not found for phone_hash={}, chat_id={}",
                phone_hash, chat_id
            ))
        })
    }

    async fn update_telegram_username(
        &self,
        phone_hash: &str,
        chat_id: i64,
        telegram_username: Option<&str>,
    ) -> AppResult<Option<TelegramBotUser>> {
        let result = sqlx::query_as::<_, TelegramBotUser>(
            r#"
            UPDATE telegram_bot_users
            SET telegram_username = $3,
                updated_at = NOW()
            WHERE phone_hash = $1 AND chat_id = $2
            RETURNING id, phone_hash, chat_id, telegram_username, user_id, created_at, updated_at
            "#,
        )
        .bind(phone_hash)
        .bind(chat_id)
        .bind(telegram_username)
        .fetch_optional(&*self.db)
        .await?;

        Ok(result)
    }

    async fn find_temp_by_user_id(&self, user_id: Uuid) -> AppResult<Vec<TelegramBotUser>> {
        let result = sqlx::query_as::<_, TelegramBotUser>(
            r#"
            SELECT id, phone_hash, chat_id, telegram_username, user_id, created_at, updated_at
            FROM telegram_bot_users
            WHERE user_id = $1 AND phone_hash LIKE 'temp_%'
            ORDER BY updated_at DESC
            LIMIT 5
            "#,
        )
        .bind(user_id)
        .fetch_all(&*self.db)
        .await?;

        Ok(result)
    }

    async fn update_phone_hash(
        &self,
        old_phone_hash: &str,
        new_phone_hash: &str,
        chat_id: i64,
    ) -> AppResult<Option<TelegramBotUser>> {
        // Сначала проверяем, не существует ли уже запись с новым phone_hash и этим chat_id
        let existing = sqlx::query_as::<_, TelegramBotUser>(
            r#"
            SELECT id, phone_hash, chat_id, telegram_username, user_id, created_at, updated_at
            FROM telegram_bot_users
            WHERE phone_hash = $1 AND chat_id = $2
            LIMIT 1
            "#,
        )
        .bind(new_phone_hash)
        .bind(chat_id)
        .fetch_optional(&*self.db)
        .await?;

        if existing.is_some() {
            // Запись уже существует - возвращаем её
            return Ok(existing);
        }

        // Обновляем phone_hash
        let result = sqlx::query_as::<_, TelegramBotUser>(
            r#"
            UPDATE telegram_bot_users
            SET phone_hash = $1, updated_at = NOW()
            WHERE phone_hash = $2 AND chat_id = $3
            RETURNING id, phone_hash, chat_id, telegram_username, user_id, created_at, updated_at
            "#,
        )
        .bind(new_phone_hash)
        .bind(old_phone_hash)
        .bind(chat_id)
        .fetch_optional(&*self.db)
        .await?;

        Ok(result)
    }

    async fn delete_temp_except(&self, user_id: Uuid, except_id: Uuid) -> AppResult<()> {
        sqlx::query(
            r#"
            DELETE FROM telegram_bot_users
            WHERE user_id = $1 AND phone_hash LIKE 'temp_%' AND id != $2
            "#,
        )
        .bind(user_id)
        .bind(except_id)
        .execute(&*self.db)
        .await?;

        Ok(())
    }

    async fn find_by_telegram_username(
        &self,
        telegram_username: &str,
    ) -> AppResult<Option<TelegramBotUser>> {
        let result = sqlx::query_as::<_, TelegramBotUser>(
            r#"
            SELECT id, phone_hash, chat_id, telegram_username, user_id, created_at, updated_at
            FROM telegram_bot_users
            WHERE telegram_username = $1
            ORDER BY updated_at DESC
            LIMIT 1
            "#,
        )
        .bind(telegram_username)
        .fetch_optional(&*self.db)
        .await?;

        Ok(result)
    }
}
