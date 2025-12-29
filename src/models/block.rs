use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use serde_json::json;
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::utils::normalize_plate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Block {
    /// ID блокировки
    #[schema(value_type = String, format = "uuid", example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: Uuid,
    /// ID пользователя, который создал блокировку
    #[schema(value_type = String, format = "uuid", example = "550e8400-e29b-41d4-a716-446655440000")]
    pub blocker_id: Uuid,
    /// Номер авто блокирующего (для совместных владельцев)
    #[schema(example = "А777ВС178")]
    pub blocker_plate: String,
    /// Номер заблокированного автомобиля
    #[schema(example = "А123БВ777")]
    pub blocked_plate: String,
    /// Дата создания блокировки
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[schema(example = json!({
    "blocked_plate": "А123БВ777",
    "notify_owner": true,
    "departure_time": "18:30",
    "notification_method": "android_push"
}))]
pub struct CreateBlockRequest {
    /// Номер автомобиля, который блокируется
    #[validate(length(
        min = 8,
        max = 9,
        message = "Номер автомобиля должен быть от 8 до 9 символов"
    ))]
    #[schema(example = "А123БВ777")]
    pub blocked_plate: String,
    /// Уведомить ли владельца (позвонить)
    #[serde(default)]
    #[schema(example = true)]
    pub notify_owner: bool,
    /// Время, когда блокирующий планирует уехать (HH:MM), чтобы привязать к его авто
    #[serde(default)]
    #[schema(example = "18:30")]
    pub departure_time: Option<String>,
    /// Способ отправки уведомлений: "android_push" или "telegram"
    #[serde(default)]
    #[schema(example = "android_push")]
    pub notification_method: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BlockResponse {
    #[schema(value_type = String, format = "uuid")]
    pub id: Uuid,
    #[schema(value_type = String, format = "uuid")]
    pub blocker_id: Uuid,
    pub blocked_plate: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BlockWithBlockerInfo {
    /// ID блокировки
    #[schema(value_type = String, format = "uuid", example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: Uuid,
    /// Номер заблокированного автомобиля
    #[schema(example = "А123БВ777")]
    pub blocked_plate: String,
    /// Дата создания
    pub created_at: DateTime<Utc>,
    /// Информация о блокирующем пользователе
    pub blocker: crate::models::user::PublicUserInfo,
    /// Тип владельца блокирующего
    #[schema(example = "renter")]
    pub blocker_owner_type: Option<String>,
    /// Информация о собственнике блокирующего
    pub blocker_owner_info: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CheckBlockResponse {
    /// Заблокирована ли машина
    #[schema(example = true)]
    pub is_blocked: bool,
    /// Информация о блокировке (если заблокирована)
    pub block: Option<BlockWithBlockerInfo>,
}

impl CreateBlockRequest {
    pub fn normalize(&mut self) {
        self.blocked_plate = normalize_plate(&self.blocked_plate);
    }
}
