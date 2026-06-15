use anyhow::{Context, Result};
use std::env;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppEnv {
    Development,
    Production,
}

impl AppEnv {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "production" | "prod" => Self::Production,
            _ => Self::Development,
        }
    }

    pub fn is_production(&self) -> bool {
        matches!(self, Self::Production)
    }
}

#[derive(Clone)]
pub struct Config {
    pub app_env: AppEnv,
    pub database_url: String,
    pub redis_url: Option<String>,
    pub rabbitmq_url: Option<String>,
    pub jwt_secret: String,
    pub jwt_expiration_minutes: i64,
    pub jwt_access_expiration_minutes: i64,
    pub jwt_refresh_expiration_minutes: i64,
    pub encryption_key: String,
    pub server_host: String,
    pub server_port: u16,
    pub migrations_path: String,
    pub sms_code_expiration_minutes: i64,
    pub sms_code_length: u32,
    pub return_sms_code_in_response: bool,
    pub sms_api_url: Option<String>,
    pub sms_api_key: Option<String>,
    pub fcm_server_key: Option<String>,
    pub web_app_url: Option<String>,
    pub cors_allowed_origins: Vec<String>,
    /// URL HTTP-эндпоинта бота для доставки SMS-кода
    pub telegram_bot_http_url: Option<String>,
    pub otp_rate_limit_max: u32,
    pub otp_rate_limit_window_secs: u64,
    pub otp_verify_max_attempts: u32,
    pub internal_api_token: Option<String>,
    pub metrics_auth_user: Option<String>,
    pub metrics_auth_password: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let app_env = AppEnv::from_str(&env::var("APP_ENV").unwrap_or_else(|_| "development".into()));

        let database_url = env::var("DATABASE_URL").context("DATABASE_URL is required")?;
        let redis_url = env::var("REDIS_URL").ok().filter(|s| !s.is_empty());
        let rabbitmq_url = env::var("RABBITMQ_URL").ok().filter(|s| !s.is_empty());

        let jwt_secret = env::var("JWT_SECRET").context("JWT_SECRET is required")?;
        if jwt_secret.len() < 32 {
            anyhow::bail!("JWT_SECRET must be at least 32 characters");
        }

        let jwt_expiration_minutes = env::var("JWT_EXPIRATION_MINUTES")
            .unwrap_or_else(|_| "10080".to_string())
            .parse()
            .context("JWT_EXPIRATION_MINUTES must be a valid number")?;
        let jwt_access_expiration_minutes = env::var("JWT_ACCESS_EXPIRATION_MINUTES")
            .unwrap_or_else(|_| "15".to_string())
            .parse()
            .context("JWT_ACCESS_EXPIRATION_MINUTES must be a valid number")?;
        let jwt_refresh_expiration_minutes = env::var("JWT_REFRESH_EXPIRATION_MINUTES")
            .unwrap_or_else(|_| "10080".to_string())
            .parse()
            .context("JWT_REFRESH_EXPIRATION_MINUTES must be a valid number")?;

        let encryption_key = env::var("ENCRYPTION_KEY")
            .context("ENCRYPTION_KEY is required (must be 64 hex characters)")?;
        if encryption_key.len() != 64 || hex::decode(&encryption_key).is_err() {
            anyhow::bail!("ENCRYPTION_KEY must be 64 hex characters");
        }

        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .context("SERVER_PORT must be a valid number")?;
        let migrations_path =
            env::var("MIGRATIONS_PATH").unwrap_or_else(|_| "./migrations".to_string());
        let sms_code_expiration_minutes = env::var("SMS_CODE_EXPIRATION_MINUTES")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .context("SMS_CODE_EXPIRATION_MINUTES must be a valid number")?;
        let sms_code_length = env::var("SMS_CODE_LENGTH")
            .unwrap_or_else(|_| "4".to_string())
            .parse()
            .context("SMS_CODE_LENGTH must be a valid number")?;

        let return_sms_code_in_response = env::var("RETURN_SMS_CODE_IN_RESPONSE")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        if app_env.is_production() && return_sms_code_in_response {
            anyhow::bail!(
                "RETURN_SMS_CODE_IN_RESPONSE must be false in production (APP_ENV=production)"
            );
        }

        if app_env.is_production() && redis_url.is_none() {
            anyhow::bail!("REDIS_URL is required in production");
        }

        let sms_api_url = env::var("SMS_API_URL").ok();
        let sms_api_key = env::var("SMS_API_KEY").ok();
        let fcm_server_key = env::var("FCM_SERVER_KEY").ok();
        let web_app_url = env::var("WEB_APP_URL").ok();
        let telegram_bot_http_url = env::var("TELEGRAM_BOT_HTTP_URL").ok();

        let cors_allowed_origins = Self::parse_cors_origins(&web_app_url);

        let otp_rate_limit_max = env::var("OTP_RATE_LIMIT_MAX")
            .unwrap_or_else(|_| "3".to_string())
            .parse()
            .context("OTP_RATE_LIMIT_MAX must be a valid number")?;
        let otp_rate_limit_window_secs = env::var("OTP_RATE_LIMIT_WINDOW_SECS")
            .unwrap_or_else(|_| "900".to_string())
            .parse()
            .context("OTP_RATE_LIMIT_WINDOW_SECS must be a valid number")?;
        let otp_verify_max_attempts = env::var("OTP_VERIFY_MAX_ATTEMPTS")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .context("OTP_VERIFY_MAX_ATTEMPTS must be a valid number")?;

        let internal_api_token = env::var("INTERNAL_API_TOKEN").ok().filter(|s| !s.is_empty());
        let metrics_auth_user = env::var("METRICS_AUTH_USER").ok().filter(|s| !s.is_empty());
        let metrics_auth_password =
            env::var("METRICS_AUTH_PASSWORD").ok().filter(|s| !s.is_empty());

        Ok(Config {
            app_env,
            database_url,
            redis_url,
            rabbitmq_url,
            jwt_secret,
            jwt_expiration_minutes,
            jwt_access_expiration_minutes,
            jwt_refresh_expiration_minutes,
            encryption_key,
            server_host,
            server_port,
            migrations_path,
            sms_code_expiration_minutes,
            sms_code_length,
            return_sms_code_in_response,
            sms_api_url,
            sms_api_key,
            fcm_server_key,
            web_app_url,
            cors_allowed_origins,
            telegram_bot_http_url,
            otp_rate_limit_max,
            otp_rate_limit_window_secs,
            otp_verify_max_attempts,
            internal_api_token,
            metrics_auth_user,
            metrics_auth_password,
        })
    }

    fn parse_cors_origins(web_app_url: &Option<String>) -> Vec<String> {
        let mut origins: Vec<String> = env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if let Some(url) = web_app_url {
            if !origins.contains(url) {
                origins.push(url.clone());
            }
        }

        if origins.is_empty() {
            origins.push("http://localhost:5173".to_string());
            origins.push("http://localhost".to_string());
            origins.push("http://127.0.0.1:5173".to_string());
        }

        origins
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_env_parses_production() {
        assert!(AppEnv::from_str("production").is_production());
        assert!(AppEnv::from_str("PROD").is_production());
        assert!(!AppEnv::from_str("development").is_production());
    }
}
