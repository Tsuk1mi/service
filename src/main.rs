use anyhow::{Context, Result};
use axum::{middleware, routing::get, Router};
use rimskiy_service::api::{
    auth_router, block_router, health_router, notification_router, ocr_router,
    server_info_router, user_plate_router, user_router, AppState,
};
use rimskiy_service::auth::sms::SmsService;
use rimskiy_service::config::Config;
use rimskiy_service::db::{create_pool, init::run_migrations, DbPool};
use rimskiy_service::error::AppError;
use rimskiy_service::metrics;
use rimskiy_service::middleware::{build_cors_layer, logging_middleware, metrics_auth_middleware};
use rimskiy_service::openapi::ApiDoc;
use rimskiy_service::queue::{EventPublisher, NoopPublisher, RabbitMqPublisher};
use rimskiy_service::redis::{JwtBlacklist, RedisClient};
use rimskiy_service::repository::{
    PostgresBlockRepository, PostgresNotificationRepository, PostgresTelegramBotRepository,
    PostgresUserPlateRepository, PostgresUserRepository,
};
use rimskiy_service::service::{
    AuthService, BlockService, PushService, TelegramService, TelephonyService, UserService,
};
use rimskiy_service::utils::encryption::Encryption;
use std::net::SocketAddr;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let default_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&default_filter));

    if std::env::var("APP_ENV")
        .map(|v| v == "production")
        .unwrap_or(false)
    {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(env_filter)
            .init();
    } else {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    }

    let prometheus_handle = metrics::init_metrics();

    let config = Config::from_env()?;

    let pool = create_pool(&config.database_url).await?;
    tracing::info!("Connected to database");

    run_migrations(&pool).await?;

    if std::env::var("RUN_MIGRATIONS_ONLY")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
    {
        tracing::info!("Migrations complete, exiting (RUN_MIGRATIONS_ONLY)");
        return Ok(());
    }

    let redis = if let Some(ref url) = config.redis_url {
        match RedisClient::connect(url).await {
            Ok(client) => {
                tracing::info!("Connected to Redis");
                Some(client)
            }
            Err(e) => {
                if config.app_env.is_production() {
                    anyhow::bail!("Redis is required in production: {}", e);
                }
                tracing::warn!("Redis unavailable, using in-memory OTP: {:?}", e);
                None
            }
        }
    } else {
        if config.app_env.is_production() {
            anyhow::bail!("REDIS_URL is required in production");
        }
        tracing::info!("REDIS_URL not set, using in-memory OTP store");
        None
    };

    let event_publisher: Arc<dyn EventPublisher> =
        if let Some(ref url) = config.rabbitmq_url {
            match RabbitMqPublisher::connect(url).await {
                Ok(publisher) => {
                    tracing::info!("Connected to RabbitMQ");
                    Arc::new(publisher)
                }
                Err(e) => {
                    tracing::warn!("RabbitMQ unavailable, using noop publisher: {:?}", e);
                    Arc::new(NoopPublisher)
                }
            }
        } else {
            tracing::info!("RABBITMQ_URL not set, using noop event publisher");
            Arc::new(NoopPublisher)
        };

    let encryption =
        Encryption::new(&config.encryption_key).map_err(|e| AppError::Encryption(e.to_string()))?;

    let sms_service = if let Some(ref redis_client) = redis {
        SmsService::with_redis(config.clone(), redis_client.clone())
    } else {
        SmsService::new(config.clone())
    };

    let telephony_service = TelephonyService::new(config.clone());
    let telegram_service = TelegramService::new(&config);

    let db_pool: DbPool = Arc::new(pool);
    let user_repository = PostgresUserRepository::new(db_pool.clone());
    let block_repository = PostgresBlockRepository::new(db_pool.clone());
    let user_plate_repository = PostgresUserPlateRepository::new(db_pool.clone());
    let notification_repository = PostgresNotificationRepository::new(db_pool.clone());
    let telegram_bot_repository = PostgresTelegramBotRepository::new(db_pool.clone());

    let jwt_blacklist = redis.as_ref().map(|r| JwtBlacklist::new(r.clone()));

    let auth_service = AuthService::new(
        sms_service.clone(),
        encryption.clone(),
        config.clone(),
        event_publisher.clone(),
        jwt_blacklist,
    );
    let user_service = UserService::new(encryption.clone());
    let push_service = PushService::new(config.fcm_server_key.clone());
    let block_service = BlockService::new(encryption.clone());

    let app_state = AppState {
        config: config.clone(),
        db_pool: db_pool.clone(),
        redis: redis.clone(),
        event_publisher,
        http_client: reqwest::Client::new(),
        encryption,
        sms_service,
        telephony_service,
        telegram_service,
        auth_service,
        push_service,
        user_service,
        block_service,
        user_repository,
        block_repository,
        user_plate_repository,
        notification_repository,
        telegram_bot_repository,
    };

    let openapi = ApiDoc::openapi();

    let auth_layer = axum::middleware::from_fn_with_state(
        app_state.clone(),
        rimskiy_service::auth::middleware::auth_middleware,
    );

    let mut app = Router::new()
        .route("/health", get(health_check))
        .nest("/health", health_router())
        .route(
            "/metrics",
            get({
                let handle = prometheus_handle.clone();
                move || {
                    let handle = handle.clone();
                    async move { handle.render() }
                }
            })
            .layer(middleware::from_fn_with_state(
                app_state.clone(),
                metrics_auth_middleware,
            )),
        )
        .merge(server_info_router())
        .nest("/api/auth", auth_router())
        .nest(
            "/api/ocr",
            ocr_router().layer(auth_layer.clone()),
        )
        .nest(
            "/api/users",
            user_router().layer(auth_layer.clone()),
        )
        .nest(
            "/api/user/plates",
            user_plate_router().layer(auth_layer.clone()),
        )
        .nest(
            "/api/blocks",
            block_router().layer(auth_layer.clone()),
        )
        .nest(
            "/api/notifications",
            notification_router().layer(auth_layer),
        )
        .layer(build_cors_layer(&config))
        .layer(middleware::from_fn(logging_middleware))
        .with_state(app_state);

    if !config.app_env.is_production() {
        app = app.merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", openapi));
    }

    let addr = SocketAddr::from((
        config
            .server_host
            .parse::<std::net::IpAddr>()
            .with_context(|| format!("Invalid SERVER_HOST: {}", config.server_host))?,
        config.server_port,
    ));
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}
