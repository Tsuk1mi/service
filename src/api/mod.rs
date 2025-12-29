pub mod app_download;
pub mod auth;
pub mod block;
pub mod notification;
pub mod ocr;
pub mod server_info;
pub mod user;
pub mod user_plate;

pub use app_download::*;
pub use auth::*;
pub use block::*;
pub use notification::*;
pub use ocr::*;
pub use server_info::*;
pub use user::*;
pub use user_plate::*;

use crate::auth::sms::SmsService;
use crate::config::Config;
use crate::repository::{
    PostgresBlockRepository, PostgresNotificationRepository, PostgresUserPlateRepository,
    PostgresUserRepository,
};
use crate::service::{AuthService, BlockService, PushService, TelephonyService, UserService};
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
