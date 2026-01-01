use anyhow::Context;
use axum::{extract::State, http::StatusCode, response::Json, routing::post, Router};
use rimskiy_service::auth::sms::SmsService;
use rimskiy_service::config::Config;
use rimskiy_service::db::pool::create_pool;
use rimskiy_service::repository::{
    PostgresTelegramBotRepository, PostgresUserRepository, TelegramBotRepository, UserRepository,
};
use rimskiy_service::service::validation_service::ValidationService;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone, Debug)]
#[command(
    rename_rule = "lowercase",
    description = "Команды бота для авторизации и проверки блокировок"
)]
enum Command {
    #[command(description = "Показать справку")]
    Help,
    #[command(description = "Запросить код авторизации: /code <телефон>")]
    Code,
    #[command(description = "Проверить блокировку: /block <номер>")]
    Block,
    #[command(description = "Получить последнюю версию приложения: /apk")]
    Apk,
}

#[derive(Clone)]
struct BotConfig {
    sms_code_expiration_minutes: i64,
    sms_code_length: u32,
    return_sms_code_in_response: bool,
    server_host: String,
    server_port: u16,
    app_apk_path: Option<String>,
}

#[derive(Clone)]
struct BotState {
    sms_service: Arc<SmsService>,
    config: Arc<BotConfig>,
    http_client: reqwest::Client,
    api_base_url: String,
    apk_path: Option<String>,
    bot: Bot,
    telegram_bot_repository: Arc<PostgresTelegramBotRepository>,
    user_repository: Arc<PostgresUserRepository>,
}

fn phone_hash(phone: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(phone.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn load_bot_config() -> anyhow::Result<BotConfig> {
    let sms_code_expiration_minutes = std::env::var("SMS_CODE_EXPIRATION_MINUTES")
        .unwrap_or_else(|_| "10".to_string())
        .parse()
        .context("SMS_CODE_EXPIRATION_MINUTES must be a valid number")?;
    let sms_code_length = std::env::var("SMS_CODE_LENGTH")
        .unwrap_or_else(|_| "4".to_string())
        .parse()
        .context("SMS_CODE_LENGTH must be a valid number")?;
    let return_sms_code_in_response = std::env::var("RETURN_SMS_CODE_IN_RESPONSE")
        .unwrap_or_else(|_| "true".to_string())
        .parse()
        .unwrap_or(true);
    let server_host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let server_port = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .context("SERVER_PORT must be a valid number")?;
    let app_apk_path = std::env::var("APP_APK_PATH").ok();

    Ok(BotConfig {
        sms_code_expiration_minutes,
        sms_code_length,
        return_sms_code_in_response,
        server_host,
        server_port,
        app_apk_path,
    })
}

#[derive(Debug, Serialize, Deserialize)]
struct CheckBlockResponse {
    is_blocked: bool,
    block: Option<BlockInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BlockInfo {
    id: String,
    blocked_plate: String,
    created_at: String,
    blocker: BlockerInfo,
}

#[derive(Debug, Serialize, Deserialize)]
struct BlockerInfo {
    name: Option<String>,
    phone: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Пытаемся загрузить .env файл из нескольких возможных мест
    // 1. Из текущей директории (для локальной разработки)
    // 2. Из /opt/rimskiy-service/.env (для production)
    // 3. Из /service/.env (альтернативный путь)
    // 4. Из рабочей директории сервиса (если установлена через WorkingDirectory в systemd)
    let mut env_paths: Vec<String> = vec![
        ".env".to_string(),
        "/opt/rimskiy-service/.env".to_string(),
        "/service/.env".to_string(),
        "/root/service/.env".to_string(),
    ];

    if let Ok(work_dir) = std::env::var("SERVICE_WORK_DIR") {
        env_paths.push(format!("{}/.env", work_dir));
    }

    // Также проверяем путь относительно бинарного файла
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let env_in_exe_dir = exe_dir.join(".env");
            if let Some(env_str) = env_in_exe_dir.to_str() {
                env_paths.push(env_str.to_string());
            }
            // Также проверяем родительскую директорию
            if let Some(parent_dir) = exe_dir.parent() {
                let env_in_parent = parent_dir.join(".env");
                if let Some(env_str) = env_in_parent.to_str() {
                    env_paths.push(env_str.to_string());
                }
            }
        }
    }

    // Инициализируем логирование раньше, чтобы видеть логи загрузки .env
    let default_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&default_filter)),
        )
        .init();

    tracing::info!("Searching for .env file in the following paths:");
    for path in &env_paths {
        tracing::info!("  - {}", path);
    }

    let mut env_loaded = false;
    for env_path in &env_paths {
        if !env_path.is_empty() && std::path::Path::new(env_path).exists() {
            tracing::info!("Found .env file at: {}", env_path);
            if let Err(e) = dotenv::from_path(env_path) {
                tracing::warn!("Failed to load .env from {}: {}", env_path, e);
            } else {
                tracing::info!("✅ Successfully loaded .env from {}", env_path);
                env_loaded = true;
                break;
            }
        } else {
            tracing::debug!(".env file not found at: {}", env_path);
        }
    }

    // Также пытаемся загрузить из стандартного места (текущая директория)
    if !env_loaded {
        tracing::info!("Trying to load .env from current directory...");
        if dotenv::dotenv().is_ok() {
            tracing::info!("✅ Successfully loaded .env from current directory");
            env_loaded = true;
        }
    }

    if !env_loaded {
        tracing::warn!("⚠️  No .env file found in any of the checked paths");
        tracing::warn!("Please ensure .env file exists in one of these locations:");
        for path in &env_paths {
            tracing::warn!("   - {}", path);
        }
    }

    // Логируем информацию о загруженных переменных окружения
    tracing::info!("Checking environment variables...");
    let env_vars_to_check = vec![
        "TELEGRAM_BOT_TOKEN",
        "SMS_API_URL",
        "SMS_API_KEY",
        "API_BASE_URL",
        "SERVER_HOST",
        "SERVER_PORT",
    ];
    for var_name in env_vars_to_check {
        if std::env::var(var_name).is_ok() {
            tracing::info!("✅ {} is set", var_name);
        } else {
            tracing::warn!("⚠️  {} is not set", var_name);
        }
    }

    // Проверяем наличие TELEGRAM_BOT_TOKEN перед загрузкой конфигурации
    let token = std::env::var("TELEGRAM_BOT_TOKEN").context(
        "TELEGRAM_BOT_TOKEN is required. Please set it in .env file or environment variables",
    )?;
    tracing::info!("TELEGRAM_BOT_TOKEN found (length: {})", token.len());

    // Проверяем наличие DATABASE_URL для подключения к БД
    let database_url = std::env::var("DATABASE_URL").context(
        "DATABASE_URL is required for Telegram bot. Please set it in .env file or environment variables",
    )?;
    tracing::info!("DATABASE_URL found");

    // Создаём пул подключений к БД
    let pool = create_pool(&database_url)
        .await
        .context("Failed to create database pool")?;
    tracing::info!("Connected to database");
    let db_pool = Arc::new(pool);

    // Создаём репозитории
    let telegram_bot_repository = Arc::new(PostgresTelegramBotRepository::new(db_pool.clone()));
    let user_repository = Arc::new(PostgresUserRepository::new(db_pool));

    // Загружаем конфигурацию для бота
    let config = Arc::new(load_bot_config()?);

    // Создаём SMS сервис (используем минимальную конфигурацию)
    let sms_config = Config {
        database_url: String::new(), // Не используется ботом
        jwt_secret: String::new(),   // Не используется ботом
        jwt_expiration_minutes: 0,   // Не используется ботом
        encryption_key: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
            .to_string(), // Не используется ботом, но требуется для создания SmsService
        server_host: config.server_host.clone(),
        server_port: config.server_port,
        migrations_path: String::new(), // Не используется ботом
        sms_code_expiration_minutes: config.sms_code_expiration_minutes,
        sms_code_length: config.sms_code_length,
        return_sms_code_in_response: config.return_sms_code_in_response,
        fcm_server_key: None,
        min_client_version: None,
        release_client_version: None,
        app_download_url: None,
        app_apk_path: config.app_apk_path.clone(),
    };
    let sms_service = Arc::new(SmsService::new(sms_config));

    // Получаем базовый URL API сервера
    let api_base_url = std::env::var("API_BASE_URL")
        .unwrap_or_else(|_| format!("http://{}:{}", config.server_host, config.server_port));

    // Определяем путь к APK файлу
    let apk_path = config.app_apk_path.clone().or_else(|| {
        // Пробуем найти APK в стандартных местах
        let default_paths = vec![
            "/opt/rimskiy-service/apk/app-release.apk",
            "/var/www/html/apk/app-release.apk",
            "./android/app/build/outputs/apk/release/app-release.apk",
        ];
        for path in default_paths {
            if std::path::Path::new(path).exists() {
                return Some(path.to_string());
            }
        }
        None
    });

    let bot = Bot::new(token.clone());

    let bot_state = Arc::new(BotState {
        sms_service,
        config,
        http_client: reqwest::Client::new(),
        api_base_url,
        apk_path,
        bot: bot.clone(),
        telegram_bot_repository: telegram_bot_repository.clone(),
        user_repository: user_repository.clone(),
    });

    tracing::info!("Telegram бот запущен");
    tracing::info!("APK путь: {:?}", bot_state.apk_path);
    tracing::info!("API базовый URL: {}", bot_state.api_base_url);

    // Запускаем HTTP сервер для приема запросов на отправку кода
    let bot_state_for_server = bot_state.clone();
    let server_port = bot_state.config.server_port + 1; // Используем следующий порт после основного сервера
    tokio::spawn(async move {
        start_http_server(bot_state_for_server, server_port).await;
    });

    let sms_configured =
        std::env::var("SMS_API_URL").is_ok() && std::env::var("SMS_API_KEY").is_ok();
    tracing::info!(
        "SMS сервис настроен: {}",
        if sms_configured { "да" } else { "нет" }
    );

    let bot_state_clone1 = bot_state.clone();
    let bot_state_clone2 = bot_state.clone();

    let handler = move |bot: Bot, msg: Message, cmd: Command| {
        let state = bot_state_clone1.clone();
        async move {
            // Автоматически сохраняем chat_id при взаимодействии с ботом
            if let Some(user) = msg.from() {
                let telegram_username = user.username.clone();
                let temp_phone_hash = format!("temp_{}", msg.chat.id.0);

                // Проверяем, есть ли уже регистрация для этого chat_id
                if let Ok(None) = state
                    .telegram_bot_repository
                    .find_by_chat_id(msg.chat.id.0)
                    .await
                {
                    // Создаём временную запись
                    let _ = state
                        .telegram_bot_repository
                        .upsert(
                            &temp_phone_hash,
                            msg.chat.id.0,
                            telegram_username.as_deref(),
                            None,
                        )
                        .await;
                    tracing::info!(
                        "Создана временная регистрация для chat_id {} при команде",
                        msg.chat.id.0
                    );
                }
            }

            tracing::info!("Обработка команды {:?} от чата {}", cmd, msg.chat.id);
            message_handler(bot, msg, cmd, (*state).clone()).await
        }
    };

    // Обработчик для текстовых сообщений, начинающихся с /code или /block (если команда не распознана как BotCommand)
    let text_handler = move |bot: Bot, msg: Message| {
        let state = bot_state_clone2.clone();
        async move {
            // Автоматически сохраняем chat_id при любом взаимодействии с ботом
            // Это позволяет автоматически отправлять коды при авторизации
            if let Some(user) = msg.from() {
                let telegram_username = user.username.clone();

                // Проверяем, есть ли уже регистрация для этого chat_id
                if let Ok(Some(bot_user)) = state
                    .telegram_bot_repository
                    .find_by_chat_id(msg.chat.id.0)
                    .await
                {
                    // Обновляем username, если изменился
                    if let Some(username) = &telegram_username {
                        let _ = state
                            .telegram_bot_repository
                            .upsert(
                                &bot_user.phone_hash,
                                msg.chat.id.0,
                                Some(username),
                                bot_user.user_id,
                            )
                            .await;
                    }
                } else {
                    // Если регистрации нет, создаём временную запись без номера телефона
                    // Это позволит автоматически связать номер при авторизации
                    // Используем специальный phone_hash для незарегистрированных пользователей
                    let temp_phone_hash = format!("temp_{}", msg.chat.id.0);
                    let _ = state
                        .telegram_bot_repository
                        .upsert(
                            &temp_phone_hash,
                            msg.chat.id.0,
                            telegram_username.as_deref(),
                            None,
                        )
                        .await;
                    tracing::info!(
                        "Создана временная регистрация для chat_id {} (ожидание привязки номера)",
                        msg.chat.id.0
                    );
                }
            }

            if let Some(text) = msg.text() {
                let trimmed = text.trim();
                tracing::info!(
                    "Получено текстовое сообщение: '{}' от чата {}",
                    trimmed,
                    msg.chat.id
                );
                // Если сообщение начинается с /code, обрабатываем его
                if trimmed.starts_with("/code") {
                    tracing::info!("Обработка /code через text_handler");
                    handle_code_command(&bot, &msg, trimmed, &state).await?;
                } else if trimmed.starts_with("/block") {
                    tracing::info!("Обработка /block через text_handler");
                    handle_block_command(&bot, &msg, trimmed, &state).await?;
                } else if trimmed.starts_with("/apk") {
                    tracing::info!("Обработка /apk через text_handler");
                    handle_apk_command(&bot, &msg, &state).await?;
                }
            }
            Ok(())
        }
    };

    let bot_state_clone3 = bot_state.clone();

    // Обработчик callback query (нажатия на кнопки)
    let callback_handler = move |bot: Bot, q: CallbackQuery| {
        let state = bot_state_clone3.clone();
        async move {
            // Автоматически сохраняем chat_id при взаимодействии с ботом
            if let Some(msg) = q.message.as_ref() {
                let telegram_username = q.from.username.clone();
                let temp_phone_hash = format!("temp_{}", msg.chat.id.0);

                // Проверяем, есть ли уже регистрация для этого chat_id
                if let Ok(None) = state
                    .telegram_bot_repository
                    .find_by_chat_id(msg.chat.id.0)
                    .await
                {
                    // Создаём временную запись
                    let _ = state
                        .telegram_bot_repository
                        .upsert(
                            &temp_phone_hash,
                            msg.chat.id.0,
                            telegram_username.as_deref(),
                            None,
                        )
                        .await;
                    tracing::info!(
                        "Создана временная регистрация для chat_id {} при callback",
                        msg.chat.id.0
                    );
                }
            }

            tracing::info!("Обработка callback query: data = {:?}", q.data);
            if let Some(data) = q.data {
                if let Some(msg) = q.message {
                    match data.as_str() {
                        "get_code" => {
                            bot.answer_callback_query(q.id).await?;
                            bot.send_message(
                                msg.chat.id,
                                "📱 Для получения кода авторизации отправьте команду:\n\n/code <номер телефона>\n\nПример:\n/code +79001234567",
                            )
                            .await?;
                        }
                        "get_app" => {
                            bot.answer_callback_query(q.id).await?;
                            handle_apk_command(&bot, &msg, &state).await?;
                        }
                        _ => {
                            bot.answer_callback_query(q.id).await?;
                        }
                    }
                }
            }
            Ok(())
        }
    };

    Dispatcher::builder(
        bot,
        dptree::entry()
            .branch(
                Update::filter_message()
                    .branch(
                        dptree::entry()
                            .filter_command::<Command>()
                            .endpoint(handler),
                    )
                    .branch(dptree::endpoint(text_handler)),
            )
            .branch(Update::filter_callback_query().endpoint(callback_handler)),
    )
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;

    Ok(())
}

async fn handle_code_command(
    bot: &Bot,
    msg: &Message,
    text: &str,
    state: &BotState,
) -> ResponseResult<()> {
    tracing::info!(
        "Обработка команды /code: текст = '{}', чат = {}",
        text,
        msg.chat.id
    );

    let phone = text.trim_start_matches("/code").trim();
    if phone.is_empty() {
        bot.send_message(
            msg.chat.id,
            "❌ Укажите номер телефона\nПример: /code +79001234567",
        )
        .await?;
        return Ok(());
    }

    // Валидируем и нормализуем телефон
    let normalized_phone = match ValidationService::validate_phone(phone) {
        Ok(phone) => phone,
        Err(e) => {
            let error_msg = format!(
                "❌ Ошибка: Неверный формат номера телефона.\n\
                Используйте формат: +79001234567 или 89001234567\n\
                Ошибка: {}",
                e
            );
            bot.send_message(msg.chat.id, error_msg).await?;
            return Ok(());
        }
    };

    // Вычисляем phone_hash для проверки принадлежности
    let phone_hash = phone_hash(&normalized_phone);

    // Проверяем, что номер зарегистрирован в системе
    let user = match state.user_repository.find_by_phone_hash(&phone_hash).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            let error_msg = format!(
                "❌ Номер телефона {} не зарегистрирован в системе.\n\n\
                📱 Пожалуйста, сначала зарегистрируйтесь в приложении, используя этот номер телефона.",
                normalized_phone
            );
            bot.send_message(msg.chat.id, error_msg).await?;
            return Ok(());
        }
        Err(e) => {
            tracing::error!("Ошибка при проверке номера в БД: {}", e);
            let error_msg = "❌ Ошибка при проверке номера. Попробуйте позже.";
            bot.send_message(msg.chat.id, error_msg).await?;
            return Ok(());
        }
    };

    // Проверяем, есть ли уже регистрация этого chat_id в боте
    let existing_registration = match state
        .telegram_bot_repository
        .find_by_chat_id(msg.chat.id.0)
        .await
    {
        Ok(Some(reg)) => Some(reg),
        Ok(None) => None,
        Err(e) => {
            tracing::error!("Ошибка при проверке регистрации в БД: {}", e);
            let error_msg = "❌ Ошибка при проверке регистрации. Попробуйте позже.";
            bot.send_message(msg.chat.id, error_msg).await?;
            return Ok(());
        }
    };

    // Проверяем, что номер привязан именно к этому пользователю Telegram
    if let Some(existing) = &existing_registration {
        // Если номер отличается от уже зарегистрированного
        if existing.phone_hash != phone_hash {
            // Проверяем, что новый номер принадлежит тому же пользователю
            if let Some(existing_user_id) = existing.user_id {
                if existing_user_id != user.id {
                    // Попытка использовать чужой номер - запрещаем
                    let error_msg = format!(
                        "❌ Номер телефона {} уже привязан к другому аккаунту.\n\n\
                        🔒 Вы уже зарегистрированы с другим номером телефона.\n\n\
                        💡 Для смены номера обратитесь в поддержку.",
                        normalized_phone
                    );
                    bot.send_message(msg.chat.id, error_msg).await?;
                    tracing::warn!(
                        "Попытка использовать чужой номер: chat_id={}, новый номер={}, существующий user_id={}, новый user_id={}",
                        msg.chat.id.0,
                        normalized_phone,
                        existing_user_id,
                        user.id
                    );
                    return Ok(());
                }
                // Номер принадлежит тому же пользователю - разрешаем смену номера
                tracing::info!(
                    "Пользователь {} меняет номер с {} на {}",
                    existing_user_id,
                    existing.phone_hash,
                    phone_hash
                );
            } else {
                // Если у существующей регистрации нет user_id, но номер отличается
                // Проверяем, не зарегистрирован ли этот номер у другого chat_id
                if let Ok(Some(other_reg)) = state
                    .telegram_bot_repository
                    .find_by_phone_hash(&phone_hash)
                    .await
                {
                    if other_reg.chat_id != msg.chat.id.0 {
                        // Этот номер уже используется другим пользователем Telegram
                        let error_msg = format!(
                            "❌ Номер телефона {} уже привязан к другому аккаунту Telegram.\n\n\
                            🔒 Каждый номер может быть привязан только к одному Telegram аккаунту.\n\n\
                            💡 Если это ваш номер, обратитесь в поддержку.",
                            normalized_phone
                        );
                        bot.send_message(msg.chat.id, error_msg).await?;
                        tracing::warn!(
                            "Попытка использовать номер, привязанный к другому chat_id: новый chat_id={}, существующий chat_id={}, номер={}",
                            msg.chat.id.0,
                            other_reg.chat_id,
                            normalized_phone
                        );
                        return Ok(());
                    }
                }
            }
        }
        // Если номер совпадает - всё ОК, это тот же номер
    }

    // Получаем telegram username из сообщения
    let telegram_username = msg.from().and_then(|u| u.username.clone());

    // Сохраняем связь в БД
    match state
        .telegram_bot_repository
        .upsert(
            &phone_hash,
            msg.chat.id.0,
            telegram_username.as_deref(),
            Some(user.id),
        )
        .await
    {
        Ok(_) => {
            tracing::info!(
                "Сохранена связь {} -> chat_id {} -> user_id {} в БД",
                normalized_phone,
                msg.chat.id.0,
                user.id
            );
        }
        Err(e) => {
            tracing::warn!("Не удалось сохранить связь в БД: {}", e);
            let error_msg = "❌ Ошибка при сохранении регистрации. Попробуйте позже.";
            bot.send_message(msg.chat.id, error_msg).await?;
            return Ok(());
        }
    }

    // Автоматически обновляем telegram username в профиле пользователя
    if let Some(ref username) = telegram_username {
        // Обновляем только если username отличается
        if user.telegram.as_deref() != Some(username.as_str()) {
            let update_data = rimskiy_service::repository::UpdateUserData {
                name: None,
                phone_encrypted: None,
                phone_hash: None,
                telegram: Some(username.clone()),
                plate: None,
                show_contacts: None,
                owner_type: None,
                owner_info: None,
                departure_time: None,
                push_token: None,
            };
            if let Err(e) = state.user_repository.update(user.id, &update_data).await {
                tracing::warn!("Не удалось обновить telegram username в профиле: {}", e);
            } else {
                tracing::info!(
                    "Обновлён telegram username в профиле пользователя {}: {}",
                    user.id,
                    username
                );
            }
        }
    }

    // Отправляем сообщение о начале обработки
    let processing_msg = bot
        .send_message(
            msg.chat.id,
            format!(
                "⏳ Генерирую код и отправляю SMS на номер {}...",
                normalized_phone
            ),
        )
        .await?;

    // Генерируем код (это автоматически отправляет SMS)
    match state.sms_service.generate_code(&normalized_phone).await {
        Ok(code) => {
            // Проверяем, настроен ли SMS провайдер
            let sms_configured =
                std::env::var("SMS_API_URL").is_ok() && std::env::var("SMS_API_KEY").is_ok();

            // Удаляем сообщение о процессе и отправляем только код
            let _ = bot.delete_message(msg.chat.id, processing_msg.id).await;
            let message = format!("🔐 Код авторизации: {}", code);
            bot.send_message(msg.chat.id, message).await?;

            tracing::info!(
                "Код авторизации отправлен для {} (чат: {}, SMS настроен: {})",
                normalized_phone,
                msg.chat.id,
                sms_configured
            );
        }
        Err(e) => {
            // Удаляем сообщение о обработке
            let _ = bot.delete_message(msg.chat.id, processing_msg.id).await;

            let error_msg = format!(
                "❌ Ошибка при генерации кода: {}\n\n\
                Попробуйте позже или обратитесь в поддержку.",
                e
            );
            bot.send_message(msg.chat.id, error_msg).await?;
            tracing::error!("Ошибка при генерации кода для {}: {}", normalized_phone, e);
        }
    }
    Ok(())
}

async fn handle_block_command(
    bot: &Bot,
    msg: &Message,
    text: &str,
    state: &BotState,
) -> ResponseResult<()> {
    let plate = text.trim_start_matches("/block").trim();
    if plate.is_empty() {
        bot.send_message(
            msg.chat.id,
            "❌ Укажите номер автомобиля\nПример: /block А123БВ777",
        )
        .await?;
        return Ok(());
    }

    // Валидируем и нормализуем номер
    let normalized_plate = match ValidationService::validate_plate(plate) {
        Ok(plate) => plate,
        Err(e) => {
            let error_msg = format!(
                "❌ Ошибка: Неверный формат номера автомобиля.\n\
                Используйте формат: А123БВ777\n\
                Ошибка: {}",
                e
            );
            bot.send_message(msg.chat.id, error_msg).await?;
            return Ok(());
        }
    };

    // Отправляем сообщение о начале обработки
    let processing_msg = bot
        .send_message(msg.chat.id, "⏳ Проверяю блокировку...")
        .await?;

    // Делаем запрос к API для проверки блокировки
    let check_url = format!(
        "{}/api/blocks/check?plate={}",
        state.api_base_url, normalized_plate
    );
    match state.http_client.get(&check_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<CheckBlockResponse>().await {
                    Ok(check_result) => {
                        let _ = bot.delete_message(msg.chat.id, processing_msg.id).await;

                        if check_result.is_blocked {
                            if let Some(block_info) = check_result.block {
                                let blocker_name =
                                    block_info.blocker.name.as_deref().unwrap_or("Неизвестно");
                                let message = format!(
                                    "🚗 Автомобиль {} заблокирован\n\n\
                                    👤 Блокирующий: {}\n\n\
                                    📅 Дата блокировки: {}\n\n\
                                    📱 Проверьте приложение для подробностей",
                                    normalized_plate, blocker_name, block_info.created_at
                                );
                                bot.send_message(msg.chat.id, message).await?;
                                tracing::info!(
                                    "Проверка блокировки для {} (чат: {}): заблокирован пользователем {}",
                                    normalized_plate,
                                    msg.chat.id,
                                    blocker_name
                                );
                            } else {
                                bot.send_message(
                                    msg.chat.id,
                                    format!("🚗 Автомобиль {} заблокирован", normalized_plate),
                                )
                                .await?;
                            }
                        } else {
                            bot.send_message(
                                msg.chat.id,
                                format!("✅ Автомобиль {} не заблокирован", normalized_plate),
                            )
                            .await?;
                        }
                    }
                    Err(e) => {
                        let _ = bot.delete_message(msg.chat.id, processing_msg.id).await;
                        let error_msg = format!("❌ Ошибка при обработке ответа сервера: {}", e);
                        bot.send_message(msg.chat.id, error_msg).await?;
                        tracing::error!("Ошибка парсинга ответа для {}: {}", normalized_plate, e);
                    }
                }
            } else {
                let _ = bot.delete_message(msg.chat.id, processing_msg.id).await;
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                let error_msg = format!(
                    "❌ Ошибка сервера: {} - {}\n\n\
                    Попробуйте позже или обратитесь в поддержку.",
                    status, error_text
                );
                bot.send_message(msg.chat.id, error_msg).await?;
                tracing::error!(
                    "Ошибка API для {}: {} - {}",
                    normalized_plate,
                    status,
                    error_text
                );
            }
        }
        Err(e) => {
            let _ = bot.delete_message(msg.chat.id, processing_msg.id).await;
            let error_msg = format!(
                "❌ Ошибка при запросе к серверу: {}\n\n\
                Попробуйте позже или обратитесь в поддержку.",
                e
            );
            bot.send_message(msg.chat.id, error_msg).await?;
            tracing::error!("Ошибка запроса для {}: {}", normalized_plate, e);
        }
    }

    Ok(())
}

async fn handle_apk_command(bot: &Bot, msg: &Message, state: &BotState) -> ResponseResult<()> {
    tracing::info!(
        "Обработка команды /apk: чат = {}, APK путь = {:?}",
        msg.chat.id,
        state.apk_path
    );
    // Отправляем сообщение о начале обработки
    let processing_msg = bot
        .send_message(msg.chat.id, "⏳ Загружаю приложение...")
        .await?;

    // Пробуем отправить APK файл напрямую с диска
    let apk_sent = if let Some(apk_path) = &state.apk_path {
        if std::path::Path::new(apk_path).exists() {
            match tokio::fs::read(apk_path).await {
                Ok(apk_data) => {
                    let file_name = std::path::Path::new(apk_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "app-release.apk".to_string());

                    match bot
                        .send_document(
                            msg.chat.id,
                            teloxide::types::InputFile::memory(apk_data).file_name(file_name),
                        )
                        .await
                    {
                        Ok(_) => {
                            let _ = bot.delete_message(msg.chat.id, processing_msg.id).await;
                            bot.send_message(
                                msg.chat.id,
                                "✅ Приложение отправлено. Установите APK файл.",
                            )
                            .await?;
                            true
                        }
                        Err(e) => {
                            tracing::error!("Ошибка при отправке APK: {}", e);
                            false
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Ошибка при чтении APK файла {}: {}", apk_path, e);
                    false
                }
            }
        } else {
            false
        }
    } else {
        false
    };

    if !apk_sent {
        // Если не удалось отправить файл напрямую, пробуем через API
        let download_url = format!("{}/api/app/download", state.api_base_url);
        let _ = bot.delete_message(msg.chat.id, processing_msg.id).await;

        match state.http_client.get(&download_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.bytes().await {
                        Ok(apk_data) => {
                            match bot
                                .send_document(
                                    msg.chat.id,
                                    teloxide::types::InputFile::memory(apk_data.to_vec())
                                        .file_name("app-release.apk"),
                                )
                                .await
                            {
                                Ok(_) => {
                                    // Удаляем сообщение о процессе
                                    let _ =
                                        bot.delete_message(msg.chat.id, processing_msg.id).await;

                                    bot.send_message(
                                        msg.chat.id,
                                        "✅ Приложение отправлено. Установите APK файл.",
                                    )
                                    .await?;
                                }
                                Err(_e) => {
                                    let _ =
                                        bot.delete_message(msg.chat.id, processing_msg.id).await;
                                    bot.send_message(
                                        msg.chat.id,
                                        "❌ Ошибка при отправке приложения.",
                                    )
                                    .await?;
                                }
                            }
                        }
                        Err(e) => {
                            let _ = bot.delete_message(msg.chat.id, processing_msg.id).await;
                            bot.send_message(msg.chat.id, "❌ Ошибка при загрузке приложения.")
                                .await?;
                            tracing::error!("Ошибка при загрузке APK через API: {}", e);
                        }
                    }
                } else {
                    let _ = bot.delete_message(msg.chat.id, processing_msg.id).await;
                    bot.send_message(msg.chat.id, "❌ Приложение не найдено на сервере.")
                        .await?;
                    tracing::warn!("APK файл не найден по URL: {}", download_url);
                }
            }
            Err(e) => {
                let _ = bot.delete_message(msg.chat.id, processing_msg.id).await;
                bot.send_message(msg.chat.id, "❌ Ошибка при запросе к серверу.")
                    .await?;
                tracing::error!("Ошибка запроса APK: {}", e);
            }
        }
    }

    Ok(())
}

async fn message_handler(
    bot: Bot,
    msg: Message,
    cmd: Command,
    state: BotState,
) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            let help_text = format!(
                "🤖 Бот для Rimskiy Service\n\n\
                Доступные команды:\n\
                {}\n\n\
                Примеры использования:\n\
                /code +79001234567 - получить код авторизации\n\
                /block А123БВ777 - проверить блокировку автомобиля\n\
                /apk - получить последнюю версию приложения",
                Command::descriptions()
            );

            // Создаем inline клавиатуру с кнопками
            let keyboard = teloxide::types::InlineKeyboardMarkup::new(vec![
                vec![teloxide::types::InlineKeyboardButton::callback(
                    "📱 Получить код авторизации",
                    "get_code",
                )],
                vec![teloxide::types::InlineKeyboardButton::callback(
                    "📲 Получить приложение",
                    "get_app",
                )],
            ]);

            bot.send_message(msg.chat.id, help_text)
                .reply_markup(keyboard)
                .await?;
        }
        Command::Code => {
            let text = msg.text().unwrap_or("");
            handle_code_command(&bot, &msg, text, &state).await?;
        }
        Command::Block => {
            let text = msg.text().unwrap_or("");
            handle_block_command(&bot, &msg, text, &state).await?;
        }
        Command::Apk => {
            handle_apk_command(&bot, &msg, &state).await?;
        }
    }
    Ok(())
}

// HTTP сервер для приема запросов на отправку кода
async fn start_http_server(state: Arc<BotState>, port: u16) {
    let app = Router::new()
        .route("/send_code", post(send_code_handler))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("HTTP сервер бота запущен на {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize)]
struct SendCodeRequest {
    phone: String,
    code: String,
}

async fn send_code_handler(
    State(state): State<Arc<BotState>>,
    Json(payload): Json<SendCodeRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    tracing::info!(
        "🔔 Получен запрос на отправку кода для {} (код: {})",
        payload.phone,
        payload.code
    );

    // Нормализуем номер телефона
    let normalized_phone = match ValidationService::validate_phone(&payload.phone) {
        Ok(phone) => phone,
        Err(e) => {
            tracing::warn!("Неверный формат номера телефона {}: {}", payload.phone, e);
            return Ok(Json(serde_json::json!({
                "success": false,
                "error": format!("Неверный формат номера телефона: {}", e),
                "sent_count": 0
            })));
        }
    };

    // Вычисляем phone_hash
    let phone_hash = phone_hash(&normalized_phone);

    // Пытаемся найти пользователя в системе (может не существовать при первой авторизации)
    let user = state
        .user_repository
        .find_by_phone_hash(&phone_hash)
        .await
        .ok()
        .flatten();
    if user.is_some() {
        tracing::info!(
            "✅ Пользователь найден в системе по phone_hash: {}",
            phone_hash
        );
    } else {
        tracing::info!("⚠️ Пользователь не найден в системе по phone_hash: {} (это нормально при первой авторизации)", phone_hash);
    }

    // Находим chat_id по phone_hash в БД
    let bot_user_by_phone = state
        .telegram_bot_repository
        .find_by_phone_hash(&phone_hash)
        .await
        .ok()
        .flatten();

    if bot_user_by_phone.is_some() {
        tracing::info!("✅ Найден chat_id по phone_hash в telegram_bot_users");
    } else {
        tracing::info!(
            "⚠️ Не найден chat_id по phone_hash в telegram_bot_users, ищем другими способами..."
        );
    }

    match bot_user_by_phone {
        Some(bot_user) => {
            // Дополнительная проверка: если пользователь существует, убеждаемся, что номер привязан к правильному user_id
            if let Some(ref user_data) = user {
                if let Some(registered_user_id) = bot_user.user_id {
                    if registered_user_id != user_data.id {
                        tracing::warn!(
                            "Попытка отправить код на номер, привязанный к другому пользователю: номер={}, зарегистрированный user_id={}, текущий user_id={}",
                            normalized_phone,
                            registered_user_id,
                            user_data.id
                        );
                        return Ok(Json(serde_json::json!({
                            "success": false,
                            "error": "Номер привязан к другому пользователю",
                            "sent_count": 0
                        })));
                    }
                }
            }

            let chat = teloxide::types::ChatId(bot_user.chat_id);
            let message = format!("🔐 Код авторизации: {}", payload.code);

            match state.bot.send_message(chat, message).await {
                Ok(_) => {
                    tracing::info!(
                        "Код отправлен в Telegram для {} (chat_id: {})",
                        normalized_phone,
                        bot_user.chat_id
                    );
                    Ok(Json(serde_json::json!({
                        "success": true,
                        "sent_count": 1
                    })))
                }
                Err(e) => {
                    tracing::warn!("Не удалось отправить код в чат {}: {}", bot_user.chat_id, e);
                    Ok(Json(serde_json::json!({
                        "success": false,
                        "error": format!("Не удалось отправить сообщение: {}", e),
                        "sent_count": 0
                    })))
                }
            }
        }
        None => {
            // Пытаемся найти пользователя по Telegram username
            // Это позволяет отправлять код даже если пользователь не взаимодействовал с ботом
            let mut found_chat_id: Option<i64> = None;

            // Сначала пытаемся найти пользователя по username в таблице users
            // Затем ищем его в telegram_bot_users
            let telegram_username_to_search = user
                .as_ref()
                .and_then(|u| u.telegram.as_ref())
                .map(|s| s.as_str());

            if let Some(telegram_username) = telegram_username_to_search {
                tracing::info!(
                    "Пытаемся найти пользователя по Telegram username: {}",
                    telegram_username
                );

                // Сначала ищем в telegram_bot_users по username
                match state
                    .telegram_bot_repository
                    .find_by_telegram_username(telegram_username)
                    .await
                {
                    Ok(Some(bot_user)) => {
                        // Найден пользователь по username в telegram_bot_users - используем его chat_id
                        tracing::info!(
                            "Найден пользователь по Telegram username {} в telegram_bot_users (chat_id: {})",
                            telegram_username,
                            bot_user.chat_id
                        );
                        found_chat_id = Some(bot_user.chat_id);

                        // Автоматически привязываем номер к найденному chat_id
                        let user_id = user.as_ref().map(|u| u.id).or(bot_user.user_id);
                        let _ = state
                            .telegram_bot_repository
                            .upsert(
                                &phone_hash,
                                bot_user.chat_id,
                                Some(telegram_username),
                                user_id,
                            )
                            .await;
                        tracing::info!(
                            "Номер {} автоматически привязан к chat_id {} по Telegram username",
                            normalized_phone,
                            bot_user.chat_id
                        );
                    }
                    Ok(None) => {
                        // Не найден в telegram_bot_users, попробуем найти в users и затем в telegram_bot_users по user_id
                        tracing::info!(
                            "Не найден в telegram_bot_users, ищем в users по username: {}",
                            telegram_username
                        );
                        if let Ok(Some(user_by_telegram)) = state
                            .user_repository
                            .find_by_telegram(telegram_username)
                            .await
                        {
                            // Найден пользователь в users, ищем его в telegram_bot_users по user_id
                            if let Ok(Some(bot_user_by_id)) = state
                                .telegram_bot_repository
                                .find_by_phone_hash(&format!("temp_{}", user_by_telegram.id))
                                .await
                            {
                                // Найден по временной записи
                                found_chat_id = Some(bot_user_by_id.chat_id);
                                tracing::info!(
                                    "Найден пользователь по user_id {} в telegram_bot_users (chat_id: {})",
                                    user_by_telegram.id,
                                    bot_user_by_id.chat_id
                                );
                            } else {
                                // Ищем все записи с таким user_id
                                if let Ok(bot_users) = state
                                    .telegram_bot_repository
                                    .find_temp_by_user_id(user_by_telegram.id)
                                    .await
                                {
                                    if let Some(first_bot_user) = bot_users.first() {
                                        found_chat_id = Some(first_bot_user.chat_id);
                                        tracing::info!(
                                            "Найден пользователь по временной записи user_id {} (chat_id: {})",
                                            user_by_telegram.id,
                                            first_bot_user.chat_id
                                        );
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Ошибка при поиске по Telegram username: {}", e);
                    }
                }
            }

            // Если нашли chat_id по username, отправляем код
            if let Some(chat_id) = found_chat_id {
                let chat = teloxide::types::ChatId(chat_id);
                let message = format!("🔐 Код авторизации: {}", payload.code);

                match state.bot.send_message(chat, message).await {
                    Ok(_) => {
                        tracing::info!(
                            "Код автоматически отправлен в Telegram для {} (chat_id: {}) по username",
                            normalized_phone,
                            chat_id
                        );
                        return Ok(Json(serde_json::json!({
                            "success": true,
                            "sent_count": 1,
                            "auto_registered": true
                        })));
                    }
                    Err(e) => {
                        tracing::warn!("Не удалось отправить код в чат {}: {}", chat_id, e);
                        // Продолжаем поиск других способов
                    }
                }
            }

            // Пытаемся найти незарегистрированные записи (временные) для этого user_id
            // и автоматически привязать номер
            if let Some(ref user_data) = user {
                tracing::info!(
                    "Не найден chat_id для номера {} (phone_hash: {}). Пытаемся найти незарегистрированные записи для user_id {}",
                    normalized_phone,
                    phone_hash,
                    user_data.id
                );

                // Ищем временные записи для этого user_id
                match state
                    .telegram_bot_repository
                    .find_temp_by_user_id(user_data.id)
                    .await
                {
                    Ok(registrations) if !registrations.is_empty() => {
                        // Найдены временные записи - автоматически привязываем номер к первому найденному chat_id
                        let first_reg = &registrations[0];
                        tracing::info!("Найдена временная регистрация для user_id {} (chat_id: {}). Автоматически привязываем номер {}", user_data.id, first_reg.chat_id, normalized_phone);

                        // Обновляем запись, заменяя временный phone_hash на реальный
                        match state
                            .telegram_bot_repository
                            .update_phone_hash(
                                &first_reg.phone_hash,
                                &phone_hash,
                                first_reg.chat_id,
                            )
                            .await
                        {
                            Ok(Some(updated_reg)) => {
                                // Удаляем другие временные записи для этого user_id
                                let _ = state
                                    .telegram_bot_repository
                                    .delete_temp_except(user_data.id, updated_reg.id)
                                    .await;

                                // Обновляем user_id в новой записи
                                let _ = state
                                    .telegram_bot_repository
                                    .update_user_id(&phone_hash, updated_reg.chat_id, user_data.id)
                                    .await;

                                // Отправляем код в Telegram
                                let chat = teloxide::types::ChatId(updated_reg.chat_id);
                                let message = format!("🔐 Код авторизации: {}", payload.code);

                                match state.bot.send_message(chat, message).await {
                                    Ok(_) => {
                                        tracing::info!("Код автоматически отправлен в Telegram для {} (chat_id: {}, user_id: {})", normalized_phone, updated_reg.chat_id, user_data.id);
                                        Ok(Json(serde_json::json!({
                                            "success": true,
                                            "sent_count": 1,
                                            "auto_registered": true
                                        })))
                                    }
                                    Err(_e) => {
                                        tracing::warn!(
                                            "Не удалось отправить код в чат {}: {}",
                                            updated_reg.chat_id,
                                            e
                                        );
                                        Ok(Json(serde_json::json!({
                                            "success": false,
                                            "error": format!("Не удалось отправить сообщение: {}", e),
                                            "sent_count": 0
                                        })))
                                    }
                                }
                            }
                            Ok(None) => {
                                tracing::warn!("Не удалось обновить временную регистрацию");
                                Ok(Json(serde_json::json!({
                                    "success": false,
                                    "error": "Не удалось привязать номер",
                                    "sent_count": 0
                                })))
                            }
                            Err(e) => {
                                tracing::error!(
                                    "Ошибка при обновлении временной регистрации: {}",
                                    e
                                );
                                Ok(Json(serde_json::json!({
                                    "success": false,
                                    "error": format!("Ошибка базы данных: {}", e),
                                    "sent_count": 0
                                })))
                            }
                        }
                    }
                    Ok(_) => {
                        // Временных записей нет - пользователь еще не взаимодействовал с ботом
                        tracing::info!("Пользователь с номером {} еще не взаимодействовал с ботом. Код будет отправлен только по SMS.", normalized_phone);
                        Ok(Json(serde_json::json!({
                            "success": false,
                            "error": "Пользователь еще не взаимодействовал с ботом. Код отправлен по SMS.",
                            "sent_count": 0,
                            "sms_sent": true
                        })))
                    }
                    Err(e) => {
                        tracing::error!("Ошибка при поиске временных регистраций: {}", e);
                        Ok(Json(serde_json::json!({
                            "success": false,
                            "error": format!("Ошибка базы данных: {}", e),
                            "sent_count": 0
                        })))
                    }
                }
            } else {
                // Пользователь не найден в системе и нет временных записей
                tracing::info!("Пользователь с номером {} не найден в системе и не зарегистрирован в боте. Код будет отправлен только по SMS.", normalized_phone);
                Ok(Json(serde_json::json!({
                    "success": false,
                    "error": "Пользователь не зарегистрирован в боте. Код отправлен по SMS.",
                    "sent_count": 0,
                    "sms_sent": true
                })))
            }
        }
    }
}
