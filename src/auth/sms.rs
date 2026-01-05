use crate::config::Config;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

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

    /// Генерирует и сохраняет код для телефона.
    ///
    /// Доставка кода (SMS/Telegram) выполняется на уровне сервиса авторизации.
    pub async fn generate_code(&self, phone: &str) -> Result<String, String> {
        // Генерируем код заданной длины
        let max_value = 10_u32.pow(self.config.sms_code_length);
        let code = format!(
            "{:0width$}",
            rand::random::<u32>() % max_value,
            width = self.config.sms_code_length as usize
        );

        let entry = CodeEntry {
            code: code.clone(),
            expires_at: chrono::Utc::now()
                + chrono::Duration::minutes(self.config.sms_code_expiration_minutes),
            user_id: None,
        };

        let mut codes = self.codes.write().await;
        codes.insert(phone.to_string(), entry);

        Ok(code)
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
