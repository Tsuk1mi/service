use crate::config::Config;
use reqwest::Client;

/// Сервис для звонков через API телефонии
#[derive(Clone)]
pub struct TelephonyService {
    #[allow(dead_code)] // Поле может использоваться в будущем для конфигурации
    config: Config,
    client: Client,
}

impl TelephonyService {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    /// Совершает звонок и проигрывает сообщение
    pub async fn call_owner(&self, phone: &str, message: &str) -> Result<(), String> {
        // Проверяем, есть ли настройки API телефонии
        let telephony_api_url = std::env::var("TELEPHONY_API_URL").ok();
        let telephony_api_key = std::env::var("TELEPHONY_API_KEY").ok();
        
        if telephony_api_url.is_none() || telephony_api_key.is_none() {
            tracing::warn!("Telephony provider not configured. Set TELEPHONY_API_URL and TELEPHONY_API_KEY environment variables");
            tracing::info!("[DEV MODE] Would call {} with message: {}", phone, message);
            // В dev режиме продолжаем без ошибки
            return Ok(());
        }

        tracing::info!("Calling {} with message: {}", phone, message);
        
        // Пример использования API телефонии (можно адаптировать под любой провайдер, например Twilio)
        let response = self.client
            .post(telephony_api_url.as_ref().unwrap())
            .header("Authorization", format!("Bearer {}", telephony_api_key.as_ref().unwrap()))
            .json(&serde_json::json!({
                "phone": phone,
                "message": message,
                "type": "call" // Или "tts" для text-to-speech
            }))
            .send()
            .await
            .map_err(|e| format!("Telephony API request failed: {}", e))?;
        
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!("Telephony API returned error: {} - {}", status, error_text);
            return Err(format!("Telephony API returned error: {}", status));
        }
        
        tracing::info!("Call initiated successfully to {}", phone);
        Ok(())
    }

    /// Формирует сообщение для звонка владельцу о блокировке
    pub fn format_block_notification_message(
        &self,
        blocked_plate: &str,
        blocker_name: &str,
    ) -> String {
        format!(
            "Здравствуйте! Ваш автомобиль {} заблокирован пользователем {}. Пожалуйста, проверьте приложение.",
            blocked_plate,
            blocker_name
        )
    }
}

