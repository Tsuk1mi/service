pub mod block_repository;
pub mod notification_repository;
pub mod user_plate_repository;
pub mod user_repository;

pub use block_repository::{BlockRepository, PostgresBlockRepository};
pub use notification_repository::{
    CreateNotificationData, NotificationRepository, PostgresNotificationRepository,
};
pub use user_plate_repository::{PostgresUserPlateRepository, UserPlateRepository};
pub use user_repository::{CreateUserData, PostgresUserRepository, UpdateUserData, UserRepository};
