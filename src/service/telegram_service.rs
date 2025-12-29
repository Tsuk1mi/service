use crate::config::Config;
use reqwest::Client;
use serde_json::json;

/// Сервис для отправки уведомлений через Telegram Bot API
#[derive(Clone)]
pub struct TelegramService {
    bot_token: Option<String>,
    client: Client,
}

impl TelegramService {
    pub fn new(_config: &Config) -> Self {
        let bot_token = std::env::var("TELEGRAM_BOT_TOKEN").ok();
        Self {
            bot_token,
            client: Client::new(),
        }
    }

    /// Отправляет уведомление о блокировке через Telegram
    /// 
    /// Примечание: Для отправки сообщений через Telegram Bot API нужен chat_id пользователя.
    /// Пользователь должен сначала начать диалог с ботом (например, отправив /start).
    /// В текущей реализации используется попытка отправки по username, но это может не работать,
    /// если пользователь не начал диалог с ботом. Для полной реализации нужно хранить chat_id в БД.
    pub async fn send_block_notification(
        &self,
        telegram_username: &str,
        blocked_plate: &str,
        blocker_name: &str,
    ) -> Result<(), String> {
        let token = match &self.bot_token {
            Some(t) if !t.is_empty() => t,
            _ => {
                tracing::warn!("TELEGRAM_BOT_TOKEN not configured, skipping Telegram notification");
                return Ok(());
            }
        };

        // Формируем сообщение
        let message = format!(
            "🚗 Ваш автомобиль {} заблокирован\n\n\
            👤 Блокирующий: {}\n\n\
            📱 Проверьте приложение для подробностей",
            blocked_plate, blocker_name
        );

        // Отправляем через Telegram Bot API
        // Пытаемся отправить по username (работает только если пользователь начал диалог с ботом)
        let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
        let clean_username = telegram_username.trim_start_matches('@');

        let response = self
            .client
            .post(&url)
            .json(&json!({
                "chat_id": format!("@{}", clean_username),
                "text": message
            }))
            .send()
            .await
            .map_err(|e| format!("Telegram API request failed: {}", e))?;

        let status = response.status();
        if status.is_success() {
            tracing::info!("Telegram notification sent to @{}", clean_username);
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            tracing::warn!(
                "Telegram API error for @{}: {} - {}",
                clean_username,
                status,
                error_text
            );
            // Не возвращаем ошибку, так как это не критично
            // Пользователь может не начать диалог с ботом
            Ok(())
        }
    }
}

