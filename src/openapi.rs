use utoipa::OpenApi;

use crate::models::{
    auth::{
        AuthStartRequest, AuthStartResponse, AuthVerifyRequest, AuthVerifyResponse,
        LogoutRequest, RefreshTokenRequest, RefreshTokenResponse,
    },
    block::{Block, BlockWithBlockerInfo, CheckBlockResponse, CreateBlockRequest},
    notification::NotificationResponse,
    user::{PublicUserInfo, UpdateUserRequest, UserResponse},
    user_plate::{CreateUserPlateRequest, UserPlateResponse},
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Rimskiy Service API",
        description = "API для веб-приложения управления перекрытыми автомобилями",
        version = "1.0.0",
        contact(
            name = "Rimskiy Service",
            email = "support@rimskiy.ru"
        )
    ),
    servers(
        (url = "http://localhost:8080", description = "Локальный сервер разработки"),
    ),
    paths(
        crate::api::auth::start_auth,
        crate::api::auth::verify_auth,
        crate::api::auth::refresh_token,
        crate::api::auth::logout,
        crate::api::user::get_profile,
        crate::api::user::update_profile,
        crate::api::user::get_user_by_plate,
        crate::api::block::create_block,
        crate::api::block::get_my_blocks,
        crate::api::block::get_blocks_for_my_plate,
        crate::api::block::check_block,
        crate::api::block::delete_block,
        crate::api::block::warn_owner,
        crate::api::notification::get_notifications,
        crate::api::notification::mark_notification_read,
        crate::api::notification::mark_all_read,
        crate::api::user_plate::create_user_plate,
        crate::api::user_plate::get_user_plates,
        crate::api::ocr::recognize_plate_auth,
    ),
    components(schemas(
        AuthStartRequest,
        AuthStartResponse,
        AuthVerifyRequest,
        AuthVerifyResponse,
        RefreshTokenRequest,
        RefreshTokenResponse,
        LogoutRequest,
        UserResponse,
        UpdateUserRequest,
        PublicUserInfo,
        Block,
        CreateBlockRequest,
        BlockWithBlockerInfo,
        CheckBlockResponse,
        NotificationResponse,
        UserPlateResponse,
        CreateUserPlateRequest,
    )),
    tags(
        (name = "auth", description = "API для аутентификации пользователей"),
        (name = "users", description = "API для управления профилем пользователя"),
        (name = "blocks", description = "API для управления блокировками автомобилями"),
        (name = "notifications", description = "API для работы с уведомлениями"),
        (name = "user_plates", description = "API для управления автомобилями пользователя"),
        (name = "ocr", description = "API распознавания номеров по фото"),
    ),
    modifiers(&SecurityAddon),
)]
pub struct ApiDoc;

use utoipa::Modify;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_token",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}
