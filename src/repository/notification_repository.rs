use crate::db::DbPool;
use crate::error::AppResult;
use crate::models::notification::Notification;
use uuid::Uuid;

/// Трейт для работы с уведомлениями в БД
#[async_trait::async_trait]
pub trait NotificationRepository: Send + Sync {
    async fn create(&self, notification: &CreateNotificationData) -> AppResult<Notification>;
    async fn find_by_user_id(
        &self,
        user_id: Uuid,
        unread_only: bool,
    ) -> AppResult<Vec<Notification>>;
    async fn mark_as_read(&self, notification_id: Uuid, user_id: Uuid) -> AppResult<()>;
    async fn mark_all_as_read(&self, user_id: Uuid) -> AppResult<()>;
}

pub struct CreateNotificationData {
    pub user_id: Uuid,
    pub r#type: String,
    pub title: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// Реализация репозитория уведомлений
#[derive(Clone)]
pub struct PostgresNotificationRepository {
    db: DbPool,
}

impl PostgresNotificationRepository {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl NotificationRepository for PostgresNotificationRepository {
    async fn create(&self, notification: &CreateNotificationData) -> AppResult<Notification> {
        let notification_id = uuid::Uuid::new_v4();

        // Используем RETURNING для избежания дополнительного SELECT
        let notification = sqlx::query_as::<_, Notification>(
            r#"
            INSERT INTO notifications (id, user_id, type, title, message, data, read, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, false, NOW())
            RETURNING id, user_id, type, title, message, data, read, created_at
            "#,
        )
        .bind(notification_id)
        .bind(notification.user_id)
        .bind(&notification.r#type)
        .bind(&notification.title)
        .bind(&notification.message)
        .bind(&notification.data)
        .fetch_one(&*self.db)
        .await?;

        Ok(notification)
    }

    async fn find_by_user_id(
        &self,
        user_id: Uuid,
        unread_only: bool,
    ) -> AppResult<Vec<Notification>> {
        // Оптимизированный запрос с использованием составного индекса
        let query = if unread_only {
            r#"
            SELECT id, user_id, type, title, message, data, read, created_at
            FROM notifications
            WHERE user_id = $1 AND read = false
            ORDER BY created_at DESC
            LIMIT 100
            "#
        } else {
            r#"
            SELECT id, user_id, type, title, message, data, read, created_at
            FROM notifications
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT 100
            "#
        };

        let notifications = sqlx::query_as::<_, Notification>(query)
            .bind(user_id)
            .fetch_all(&*self.db)
            .await?;

        Ok(notifications)
    }

    async fn mark_as_read(&self, notification_id: Uuid, user_id: Uuid) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE notifications
            SET read = true
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(notification_id)
        .bind(user_id)
        .execute(&*self.db)
        .await?;

        Ok(())
    }

    async fn mark_all_as_read(&self, user_id: Uuid) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE notifications
            SET read = true
            WHERE user_id = $1 AND read = false
            "#,
        )
        .bind(user_id)
        .execute(&*self.db)
        .await?;

        Ok(())
    }
}
