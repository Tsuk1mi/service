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

    /// Отправляет сообщение по chat_id (предпочтительный способ)
    pub async fn send_message_to_chat(&self, chat_id: i64, message: &str) -> Result<(), String> {
        let token = match &self.bot_token {
            Some(t) if !t.is_empty() => t,
            _ => {
                tracing::warn!("TELEGRAM_BOT_TOKEN not configured, skipping Telegram notification");
                return Ok(());
            }
        };

        let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
        let response = self
            .client
            .post(&url)
            .json(&json!({
                "chat_id": chat_id,
                "text": message
            }))
            .send()
            .await
            .map_err(|e| format!("Telegram API request failed: {}", e))?;

        if response.status().is_success() {
            tracing::info!("Telegram message sent to chat_id {}", chat_id);
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            tracing::warn!(
                "Telegram API error for chat_id {}: {}",
                chat_id,
                error_text
            );
            Ok(())
        }
    }

    /// Отправляет уведомление о блокировке (chat_id приоритетнее username)
    pub async fn send_block_notification(
        &self,
        chat_id: Option<i64>,
        telegram_username: Option<&str>,
        blocked_plate: &str,
        blocker_name: &str,
    ) -> Result<(), String> {
        let message = format!(
            "🚗 Ваш автомобиль {} заблокирован\n\n\
            👤 Блокирующий: {}\n\n\
            📱 Проверьте приложение для подробностей",
            blocked_plate, blocker_name
        );

        if let Some(cid) = chat_id {
            return self.send_message_to_chat(cid, &message).await;
        }

        if let Some(username) = telegram_username {
            let token = match &self.bot_token {
                Some(t) if !t.is_empty() => t,
                _ => return Ok(()),
            };
            let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
            let clean_username = username.trim_start_matches('@');
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

            if !response.status().is_success() {
                let error_text = response.text().await.unwrap_or_default();
                tracing::warn!(
                    "Telegram API error for @{}: {}",
                    clean_username,
                    error_text
                );
            }
        }

        Ok(())
    }
}
