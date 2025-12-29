use anyhow::{Context, Result};
use axum::{middleware, routing::get, Router};
use rimskiy_service::api::{
    auth_router, block_router, notification_router, ocr_router, server_info_router,
    user_plate_router, user_router, AppState,
};
use rimskiy_service::auth::sms::SmsService;
use rimskiy_service::config::Config;
use rimskiy_service::db::{create_pool, init::ensure_database_and_tables};
use rimskiy_service::error::AppError;
use rimskiy_service::middleware::logging_middleware;
use rimskiy_service::openapi::ApiDoc;
use rimskiy_service::repository::{
    PostgresBlockRepository, PostgresNotificationRepository, PostgresUserPlateRepository,
    PostgresUserRepository,
};
use rimskiy_service::service::{
    AuthService, BlockService, PushService, TelephonyService, UserService,
};
use rimskiy_service::utils::encryption::Encryption;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing_subscriber;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() -> Result<()> {
    // Загружаем переменные окружения
    dotenv::dotenv().ok();

    // Инициализируем логирование
    // Устанавливаем дефолтный уровень логирования, если RUST_LOG не установлен
    let default_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&default_filter)),
        )
        .init();

    // Загружаем конфигурацию
    let config = Config::from_env()?;

    // Создаём пул подключений к БД
    let pool = create_pool(&config.database_url).await?;
    tracing::info!("Connected to database");

    // Автоматически создаём БД и таблицы, если их нет
    ensure_database_and_tables(&pool).await?;
    tracing::info!("Database schema ensured");

    // Инициализируем шифрование
    let encryption =
        Encryption::new(&config.encryption_key).map_err(|e| AppError::Encryption(e.to_string()))?;

    // Инициализируем SMS сервис
    let sms_service = SmsService::new(config.clone());

    // Инициализируем сервис телефонии
    let telephony_service = TelephonyService::new(config.clone());

    // Создаём репозитории
    let db_pool = std::sync::Arc::new(pool);
    let user_repository = PostgresUserRepository::new(db_pool.clone());
    let block_repository = PostgresBlockRepository::new(db_pool.clone());
    let user_plate_repository = PostgresUserPlateRepository::new(db_pool.clone());
    let notification_repository = PostgresNotificationRepository::new(db_pool.clone());

    // Создаём сервисы
    let auth_service = AuthService::new(sms_service.clone(), encryption.clone(), config.clone());
    let user_service = UserService::new(encryption.clone());
    let push_service = PushService::new(config.fcm_server_key.clone());
    let block_service = BlockService::new(encryption.clone(), push_service.clone());

    // Создаём состояние приложения
    let app_state = AppState {
        config: config.clone(),
        encryption,
        sms_service,
        telephony_service,
        auth_service,
        push_service,
        user_service,
        block_service,
        user_repository,
        block_repository,
        user_plate_repository,
        notification_repository,
    };

    // Создаём OpenAPI документацию
    let openapi = ApiDoc::openapi();

    // Создаём роутер
    let app = Router::new()
        .route("/health", get(health_check))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", openapi.clone()))
        .merge(server_info_router())
        .nest("/api/auth", auth_router())
        .nest("/api/ocr", ocr_router())
        .nest(
            "/api/users",
            user_router().layer(axum::middleware::from_fn_with_state(
                app_state.clone(),
                rimskiy_service::auth::middleware::auth_middleware,
            )),
        )
        .nest(
            "/api/user/plates",
            user_plate_router().layer(axum::middleware::from_fn_with_state(
                app_state.clone(),
                rimskiy_service::auth::middleware::auth_middleware,
            )),
        )
        .nest(
            "/api/blocks",
            block_router().layer(axum::middleware::from_fn_with_state(
                app_state.clone(),
                rimskiy_service::auth::middleware::auth_middleware,
            )),
        )
        .nest(
            "/api/notifications",
            notification_router().layer(axum::middleware::from_fn_with_state(
                app_state.clone(),
                rimskiy_service::auth::middleware::auth_middleware,
            )),
        )
        .layer(
            CorsLayer::permissive()
                .allow_origin(tower_http::cors::Any)
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any),
        )
        .layer(middleware::from_fn(logging_middleware))
        .with_state(app_state);

    // Запускаем сервер
    let addr = SocketAddr::from((
        config
            .server_host
            .parse::<std::net::IpAddr>()
            .with_context(|| format!("Invalid SERVER_HOST: {}", config.server_host))?,
        config.server_port,
    ));
    tracing::info!("Server listening on {}", addr);
    println!("[SERVER] ========================================");
    println!("[SERVER] Rimskiy Service Starting...");
    println!("[SERVER] ========================================");
    println!("[SERVER] Server listening on {}", addr);
    println!("[SERVER] Access server at:");
    println!("[SERVER]   - http://localhost:{}", config.server_port);
    println!("[SERVER]   - http://127.0.0.1:{}", config.server_port);
    if config.server_host == "0.0.0.0" {
        println!(
            "[SERVER]   - http://<your-ip>:{} (for network access)",
            config.server_port
        );
    }
    println!("[SERVER] ========================================");
    println!("[SERVER] API Documentation:");
    println!(
        "[SERVER]   - Swagger UI: http://localhost:{}/swagger-ui/",
        config.server_port
    );
    println!(
        "[SERVER]   - OpenAPI JSON: http://localhost:{}/api-doc/openapi.json",
        config.server_port
    );
    println!("[SERVER] ========================================");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}
