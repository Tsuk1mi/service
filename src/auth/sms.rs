use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::config::Config;
use reqwest::Client;

/// Простое хранилище кодов для MVP (в продакшене использовать Redis)
pub type CodeStorage = Arc<RwLock<HashMap<String, CodeEntry>>>;

#[derive(Clone)]
pub struct CodeEntry {
    pub code: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub user_id: Option<Uuid>,
}

#[derive(Clone)]
pub struct SmsService {
    codes: CodeStorage,
    config: Config,
}

impl SmsService {
    pub fn new(config: Config) -> Self {
        Self {
            codes: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Генерирует и сохраняет код для телефона, отправляет SMS
    pub async fn generate_code(&self, phone: &str) -> Result<String, String> {
        // Генерируем код заданной длины
        let max_value = 10_u32.pow(self.config.sms_code_length);
        let code = format!("{:0width$}", rand::random::<u32>() % max_value, width = self.config.sms_code_length as usize);
        
        let entry = CodeEntry {
            code: code.clone(),
            expires_at: chrono::Utc::now() + chrono::Duration::minutes(self.config.sms_code_expiration_minutes),
            user_id: None,
        };

        let mut codes = self.codes.write().await;
        codes.insert(phone.to_string(), entry);
        
        // Отправляем SMS
        match self.send_sms(phone, &code).await {
            Ok(_) => {
                tracing::info!("SMS code generated and sent successfully to {}", phone);
            }
            Err(e) => {
                tracing::warn!("Failed to send SMS to {}: {}", phone, e);
                // Всегда логируем код для разработки/отладки
                tracing::info!("[DEV MODE] SMS code for {}: {} (expires in {} minutes)", 
                    phone, code, self.config.sms_code_expiration_minutes);
                // В dev режиме продолжаем работу даже если SMS не отправилось
                if self.config.return_sms_code_in_response {
                    tracing::info!("Continuing in dev mode - code will be returned in response");
                } else {
                    // В prod режиме возвращаем ошибку только если SMS обязательно нужен
                    // Для гибкости, продолжаем работу даже без SMS если код возвращается в ответе
                    tracing::warn!("SMS failed but continuing - code logged above");
                }
            }
        }
        
        Ok(code)
    }
    
    /// Отправляет SMS с кодом подтверждения
    async fn send_sms(&self, phone: &str, code: &str) -> Result<(), String> {
        // Проверяем, есть ли настройки SMS провайдера
        let sms_api_url = std::env::var("SMS_API_URL").ok();
        let sms_api_key = std::env::var("SMS_API_KEY").ok();
        
        if sms_api_url.is_none() || sms_api_key.is_none() {
            // Если нет настроек SMS провайдера
            // В dev режиме (return_sms_code_in_response=true) продолжаем без ошибки
            // В prod режиме возвращаем ошибку
            if self.config.return_sms_code_in_response {
                tracing::info!("SMS provider not configured, but continuing in dev mode. Code: {}", code);
                return Ok(());
            } else {
                return Err("SMS provider not configured. Set SMS_API_URL and SMS_API_KEY environment variables".to_string());
            }
        }
        
        let client = Client::new();
        let message = format!("Ваш код подтверждения: {}", code);
        
        // Пример использования SMS API (можно адаптировать под любой провайдер)
        let response = client
            .post(sms_api_url.as_ref().unwrap())
            .header("Authorization", format!("Bearer {}", sms_api_key.as_ref().unwrap()))
            .json(&serde_json::json!({
                "phone": phone,
                "message": message
            }))
            .send()
            .await
            .map_err(|e| format!("SMS API request failed: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("SMS API returned error: {}", response.status()));
        }
        
        tracing::info!("SMS sent successfully to {}", phone);
        Ok(())
    }

    /// Проверяет код
    pub async fn verify_code(&self, phone: &str, code: &str) -> bool {
        let codes = self.codes.read().await;
        
        if let Some(entry) = codes.get(phone) {
            if entry.code == code && entry.expires_at > chrono::Utc::now() {
                return true;
            }
        }
        
        false
    }

    /// Удаляет использованный код
    pub async fn remove_code(&self, phone: &str) {
        let mut codes = self.codes.write().await;
        codes.remove(phone);
    }
}


