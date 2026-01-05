use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::Config;
use crate::error::{AppError, AppResult};

/// JWT claims (данные токена)
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// ID пользователя (subject)
    pub sub: Uuid,
    /// Время истечения (expiration timestamp)
    pub exp: i64,
    /// Время выдачи (issued at timestamp)
    pub iat: i64,
}

impl Claims {
    pub fn new(user_id: Uuid, expiration_minutes: i64) -> Self {
        let now = Utc::now();
        let exp = now + Duration::minutes(expiration_minutes);

        Self {
            sub: user_id,
            exp: exp.timestamp(),
            iat: now.timestamp(),
        }
    }

    /// Проверяет, истек ли токен или скоро истечет (менее чем через 30 секунд)
    ///
    /// Используется для автоматического обновления токена на клиенте
    pub fn is_expired_or_expiring_soon(&self) -> bool {
        let now = Utc::now().timestamp();
        let expires_at = self.exp;
        // Токен считается истекающим, если до истечения осталось меньше 30 секунд
        expires_at <= now + 30
    }
}

/// Создает новый JWT токен для пользователя
pub fn create_token(user_id: Uuid, config: &Config) -> AppResult<String> {
    let claims = Claims::new(user_id, config.jwt_expiration_minutes);
    let key = EncodingKey::from_secret(config.jwt_secret.as_ref());

    encode(&Header::default(), &claims, &key)
        .map_err(|e| AppError::Auth(format!("Failed to create token: {}", e)))
}

/// Проверяет и декодирует JWT токен
pub fn verify_token(token: &str, config: &Config) -> AppResult<Claims> {
    let key = DecodingKey::from_secret(config.jwt_secret.as_ref());
    let mut validation = Validation::default();
    // Не позволяем использовать токен с истекшим сроком жизни
    validation.validate_exp = true;

    decode::<Claims>(token, &key, &validation)
        .map(|data| data.claims)
        .map_err(|e| AppError::Auth(format!("Invalid token: {}", e)))
}
