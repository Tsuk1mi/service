use std::env;

use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "Команды бота",
    parse_with = "split"
)]
enum Command {
    #[command(description = "Показать справку")]
    Help,
    #[command(description = "Отправить код: /code <телефон> <код>")]
    Code { phone: String, code: String },
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN is required");
    let bot = Bot::new(token);

    teloxide::repl(bot, handler).await;
}

async fn handler(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::Code { phone, code } => {
            let text = format!("Код для авторизации\nТелефон: {}\nКод: {}", phone, code);
            bot.send_message(msg.chat.id, text).await?;
        }
    }
    Ok(())
}
