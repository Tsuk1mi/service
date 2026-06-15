use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::Config;
use crate::error::{AppError, AppResult};

/// Тип JWT токена
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Access,
    Refresh,
}

/// JWT claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: i64,
    pub iat: i64,
    #[serde(default = "new_jti")]
    pub jti: String,
    #[serde(default = "default_token_type")]
    pub typ: TokenType,
}

fn new_jti() -> String {
    Uuid::new_v4().to_string()
}

fn default_token_type() -> TokenType {
    TokenType::Access
}

impl Claims {
    pub fn new(user_id: Uuid, expiration_minutes: i64, typ: TokenType) -> Self {
        let now = Utc::now();
        let exp = now + Duration::minutes(expiration_minutes);
        Self {
            sub: user_id,
            exp: exp.timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
            typ,
        }
    }

    pub fn is_expired_or_expiring_soon(&self) -> bool {
        let now = Utc::now().timestamp();
        self.exp <= now + 30
    }
}

pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
}

/// Создаёт пару access + refresh токенов
pub fn create_token_pair(user_id: Uuid, config: &Config) -> AppResult<TokenPair> {
    let access = create_access_token(user_id, config)?;
    let refresh = create_refresh_token(user_id, config)?;
    Ok(TokenPair {
        access_token: access,
        refresh_token: refresh,
    })
}

/// Обратная совместимость: создаёт access token
pub fn create_token(user_id: Uuid, config: &Config) -> AppResult<String> {
    create_access_token(user_id, config)
}

pub fn create_access_token(user_id: Uuid, config: &Config) -> AppResult<String> {
    let claims = Claims::new(
        user_id,
        config.jwt_access_expiration_minutes,
        TokenType::Access,
    );
    encode_token(&claims, config)
}

pub fn create_refresh_token(user_id: Uuid, config: &Config) -> AppResult<String> {
    let claims = Claims::new(
        user_id,
        config.jwt_refresh_expiration_minutes,
        TokenType::Refresh,
    );
    encode_token(&claims, config)
}

fn encode_token(claims: &Claims, config: &Config) -> AppResult<String> {
    let key = EncodingKey::from_secret(config.jwt_secret.as_ref());
    encode(&Header::default(), claims, &key)
        .map_err(|e| AppError::Auth(format!("Failed to create token: {}", e)))
}

/// Проверяет access token
pub fn verify_token(token: &str, config: &Config) -> AppResult<Claims> {
    verify_token_with_type(token, config, TokenType::Access)
}

pub fn verify_refresh_token(token: &str, config: &Config) -> AppResult<Claims> {
    verify_token_with_type(token, config, TokenType::Refresh)
}

fn verify_token_with_type(
    token: &str,
    config: &Config,
    expected_type: TokenType,
) -> AppResult<Claims> {
    let key = DecodingKey::from_secret(config.jwt_secret.as_ref());
    let mut validation = Validation::default();
    validation.validate_exp = true;

    let claims = decode::<Claims>(token, &key, &validation)
        .map(|data| data.claims)
        .map_err(|e| AppError::Auth(format!("Invalid token: {}", e)))?;

    if claims.typ != expected_type {
        return Err(AppError::Auth(format!(
            "Invalid token type: expected {:?}",
            expected_type
        )));
    }

    Ok(claims)
}

/// Декодирует токен без проверки exp (для refresh flow)
pub fn decode_token_ignore_exp(token: &str, config: &Config) -> AppResult<Claims> {
    let key = DecodingKey::from_secret(config.jwt_secret.as_ref());
    let mut validation = Validation::default();
    validation.validate_exp = false;

    decode::<Claims>(token, &key, &validation)
        .map(|data| data.claims)
        .map_err(|e| AppError::Auth(format!("Invalid token format: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AppEnv, Config};

    fn test_config() -> Config {
        Config {
            app_env: AppEnv::Development,
            database_url: "postgresql://localhost/test".into(),
            redis_url: None,
            rabbitmq_url: None,
            jwt_secret: "test-secret-key-minimum-32-characters-long".into(),
            jwt_expiration_minutes: 10080,
            jwt_access_expiration_minutes: 15,
            jwt_refresh_expiration_minutes: 10080,
            encryption_key: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                .into(),
            server_host: "0.0.0.0".into(),
            server_port: 8080,
            migrations_path: "./migrations".into(),
            sms_code_expiration_minutes: 10,
            sms_code_length: 4,
            return_sms_code_in_response: false,
            sms_api_url: None,
            sms_api_key: None,
            fcm_server_key: None,
            web_app_url: None,
            cors_allowed_origins: vec![],
            telegram_bot_http_url: None,
            otp_rate_limit_max: 3,
            otp_rate_limit_window_secs: 900,
            otp_verify_max_attempts: 5,
            internal_api_token: None,
            metrics_auth_user: None,
            metrics_auth_password: None,
        }
    }

    #[test]
    fn create_and_verify_access_token() {
        let config = test_config();
        let user_id = Uuid::new_v4();
        let token = create_access_token(user_id, &config).unwrap();
        let claims = verify_token(&token, &config).unwrap();
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.typ, TokenType::Access);
    }

    #[test]
    fn refresh_token_has_refresh_type() {
        let config = test_config();
        let user_id = Uuid::new_v4();
        let token = create_refresh_token(user_id, &config).unwrap();
        let claims = verify_refresh_token(&token, &config).unwrap();
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.typ, TokenType::Refresh);
    }

    #[test]
    fn access_token_rejected_as_refresh() {
        let config = test_config();
        let user_id = Uuid::new_v4();
        let token = create_access_token(user_id, &config).unwrap();
        assert!(verify_refresh_token(&token, &config).is_err());
    }
}
