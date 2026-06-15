use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use serde_json::json;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[schema(example = json!({"phone": "+79165180900"}))]
pub struct AuthStartRequest {
    #[validate(length(min = 10, max = 15))]
    #[schema(example = "+79165180900")]
    pub phone: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthStartResponse {
    #[schema(example = "")]
    pub code: String,
    #[schema(example = 600)]
    pub expires_in: u64,
    #[schema(example = "your_bot_username")]
    pub telegram_bot_username: Option<String>,
    #[schema(example = "https://t.me/your_bot_username?start=p79001234567")]
    pub telegram_deeplink: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[schema(example = json!({"phone": "+79165180900", "code": "1234"}))]
pub struct AuthVerifyRequest {
    #[validate(length(min = 10, max = 15))]
    #[schema(example = "+79165180900")]
    pub phone: String,
    #[validate(length(min = 4, max = 6))]
    #[schema(example = "1234")]
    pub code: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthVerifyResponse {
    /// Access JWT (короткий TTL)
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub token: String,
    /// Refresh JWT (длинный TTL)
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub refresh_token: String,
    #[schema(value_type = String, format = "uuid", example = "550e8400-e29b-41d4-a716-446655440000")]
    pub user_id: uuid::Uuid,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PinAuthRequest {
    pub pin: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PinAuthResponse {
    pub token: String,
    #[schema(value_type = String, format = "uuid")]
    pub user_id: uuid::Uuid,
}

#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({"token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."}))]
pub struct RefreshTokenRequest {
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    #[serde(default)]
    pub token: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LogoutRequest {
    #[serde(default)]
    pub token: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RefreshTokenResponse {
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub token: String,
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub refresh_token: String,
    #[schema(value_type = String, format = "uuid", example = "550e8400-e29b-41d4-a716-446655440000")]
    pub user_id: uuid::Uuid,
}
