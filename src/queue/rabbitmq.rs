use async_trait::async_trait;
use lapin::{
    options::{
        BasicPublishOptions, ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions,
    },
    types::{AMQPValue, FieldTable},
    BasicProperties, Connection, ConnectionProperties, ExchangeKind,
};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::error::{AppError, AppResult};
use crate::metrics;
use crate::queue::{EventPublisher, NotificationEvent};

const EXCHANGE: &str = "rimskiy.events";
const DLX_EXCHANGE: &str = "rimskiy.dlx";
const QUEUE_NOTIFICATIONS: &str = "notifications";
const QUEUE_SMS: &str = "notifications.sms";
const QUEUE_DLQ: &str = "notifications.dlq";

fn dlq_args() -> FieldTable {
    let mut args = FieldTable::default();
    args.insert(
        "x-dead-letter-exchange".into(),
        AMQPValue::LongString(DLX_EXCHANGE.into()),
    );
    args.insert(
        "x-dead-letter-routing-key".into(),
        AMQPValue::LongString(QUEUE_DLQ.into()),
    );
    args
}

#[derive(Clone)]
pub struct RabbitMqPublisher {
    channel: Arc<Mutex<lapin::Channel>>,
}

impl RabbitMqPublisher {
    pub async fn connect(url: &str) -> AppResult<Self> {
        let conn = Connection::connect(url, ConnectionProperties::default())
            .await
            .map_err(|e| AppError::Internal(format!("RabbitMQ connect error: {}", e)))?;

        let channel = conn
            .create_channel()
            .await
            .map_err(|e| AppError::Internal(format!("RabbitMQ channel error: {}", e)))?;

        channel
            .exchange_declare(
                EXCHANGE,
                ExchangeKind::Topic,
                ExchangeDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await
            .map_err(|e| AppError::Internal(format!("RabbitMQ exchange error: {}", e)))?;

        channel
            .exchange_declare(
                DLX_EXCHANGE,
                ExchangeKind::Direct,
                ExchangeDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await
            .map_err(|e| AppError::Internal(format!("RabbitMQ DLX error: {}", e)))?;

        channel
            .queue_declare(
                QUEUE_DLQ,
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await
            .map_err(|e| AppError::Internal(format!("RabbitMQ DLQ queue error: {}", e)))?;

        channel
            .queue_bind(
                QUEUE_DLQ,
                DLX_EXCHANGE,
                QUEUE_DLQ,
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|e| AppError::Internal(format!("RabbitMQ DLQ bind error: {}", e)))?;

        for queue in [QUEUE_NOTIFICATIONS, QUEUE_SMS] {
            let queue_args = if queue == QUEUE_NOTIFICATIONS {
                dlq_args()
            } else {
                FieldTable::default()
            };

            channel
                .queue_declare(
                    queue,
                    QueueDeclareOptions {
                        durable: true,
                        ..Default::default()
                    },
                    queue_args,
                )
                .await
                .map_err(|e| AppError::Internal(format!("RabbitMQ queue error: {}", e)))?;

            let routing_key = if queue == QUEUE_SMS {
                "notification.sms"
            } else {
                "notification.#"
            };

            channel
                .queue_bind(
                    queue,
                    EXCHANGE,
                    routing_key,
                    QueueBindOptions::default(),
                    FieldTable::default(),
                )
                .await
                .map_err(|e| AppError::Internal(format!("RabbitMQ bind error: {}", e)))?;
        }

        Ok(Self {
            channel: Arc::new(Mutex::new(channel)),
        })
    }
}

#[async_trait]
impl EventPublisher for RabbitMqPublisher {
    async fn publish_notification(&self, event: NotificationEvent) -> AppResult<()> {
        let routing_key = format!("notification.{}", event.event_type);
        let payload = serde_json::to_vec(&event)
            .map_err(|e| AppError::Internal(format!("Serialize event error: {}", e)))?;

        let channel = self.channel.lock().await;
        channel
            .basic_publish(
                EXCHANGE,
                &routing_key,
                BasicPublishOptions::default(),
                &payload,
                BasicProperties::default().with_delivery_mode(2),
            )
            .await
            .map_err(|e| AppError::Internal(format!("RabbitMQ publish error: {}", e)))?
            .await
            .map_err(|e| AppError::Internal(format!("RabbitMQ publish confirm error: {}", e)))?;

        metrics::record_queue_message(&event.event_type);
        Ok(())
    }

    async fn publish_sms(&self, phone: &str, code: &str) -> AppResult<()> {
        let event = NotificationEvent {
            event_type: "sms".into(),
            chat_id: None,
            telegram_username: None,
            push_token: None,
            phone: Some(phone.to_string()),
            message: format!("Ваш код подтверждения: {}", code),
            title: None,
        };
        self.publish_notification(event).await
    }
}
