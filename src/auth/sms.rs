use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::redis::{RateLimiter, RedisClient};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

const OTP_PREFIX: &str = "otp:";

#[derive(Clone, Serialize, Deserialize)]
struct OtpEntry {
    code: String,
    expires_at: i64,
}

/// In-memory fallback когда Redis недоступен (тесты / dev)
#[derive(Clone)]
struct MemoryOtpStore {
    codes: Arc<RwLock<HashMap<String, OtpEntry>>>,
}

impl MemoryOtpStore {
    fn new() -> Self {
        Self {
            codes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn set(&self, phone: &str, entry: OtpEntry) {
        self.codes.write().await.insert(phone.to_string(), entry);
    }

    async fn get(&self, phone: &str) -> Option<OtpEntry> {
        let codes = self.codes.read().await;
        codes.get(phone).cloned()
    }

    async fn remove(&self, phone: &str) {
        self.codes.write().await.remove(phone);
    }
}

#[derive(Clone)]
pub struct SmsService {
    config: Config,
    redis: Option<RedisClient>,
    memory: MemoryOtpStore,
    rate_limiter: Option<RateLimiter>,
}

impl SmsService {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            redis: None,
            memory: MemoryOtpStore::new(),
            rate_limiter: None,
        }
    }

    pub fn with_redis(config: Config, redis: RedisClient) -> Self {
        let rate_limiter = RateLimiter::new(redis.clone(), config.clone());
        Self {
            config,
            redis: Some(redis),
            memory: MemoryOtpStore::new(),
            rate_limiter: Some(rate_limiter),
        }
    }

    pub fn rate_limiter(&self) -> Option<&RateLimiter> {
        self.rate_limiter.as_ref()
    }

    pub async fn check_rate_limit(&self, phone: &str) -> AppResult<()> {
        if let Some(limiter) = &self.rate_limiter {
            limiter.check_otp_start(phone).await
        } else {
            Ok(())
        }
    }

    pub async fn check_verify_attempts(&self, phone: &str) -> AppResult<()> {
        if let Some(limiter) = &self.rate_limiter {
            limiter.check_otp_verify(phone).await?;
        }
        Ok(())
    }

    pub async fn reset_verify_attempts(&self, phone: &str) -> AppResult<()> {
        if let Some(limiter) = &self.rate_limiter {
            limiter.reset_otp_verify(phone).await?;
        }
        Ok(())
    }

    pub async fn generate_code(&self, phone: &str) -> AppResult<String> {
        self.check_rate_limit(phone).await?;

        let max_value = 10_u32.pow(self.config.sms_code_length);
        let code = format!(
            "{:0width$}",
            rand::random::<u32>() % max_value,
            width = self.config.sms_code_length as usize
        );

        let expires_at = chrono::Utc::now().timestamp()
            + self.config.sms_code_expiration_minutes * 60;

        let entry = OtpEntry {
            code: code.clone(),
            expires_at,
        };

        if let Some(redis) = &self.redis {
            let key = format!("{}{}", OTP_PREFIX, phone);
            let json = serde_json::to_string(&entry)
                .map_err(|e| AppError::Internal(format!("OTP serialize error: {}", e)))?;
            redis
                .set_ex(
                    &key,
                    &json,
                    (self.config.sms_code_expiration_minutes * 60) as u64,
                )
                .await?;
        } else {
            self.memory.set(phone, entry).await;
        }

        Ok(code)
    }

    pub async fn verify_code(&self, phone: &str, code: &str) -> AppResult<bool> {
        self.check_verify_attempts(phone).await?;

        let entry = if let Some(redis) = &self.redis {
            let key = format!("{}{}", OTP_PREFIX, phone);
            match redis.get(&key).await? {
                Some(json) => serde_json::from_str(&json).ok(),
                None => None,
            }
        } else {
            self.memory.get(phone).await
        };

        let valid = match entry {
            Some(e) => e.code == code && e.expires_at > chrono::Utc::now().timestamp(),
            None => false,
        };

        Ok(valid)
    }

    pub async fn remove_code(&self, phone: &str) -> AppResult<()> {
        if let Some(redis) = &self.redis {
            redis.del(&format!("{}{}", OTP_PREFIX, phone)).await?;
        } else {
            self.memory.remove(phone).await;
        }
        self.reset_verify_attempts(phone).await
    }
}
