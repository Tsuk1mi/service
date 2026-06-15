use anyhow::Context;
use lapin::{
    options::{
        BasicAckOptions, BasicConsumeOptions, BasicNackOptions, BasicPublishOptions,
        QueueDeclareOptions,
    },
    types::{AMQPValue, FieldTable},
    BasicProperties, Channel, Connection, ConnectionProperties,
};
use rimskiy_service::config::Config;
use rimskiy_service::queue::NotificationEvent;
use rimskiy_service::service::push_service::PushService;
use rimskiy_service::service::telegram_service::TelegramService;
use std::sync::Arc;
use std::time::Duration;

const QUEUE_NOTIFICATIONS: &str = "notifications";
const QUEUE_SMS: &str = "notifications.sms";
const QUEUE_DLQ: &str = "notifications.dlq";
const MAX_RETRIES: u32 = 3;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = Config::from_env()?;
    let rabbitmq_url = config
        .rabbitmq_url
        .as_deref()
        .context("RABBITMQ_URL is required for notification_worker")?;

    let conn = Connection::connect(rabbitmq_url, ConnectionProperties::default()).await?;
    let channel = Arc::new(conn.create_channel().await?);

    for queue in [QUEUE_NOTIFICATIONS, QUEUE_SMS, QUEUE_DLQ] {
        channel
            .queue_declare(
                queue,
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await?;
    }

    let telegram_service = Arc::new(TelegramService::new(&config));
    let push_service = Arc::new(PushService::new(config.fcm_server_key.clone()));

    tracing::info!(
        "Notification worker consuming {} and {}",
        QUEUE_NOTIFICATIONS,
        QUEUE_SMS
    );

    let mut handles = vec![];
    for queue in [QUEUE_NOTIFICATIONS, QUEUE_SMS] {
        let ch = channel.clone();
        let tg = telegram_service.clone();
        let push = push_service.clone();
        handles.push(tokio::spawn(async move {
            consume_queue(ch, queue, tg, push).await
        }));
    }

    for handle in handles {
        handle.await??;
    }

    Ok(())
}

async fn consume_queue(
    channel: Arc<Channel>,
    queue: &str,
    telegram_service: Arc<TelegramService>,
    push_service: Arc<PushService>,
) -> anyhow::Result<()> {
    let mut consumer = channel
        .basic_consume(
            queue,
            &format!("notification_worker_{queue}"),
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    use futures_lite::stream::StreamExt;

    while let Some(delivery) = consumer.next().await {
        let delivery = match delivery {
            Ok(d) => d,
            Err(e) => {
                tracing::error!("Consumer error on {}: {}", queue, e);
                continue;
            }
        };

        let retry_count = delivery
            .properties
            .headers()
            .as_ref()
            .and_then(|h| h.inner().get("x-retry-count").cloned())
            .and_then(|v| match v {
                AMQPValue::LongInt(n) => Some(n as u32),
                AMQPValue::ShortInt(n) => Some(n as u32),
                _ => None,
            })
            .unwrap_or(0);

        let event: NotificationEvent = match serde_json::from_slice(&delivery.data) {
            Ok(e) => e,
            Err(e) => {
                tracing::error!("Parse error: {}", e);
                let _ = delivery.ack(BasicAckOptions::default()).await;
                continue;
            }
        };

        let success = process_event(&telegram_service, &push_service, &event).await;

        if success {
            let _ = delivery.ack(BasicAckOptions::default()).await;
        } else if retry_count + 1 >= MAX_RETRIES {
            tracing::error!(
                "Moving event {:?} to DLQ after {} retries",
                event.event_type,
                retry_count
            );
            let _ = channel
                .basic_publish(
                    "",
                    QUEUE_DLQ,
                    BasicPublishOptions::default(),
                    &delivery.data,
                    BasicProperties::default().with_delivery_mode(2),
                )
                .await;
            let _ = delivery.ack(BasicAckOptions::default()).await;
        } else {
            let delay_ms = 500u64.saturating_mul(1u64 << retry_count.min(4));
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;

            let mut headers = FieldTable::default();
            headers.insert(
                "x-retry-count".into(),
                AMQPValue::LongInt((retry_count + 1) as i32),
            );
            let _ = channel
                .basic_publish(
                    "",
                    queue,
                    BasicPublishOptions::default(),
                    &delivery.data,
                    BasicProperties::default()
                        .with_delivery_mode(2)
                        .with_headers(headers.into()),
                )
                .await;
            let _ = delivery
                .nack(BasicNackOptions {
                    requeue: false,
                    ..Default::default()
                })
                .await;
        }
    }

    Ok(())
}

async fn process_event(
    telegram_service: &TelegramService,
    push_service: &PushService,
    event: &NotificationEvent,
) -> bool {
    match event.event_type.as_str() {
        "telegram" | "block" => {
            if let Some(chat_id) = event.chat_id {
                telegram_service
                    .send_message_to_chat(chat_id, &event.message)
                    .await
                    .is_ok()
            } else {
                tracing::warn!("Telegram event without chat_id");
                false
            }
        }
        "push" => {
            if let Some(ref token) = event.push_token {
                let title = event.title.as_deref().unwrap_or("Уведомление");
                push_service
                    .send_fcm(token, title, &event.message, serde_json::json!({}))
                    .await
                    .is_ok()
            } else {
                false
            }
        }
        "sms" => {
            tracing::info!("SMS to {:?}: {}", event.phone, event.message);
            true
        }
        other => {
            tracing::warn!("Unknown event: {}", other);
            true
        }
    }
}
