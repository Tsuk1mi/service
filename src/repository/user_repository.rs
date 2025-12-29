use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::user::User;
use uuid::Uuid;

/// Трейт для работы с пользователями в БД (DIP - Dependency Inversion Principle)
#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_phone_hash(&self, phone_hash: &str) -> AppResult<Option<User>>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<User>>;
    async fn create(&self, user: &CreateUserData) -> AppResult<User>;
    async fn update(&self, id: Uuid, update_data: &UpdateUserData) -> AppResult<User>;
    async fn get_plate_by_id(&self, id: Uuid) -> AppResult<Option<String>>;
}

pub struct CreateUserData {
    pub id: Uuid,
    pub phone_encrypted: String,
    pub phone_hash: String,
    pub plate: String,
}

pub struct UpdateUserData {
    pub name: Option<String>,
    pub phone_encrypted: Option<String>,
    pub phone_hash: Option<String>,
    pub telegram: Option<String>,
    pub plate: Option<String>,
    pub show_contacts: Option<bool>,
    pub owner_type: Option<String>,
    pub owner_info: Option<serde_json::Value>,
    pub departure_time: Option<chrono::NaiveTime>,
    pub push_token: Option<String>,
}

/// Реализация репозитория пользователей
#[derive(Clone)]
pub struct PostgresUserRepository {
    db: DbPool,
}

impl PostgresUserRepository {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_phone_hash(&self, phone_hash: &str) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT 
                id, phone_encrypted, phone_hash, telegram, plate, name, show_contacts, 
                owner_type, owner_info, departure_time, push_token, created_at, updated_at
            FROM users
            WHERE phone_hash = $1
            LIMIT 1
            "#,
        )
        .bind(phone_hash)
        .fetch_optional(&*self.db)
        .await?;

        Ok(user)
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<User>> {
        tracing::debug!("find_by_id called for user_id: {}", id);
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, phone_encrypted, phone_hash, telegram, plate, name, show_contacts, owner_type, owner_info, departure_time, push_token, created_at, updated_at
            FROM users
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&*self.db)
        .await?;

        if user.is_none() {
            tracing::warn!("User not found in database: {}", id);
        } else {
            tracing::debug!(
                "User found: {} (plate: {:?})",
                id,
                user.as_ref().unwrap().plate.as_ref()
            );
        }

        Ok(user)
    }

    async fn create(&self, data: &CreateUserData) -> AppResult<User> {
        // Пустая строка для plate сохраняется как NULL
        let plate_value: Option<String> = if data.plate.trim().is_empty() {
            None
        } else {
            Some(data.plate.clone())
        };

        // Используем RETURNING для избежания дополнительного SELECT
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, phone_encrypted, phone_hash, plate, show_contacts, owner_type, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, 'renter', NOW(), NOW())
            RETURNING id, phone_encrypted, phone_hash, telegram, plate, name, show_contacts, 
                      owner_type, owner_info, departure_time, push_token, created_at, updated_at
            "#
        )
        .bind(data.id)
        .bind(&data.phone_encrypted)
        .bind(&data.phone_hash)
        .bind(plate_value.as_ref())
        .bind(true) // Контакты открыты по умолчанию
        .fetch_one(&*self.db)
        .await?;

        Ok(user)
    }

    async fn update(&self, id: Uuid, update_data: &UpdateUserData) -> AppResult<User> {
        // Сначала получаем текущего пользователя
        let current_user = self
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Объединяем данные: если значение передано, используем его, иначе оставляем текущее
        // Для полей, которые могут быть явно null (пустые строки), обновляем их
        let name = if let Some(ref name) = update_data.name {
            if name.is_empty() {
                None
            } else {
                Some(name.clone())
            }
        } else {
            current_user.name
        };

        let telegram = if let Some(ref telegram) = update_data.telegram {
            if telegram.is_empty() {
                None
            } else {
                Some(telegram.clone())
            }
        } else {
            current_user.telegram
        };

        let plate = update_data
            .plate
            .clone()
            .or_else(|| current_user.plate.clone());
        let show_contacts = update_data
            .show_contacts
            .unwrap_or(current_user.show_contacts);
        let phone_encrypted = update_data
            .phone_encrypted
            .clone()
            .or(current_user.phone_encrypted.clone());

        // owner_type: если передано, используем, иначе оставляем текущее значение
        // НЕ используем дефолтное значение, чтобы не перезаписывать существующие данные
        let owner_type = if let Some(ref owner_type) = update_data.owner_type {
            owner_type.clone()
        } else if let Some(ref current_owner_type) = current_user.owner_type {
            current_owner_type.clone()
        } else {
            // Только если вообще нет значения, устанавливаем дефолт
            "renter".to_string()
        };

        // Определяем owner_info:
        // 1. Если owner_type = "owner" - всегда null (очищаем)
        // 2. Если передан явно в update_data.owner_info - используем его
        // 3. Иначе сохраняем текущее значение (не изменяем)
        let owner_info_value = if owner_type == "owner" {
            // Для owner всегда очищаем owner_info
            None
        } else if update_data.owner_info.is_some() {
            // Если передано явно (даже пустой объект) - используем его
            update_data.owner_info.clone()
        } else {
            // Иначе сохраняем текущее значение owner_info
            current_user.owner_info.clone()
        };

        tracing::info!(
            "Updating user {}: owner_type={}, owner_info={:?}",
            id,
            owner_type,
            owner_info_value.is_some()
        );

        // ВСЕГДА обновляем все поля, включая owner_info и departure_time
        let departure_time = update_data
            .departure_time
            .clone()
            .or_else(|| current_user.departure_time.clone());

        // Используем RETURNING для избежания дополнительного SELECT
        let phone_hash = update_data
            .phone_hash
            .clone()
            .or(current_user.phone_hash.clone());

        let updated_user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET name = $1, 
                telegram = $2, 
                plate = $3, 
                show_contacts = $4, 
                phone_encrypted = COALESCE($5, phone_encrypted), 
                phone_hash = COALESCE($10, phone_hash),
                owner_type = $6, 
                owner_info = $7,
                departure_time = $8,
                push_token = COALESCE($9, push_token),
                updated_at = NOW()
            WHERE id = $11
            RETURNING id, phone_encrypted, phone_hash, telegram, plate, name, show_contacts, 
                      owner_type, owner_info, departure_time, push_token, created_at, updated_at
            "#,
        )
        .bind(name.as_ref())
        .bind(telegram.as_ref())
        .bind(&plate)
        .bind(show_contacts)
        .bind(phone_encrypted.as_ref())
        .bind(&owner_type)
        .bind(owner_info_value.as_ref())
        .bind(departure_time.as_ref())
        .bind(update_data.push_token.as_ref())
        .bind(phone_hash.as_ref())
        .bind(id)
        .fetch_optional(&*self.db)
        .await
        .map_err(|e| {
            tracing::error!("Database update error: {:?}", e);
            e
        })?;

        updated_user.ok_or_else(|| AppError::NotFound("User not found".to_string()))
    }

    async fn get_plate_by_id(&self, id: Uuid) -> AppResult<Option<String>> {
        let result: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT plate FROM users WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&*self.db)
        .await?;

        Ok(result.map(|r| r.0))
    }
}
