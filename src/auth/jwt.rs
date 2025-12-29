use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::Config;
use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid, // user_id
    pub exp: i64,
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
    pub fn is_expired_or_expiring_soon(&self) -> bool {
        let now = Utc::now().timestamp();
        let expires_at = self.exp;
        // Токен считается истекающим, если до истечения осталось меньше 30 секунд
        expires_at <= now + 30
    }
}

pub fn create_token(user_id: Uuid, config: &Config) -> AppResult<String> {
    let claims = Claims::new(user_id, config.jwt_expiration_minutes);
    let key = EncodingKey::from_secret(config.jwt_secret.as_ref());

    encode(&Header::default(), &claims, &key)
        .map_err(|e| AppError::Auth(format!("Failed to create token: {}", e)))
}

pub fn verify_token(token: &str, config: &Config) -> AppResult<Claims> {
    let key = DecodingKey::from_secret(config.jwt_secret.as_ref());
    let validation = Validation::default();

    decode::<Claims>(token, &key, &validation)
        .map(|data| data.claims)
        .map_err(|e| AppError::Auth(format!("Invalid token: {}", e)))
}
