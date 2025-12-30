use anyhow::Context;
use rimskiy_service::auth::sms::SmsService;
use rimskiy_service::config::Config;
use rimskiy_service::service::validation_service::ValidationService;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
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
    // 3. Из рабочей директории сервиса (если установлена через WorkingDirectory в systemd)
    let mut env_paths: Vec<String> =
        vec![".env".to_string(), "/opt/rimskiy-service/.env".to_string()];

    if let Ok(work_dir) = std::env::var("SERVICE_WORK_DIR") {
        env_paths.push(format!("{}/.env", work_dir));
    }

    for env_path in env_paths {
        if !env_path.is_empty() && std::path::Path::new(&env_path).exists() {
            if let Err(e) = dotenv::from_path(&env_path) {
                tracing::warn!("Failed to load .env from {}: {}", env_path, e);
            } else {
                tracing::info!("Loaded .env from {}", env_path);
                break;
            }
        }
    }

    // Также пытаемся загрузить из стандартного места
    dotenv::dotenv().ok();

    // Инициализируем логирование
    let default_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&default_filter)),
        )
        .init();

    // Загружаем конфигурацию для бота (не требует DATABASE_URL и других полей)
    let config = Arc::new(load_bot_config()?);

    // Создаём SMS сервис (используем минимальную конфигурацию)
    let sms_config = Config {
        database_url: String::new(), // Не используется ботом
        jwt_secret: String::new(),   // Не используется ботом
        jwt_expiration_minutes: 0,   // Не используется ботом
        encryption_key: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(), // Не используется ботом, но требуется для создания SmsService
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

    let bot_state = Arc::new(BotState {
        sms_service,
        config,
        http_client: reqwest::Client::new(),
        api_base_url,
        apk_path,
    });

    let token = std::env::var("TELEGRAM_BOT_TOKEN").context("TELEGRAM_BOT_TOKEN is required")?;
    let bot = Bot::new(token);

    tracing::info!("Telegram бот запущен");
    tracing::info!("APK путь: {:?}", bot_state.apk_path);
    tracing::info!("API базовый URL: {}", bot_state.api_base_url);
    let sms_configured = std::env::var("SMS_API_URL").is_ok() && std::env::var("SMS_API_KEY").is_ok();
    tracing::info!("SMS сервис настроен: {}", if sms_configured { "да" } else { "нет" });

    let bot_state_clone1 = bot_state.clone();
    let bot_state_clone2 = bot_state.clone();

    let handler = move |bot: Bot, msg: Message, cmd: Command| {
        let state = bot_state_clone1.clone();
        async move {
            tracing::info!("Обработка команды {:?} от чата {}", cmd, msg.chat.id);
            message_handler(bot, msg, cmd, (*state).clone()).await
        }
    };

    // Обработчик для текстовых сообщений, начинающихся с /code или /block (если команда не распознана как BotCommand)
    let text_handler = move |bot: Bot, msg: Message| {
        let state = bot_state_clone2.clone();
        async move {
            if let Some(text) = msg.text() {
                let trimmed = text.trim();
                tracing::info!("Получено текстовое сообщение: '{}' от чата {}", trimmed, msg.chat.id);
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
    tracing::info!("Обработка команды /code: текст = '{}', чат = {}", text, msg.chat.id);
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

            // Формируем сообщение в зависимости от того, отправлено ли SMS
            let message = if sms_configured {
                format!(
                    "✅ Код авторизации для {}\n\n\
                    📱 SMS отправлено на номер {}\n\n\
                    🔐 Ваш код: {}\n\n\
                    ⏰ Код действителен {} минут\n\n\
                    💬 Код также отправлен в этом сообщении для удобства\n\n\
                    📲 Введите этот код в приложении для завершения авторизации.",
                    normalized_phone,
                    normalized_phone,
                    code,
                    state.config.sms_code_expiration_minutes
                )
            } else {
                format!(
                    "✅ Код авторизации для {}\n\n\
                    ⚠️ SMS провайдер не настроен, код отправлен только в Telegram\n\n\
                    🔐 Ваш код: {}\n\n\
                    ⏰ Код действителен {} минут\n\n\
                    📲 Введите этот код в приложении для завершения авторизации.\n\n\
                    💡 Для настройки автоматической отправки SMS укажите SMS_API_URL и SMS_API_KEY в .env",
                    normalized_phone, code, state.config.sms_code_expiration_minutes
                )
            };

            // Удаляем сообщение о обработке и отправляем финальное сообщение
            let _ = bot.delete_message(msg.chat.id, processing_msg.id).await;
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
        .send_message(
            msg.chat.id,
            format!("⏳ Проверяю блокировку для номера {}...", normalized_plate),
        )
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
    tracing::info!("Обработка команды /apk: чат = {}, APK путь = {:?}", msg.chat.id, state.apk_path);
    // Отправляем сообщение о начале обработки
    let processing_msg = bot
        .send_message(msg.chat.id, "⏳ Загружаю последнюю версию приложения...")
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

                    let file_name_clone = file_name.clone();
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
                                format!(
                                    "✅ Приложение успешно отправлено!\n\n\
                                    📱 Файл: {}\n\n\
                                    💡 Установите APK файл на ваше Android устройство.\n\n\
                                    ⚠️ Если установка не запускается автоматически, разрешите установку из неизвестных источников в настройках безопасности.",
                                    file_name_clone
                                ),
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
                                    bot.send_message(
                                        msg.chat.id,
                                        "✅ Приложение успешно отправлено!\n\n\
                                        📱 Файл: app-release.apk\n\n\
                                        💡 Установите APK файл на ваше Android устройство.\n\n\
                                        ⚠️ Если установка не запускается автоматически, разрешите установку из неизвестных источников в настройках безопасности.",
                                    )
                                    .await?;
                                }
                                Err(e) => {
                                    let error_msg = format!(
                                        "❌ Ошибка при отправке APK файла: {}\n\n\
                                        Попробуйте скачать приложение по ссылке:\n{}",
                                        e, download_url
                                    );
                                    bot.send_message(msg.chat.id, error_msg).await?;
                                }
                            }
                        }
                        Err(e) => {
                            let error_msg = format!(
                                "❌ Ошибка при загрузке APK файла: {}\n\n\
                                Попробуйте позже или обратитесь в поддержку.",
                                e
                            );
                            bot.send_message(msg.chat.id, error_msg).await?;
                            tracing::error!("Ошибка при загрузке APK через API: {}", e);
                        }
                    }
                } else {
                    let error_msg = format!(
                        "❌ APK файл не найден на сервере.\n\n\
                        Попробуйте позже или обратитесь в поддержку.\n\n\
                        URL: {}",
                        download_url
                    );
                    bot.send_message(msg.chat.id, error_msg).await?;
                    tracing::warn!("APK файл не найден по URL: {}", download_url);
                }
            }
            Err(e) => {
                let error_msg = format!(
                    "❌ Ошибка при запросе к серверу: {}\n\n\
                    Попробуйте позже или обратитесь в поддержку.",
                    e
                );
                bot.send_message(msg.chat.id, error_msg).await?;
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
