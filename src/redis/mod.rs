use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::Config;
use crate::error::{AppError, AppResult};

/// Обёртка над Redis connection manager
#[derive(Clone)]
pub struct RedisClient {
    conn: Arc<Mutex<ConnectionManager>>,
}

impl RedisClient {
    pub async fn connect(url: &str) -> AppResult<Self> {
        let client = redis::Client::open(url)
            .map_err(|e| AppError::Internal(format!("Redis client error: {}", e)))?;
        let conn = client
            .get_connection_manager()
            .await
            .map_err(|e| AppError::Internal(format!("Redis connection error: {}", e)))?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub async fn ping(&self) -> AppResult<()> {
        let mut conn = self.conn.lock().await;
        let _: String = redis::cmd("PING")
            .query_async(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("Redis ping failed: {}", e)))?;
        Ok(())
    }

    pub async fn set_ex(&self, key: &str, value: &str, ttl_secs: u64) -> AppResult<()> {
        let mut conn = self.conn.lock().await;
        conn.set_ex::<_, _, ()>(key, value, ttl_secs)
            .await
            .map_err(|e| AppError::Internal(format!("Redis SET error: {}", e)))?;
        Ok(())
    }

    pub async fn get(&self, key: &str) -> AppResult<Option<String>> {
        let mut conn = self.conn.lock().await;
        let val: Option<String> = conn
            .get(key)
            .await
            .map_err(|e| AppError::Internal(format!("Redis GET error: {}", e)))?;
        Ok(val)
    }

    pub async fn del(&self, key: &str) -> AppResult<()> {
        let mut conn = self.conn.lock().await;
        conn.del::<_, ()>(key)
            .await
            .map_err(|e| AppError::Internal(format!("Redis DEL error: {}", e)))?;
        Ok(())
    }

    pub async fn incr_with_ttl(&self, key: &str, ttl_secs: u64) -> AppResult<u32> {
        let mut conn = self.conn.lock().await;
        let count: i64 = conn
            .incr(key, 1i64)
            .await
            .map_err(|e| AppError::Internal(format!("Redis INCR error: {}", e)))?;
        if count == 1 {
            conn.expire::<_, ()>(key, ttl_secs as i64)
                .await
                .map_err(|e| AppError::Internal(format!("Redis EXPIRE error: {}", e)))?;
        }
        Ok(count as u32)
    }
}

/// Rate limiter на базе Redis
#[derive(Clone)]
pub struct RateLimiter {
    redis: RedisClient,
    config: Config,
}

impl RateLimiter {
    pub fn new(redis: RedisClient, config: Config) -> Self {
        Self { redis, config }
    }

    pub async fn check_otp_start(&self, phone: &str) -> AppResult<()> {
        let key = format!("otp:rate:{}", phone);
        let count = self
            .redis
            .incr_with_ttl(&key, self.config.otp_rate_limit_window_secs)
            .await?;
        if count > self.config.otp_rate_limit_max {
            return Err(AppError::RateLimit(
                "Слишком много запросов кода. Попробуйте позже.".to_string(),
            ));
        }
        Ok(())
    }

    pub async fn check_otp_verify(&self, phone: &str) -> AppResult<u32> {
        let key = format!("otp:attempts:{}", phone);
        let count = self
            .redis
            .incr_with_ttl(&key, self.config.sms_code_expiration_minutes as u64 * 60)
            .await?;
        if count > self.config.otp_verify_max_attempts {
            return Err(AppError::RateLimit(
                "Превышено число попыток. Запросите новый код.".to_string(),
            ));
        }
        Ok(count)
    }

    pub async fn reset_otp_verify(&self, phone: &str) -> AppResult<()> {
        self.redis.del(&format!("otp:attempts:{}", phone)).await
    }
}

/// JWT refresh token blacklist (Redis)
#[derive(Clone)]
pub struct JwtBlacklist {
    redis: RedisClient,
}

impl JwtBlacklist {
    pub fn new(redis: RedisClient) -> Self {
        Self { redis }
    }

    pub async fn blacklist_jti(&self, jti: &str, ttl_secs: u64) -> AppResult<()> {
        self.redis
            .set_ex(&format!("jwt:blacklist:{}", jti), "1", ttl_secs)
            .await
    }

    pub async fn is_blacklisted(&self, jti: &str) -> AppResult<bool> {
        Ok(self
            .redis
            .get(&format!("jwt:blacklist:{}", jti))
            .await?
            .is_some())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn blacklist_key_format() {
        let jti = "test-jti-uuid";
        assert_eq!(format!("jwt:blacklist:{}", jti), "jwt:blacklist:test-jti-uuid");
    }
}
