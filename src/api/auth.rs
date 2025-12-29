use axum::{
    extract::State,
    response::Json,
    routing::{post, Router},
};

use crate::api::AppState;
use crate::error::AppResult;
use crate::models::auth::{
    AuthStartRequest, AuthStartResponse, AuthVerifyRequest, AuthVerifyResponse,
    RefreshTokenRequest, RefreshTokenResponse,
};

pub fn auth_router() -> Router<AppState> {
    Router::new()
        .route("/start", post(start_auth))
        .route("/verify", post(verify_auth))
        .route("/refresh", post(refresh_token))
}

/// Начало авторизации - отправка SMS кода
#[utoipa::path(
    post,
    path = "/api/auth/start",
    request_body = AuthStartRequest,
    responses(
        (status = 200, description = "SMS код отправлен", body = AuthStartResponse),
        (status = 400, description = "Неверный формат номера телефона"),
    ),
    tag = "auth"
)]
pub async fn start_auth(
    State(state): State<AppState>,
    Json(payload): Json<AuthStartRequest>,
) -> AppResult<Json<AuthStartResponse>> {
    let response = state.auth_service.start_auth(&payload.phone).await?;
    Ok(Json(response))
}

/// Подтверждение авторизации - проверка SMS кода
#[utoipa::path(
    post,
    path = "/api/auth/verify",
    request_body = AuthVerifyRequest,
    responses(
        (status = 200, description = "Авторизация успешна", body = AuthVerifyResponse),
        (status = 400, description = "Неверный код"),
        (status = 401, description = "Код неверен или истек"),
    ),
    tag = "auth"
)]
pub async fn verify_auth(
    State(state): State<AppState>,
    Json(payload): Json<AuthVerifyRequest>,
) -> AppResult<Json<AuthVerifyResponse>> {
    let response = state
        .auth_service
        .verify_auth(
            &payload.phone,
            &payload.code,
            &state.user_repository,
            &state.user_plate_repository,
        )
        .await?;
    Ok(Json(response))
}

/// Обновление JWT токена
#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Токен обновлен", body = RefreshTokenResponse),
        (status = 401, description = "Токен неверен или истек"),
    ),
    tag = "auth"
)]
pub async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> AppResult<Json<RefreshTokenResponse>> {
    let response = state.auth_service.refresh_token(&payload.token).await?;
    Ok(Json(response))
}
