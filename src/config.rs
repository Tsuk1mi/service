use anyhow::{Context, Result};
use std::env;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_minutes: i64,
    pub encryption_key: String,
    pub server_host: String,
    pub server_port: u16,
    pub migrations_path: String,
    pub sms_code_expiration_minutes: i64,
    pub sms_code_length: u32,
    pub return_sms_code_in_response: bool,
    pub fcm_server_key: Option<String>,
    pub min_client_version: Option<String>,
    pub release_client_version: Option<String>,
    pub app_download_url: Option<String>,
    pub app_apk_path: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let database_url = env::var("DATABASE_URL").context("DATABASE_URL is required")?;
        let jwt_secret = env::var("JWT_SECRET").context("JWT_SECRET is required")?;
        // Срок жизни JWT: по умолчанию 7 дней (10080 минут)
        let jwt_expiration_minutes = env::var("JWT_EXPIRATION_MINUTES")
            .unwrap_or_else(|_| "10080".to_string())
            .parse()
            .context("JWT_EXPIRATION_MINUTES must be a valid number")?;

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
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);
        let fcm_server_key = env::var("FCM_SERVER_KEY").ok();
        let min_client_version = env::var("MIN_CLIENT_VERSION").ok();
        let release_client_version = env::var("RELEASE_CLIENT_VERSION").ok();
        let app_download_url = env::var("APP_DOWNLOAD_URL").ok();
        let app_apk_path = env::var("APP_APK_PATH").ok();

        Ok(Config {
            database_url,
            jwt_secret,
            jwt_expiration_minutes,
            encryption_key,
            server_host,
            server_port,
            migrations_path,
            sms_code_expiration_minutes,
            sms_code_length,
            return_sms_code_in_response,
            fcm_server_key,
            min_client_version,
            release_client_version,
            app_download_url,
            app_apk_path,
        })
    }
}
