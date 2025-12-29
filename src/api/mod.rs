pub mod auth;
pub mod user;
pub mod block;
pub mod server_info;
pub mod ocr;
pub mod user_plate;
pub mod notification;

pub use auth::*;
pub use user::*;
pub use block::*;
pub use server_info::*;
pub use ocr::*;
pub use user_plate::*;
pub use notification::*;

use crate::auth::sms::SmsService;
use crate::config::Config;
use crate::repository::{PostgresUserRepository, PostgresBlockRepository, PostgresUserPlateRepository, PostgresNotificationRepository};
use crate::service::{AuthService, UserService, BlockService, TelephonyService, PushService};
use crate::utils::encryption::Encryption;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub encryption: Encryption,
    pub sms_service: SmsService,
    pub telephony_service: TelephonyService,
    pub push_service: PushService,
    pub auth_service: AuthService,
    pub user_service: UserService,
    pub block_service: BlockService,
    pub user_repository: PostgresUserRepository,
    pub block_repository: PostgresBlockRepository,
    pub user_plate_repository: PostgresUserPlateRepository,
    pub notification_repository: PostgresNotificationRepository,
}

