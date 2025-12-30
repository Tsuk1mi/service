use anyhow::Context;
use rimskiy_service::auth::sms::SmsService;
use rimskiy_service::config::Config;
use rimskiy_service::service::validation_service::ValidationService;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "Команды бота для авторизации"
)]
enum Command {
    #[command(description = "Показать справку")]
    Help,
    #[command(description = "Запросить код авторизации: /code <телефон>")]
    Code,
}

#[derive(Clone)]
struct BotState {
    sms_service: Arc<SmsService>,
    config: Arc<Config>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    // Инициализируем логирование
    let default_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&default_filter)),
        )
        .init();

    // Загружаем конфигурацию
    let config = Arc::new(Config::from_env()?);

    // Создаём SMS сервис
    let sms_service = Arc::new(SmsService::new((*config).clone()));

    let bot_state = Arc::new(BotState {
        sms_service,
        config,
    });

    let token = std::env::var("TELEGRAM_BOT_TOKEN").context("TELEGRAM_BOT_TOKEN is required")?;
    let bot = Bot::new(token);

    tracing::info!("Telegram бот запущен");

    let handler = move |bot: Bot, msg: Message, cmd: Command| {
        let state = bot_state.clone();
        async move { message_handler(bot, msg, cmd, (*state).clone()).await }
    };

    Dispatcher::builder(
        bot,
        Update::filter_message().branch(dptree::endpoint(handler)),
    )
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;

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
                "🤖 Бот для авторизации Rimskiy Service\n\n\
                Доступные команды:\n\
                {}\n\n\
                Пример использования:\n\
                /code +79001234567",
                Command::descriptions()
            );
            bot.send_message(msg.chat.id, help_text).await?;
        }
        Command::Code => {
            // Получаем текст сообщения
            let text = msg.text().unwrap_or("");

            // Парсим команду: /code <телефон>
            let phone = if text.starts_with("/code") {
                text.trim_start_matches("/code").trim()
            } else {
                bot.send_message(
                    msg.chat.id,
                    "❌ Используйте команду: /code <телефон>\nПример: /code +79001234567",
                )
                .await?;
                return Ok(());
            };

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
                    let sms_configured = std::env::var("SMS_API_URL").is_ok()
                        && std::env::var("SMS_API_KEY").is_ok();

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
        }
    }
    Ok(())
}
