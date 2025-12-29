use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use serde_json::json;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[schema(example = json!({"phone": "+79165180900"}))]
pub struct AuthStartRequest {
    /// Номер телефона в формате +7XXXXXXXXXX
    #[validate(length(min = 10, max = 15))]
    #[schema(example = "+79165180900")]
    pub phone: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthStartResponse {
    /// SMS код для подтверждения
    #[schema(example = "1234")]
    pub code: String,
    /// Время жизни кода в секундах
    #[schema(example = 600)]
    pub expires_in: u64,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[schema(example = json!({"phone": "+79165180900", "code": "1234"}))]
pub struct AuthVerifyRequest {
    /// Номер телефона в формате +7XXXXXXXXXX
    #[validate(length(min = 10, max = 15))]
    #[schema(example = "+79165180900")]
    pub phone: String,
    /// SMS код подтверждения
    #[validate(length(min = 4, max = 6))]
    #[schema(example = "1234")]
    pub code: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthVerifyResponse {
    /// JWT токен для авторизации
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub token: String,
    /// ID пользователя
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
    /// Текущий JWT токен
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub token: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RefreshTokenResponse {
    /// Новый JWT токен
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub token: String,
    /// ID пользователя
    #[schema(value_type = String, format = "uuid", example = "550e8400-e29b-41d4-a716-446655440000")]
    pub user_id: uuid::Uuid,
}
