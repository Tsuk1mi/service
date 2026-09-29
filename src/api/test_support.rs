//! Shared helpers for Axum router smoke tests (no live DB/Redis).

use std::sync::Arc;

use crate::api::AppState;
use crate::auth::sms::SmsService;
use crate::config::{AppEnv, Config};
use crate::db::DbPool;
use crate::queue::NoopPublisher;
use crate::repository::{
    PostgresBlockRepository, PostgresNotificationRepository, PostgresTelegramBotRepository,
    PostgresUserPlateRepository, PostgresUserRepository,
};
use crate::service::{
    AuthService, BlockService, PushService, TelegramService, TelephonyService, UserService,
};
use crate::utils::encryption::Encryption;

pub fn test_config() -> Config {
    Config {
        app_env: AppEnv::Development,
        database_url: "postgres://rimskiy:rimskiy@127.0.0.1:5432/rimskiy".into(),
        redis_url: None,
        rabbitmq_url: None,
        jwt_secret: "test-jwt-secret-at-least-32-chars!!".into(),
        jwt_expiration_minutes: 60,
        jwt_access_expiration_minutes: 15,
        jwt_refresh_expiration_minutes: 10080,
        encryption_key: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".into(),
        server_host: "0.0.0.0".into(),
        server_port: 8080,
        migrations_path: "./migrations".into(),
        sms_code_expiration_minutes: 10,
        sms_code_length: 4,
        return_sms_code_in_response: false,
        sms_api_url: None,
        sms_api_key: None,
        fcm_server_key: None,
        web_app_url: Some("http://localhost".into()),
        cors_allowed_origins: vec!["http://localhost".into()],
        telegram_bot_http_url: None,
        otp_rate_limit_max: 3,
        otp_rate_limit_window_secs: 900,
        otp_verify_max_attempts: 5,
        internal_api_token: None,
        metrics_auth_user: None,
        metrics_auth_password: None,
    }
}

pub fn test_app_state() -> AppState {
    let config = test_config();
    let pool =
        sqlx::PgPool::connect_lazy(&config.database_url).expect("lazy postgres pool for tests");
    let db_pool: DbPool = Arc::new(pool);
    let encryption = Encryption::new(&config.encryption_key).expect("test encryption key");
    let event_publisher = Arc::new(NoopPublisher);
    let sms_service = SmsService::new(config.clone());
    let auth_service = AuthService::new(
        sms_service.clone(),
        encryption.clone(),
        config.clone(),
        event_publisher.clone(),
        None,
    );

    AppState {
        config: config.clone(),
        db_pool: db_pool.clone(),
        redis: None,
        event_publisher,
        http_client: reqwest::Client::new(),
        encryption: encryption.clone(),
        sms_service,
        telephony_service: TelephonyService::new(config.clone()),
        telegram_service: TelegramService::new(&config),
        push_service: PushService::new(None),
        auth_service,
        user_service: UserService::new(encryption.clone()),
        block_service: BlockService::new(encryption),
        user_repository: PostgresUserRepository::new(db_pool.clone()),
        block_repository: PostgresBlockRepository::new(db_pool.clone()),
        user_plate_repository: PostgresUserPlateRepository::new(db_pool.clone()),
        notification_repository: PostgresNotificationRepository::new(db_pool.clone()),
        telegram_bot_repository: PostgresTelegramBotRepository::new(db_pool),
    }
}
