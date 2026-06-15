pub mod auth;
pub mod block;
pub mod health;
pub mod notification;
pub mod ocr;
pub mod server_info;
pub mod user;
pub mod user_plate;

pub use auth::*;
pub use block::*;
pub use health::*;
pub use notification::*;
pub use ocr::*;
pub use server_info::*;
pub use user::*;
pub use user_plate::*;

use std::sync::Arc;

use crate::auth::sms::SmsService;
use crate::config::Config;
use crate::db::DbPool;
use crate::queue::EventPublisher;
use crate::redis::RedisClient;
use crate::repository::{
    PostgresBlockRepository, PostgresNotificationRepository, PostgresTelegramBotRepository,
    PostgresUserPlateRepository, PostgresUserRepository,
};
use crate::service::{
    AuthService, BlockService, PushService, TelegramService, TelephonyService, UserService,
};
use crate::utils::encryption::Encryption;
use reqwest::Client;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db_pool: DbPool,
    pub redis: Option<RedisClient>,
    pub event_publisher: Arc<dyn EventPublisher>,
    pub http_client: Client,
    pub encryption: Encryption,
    pub sms_service: SmsService,
    pub telephony_service: TelephonyService,
    pub telegram_service: TelegramService,
    pub push_service: PushService,
    pub auth_service: AuthService,
    pub user_service: UserService,
    pub block_service: BlockService,
    pub user_repository: PostgresUserRepository,
    pub block_repository: PostgresBlockRepository,
    pub user_plate_repository: PostgresUserPlateRepository,
    pub notification_repository: PostgresNotificationRepository,
    pub telegram_bot_repository: PostgresTelegramBotRepository,
}
