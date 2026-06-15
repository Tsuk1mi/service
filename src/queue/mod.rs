use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationEvent {
    pub event_type: String,
    pub chat_id: Option<i64>,
    pub telegram_username: Option<String>,
    pub push_token: Option<String>,
    pub phone: Option<String>,
    pub message: String,
    pub title: Option<String>,
}

#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish_notification(&self, event: NotificationEvent) -> AppResult<()>;
    async fn publish_sms(&self, phone: &str, code: &str) -> AppResult<()>;
}

/// No-op publisher когда RabbitMQ не настроен
#[derive(Clone)]
pub struct NoopPublisher;

#[async_trait]
impl EventPublisher for NoopPublisher {
    async fn publish_notification(&self, event: NotificationEvent) -> AppResult<()> {
        tracing::debug!("NoopPublisher: notification {:?}", event.event_type);
        Ok(())
    }

    async fn publish_sms(&self, _phone: &str, _code: &str) -> AppResult<()> {
        Ok(())
    }
}

pub mod rabbitmq;

pub use rabbitmq::RabbitMqPublisher;
