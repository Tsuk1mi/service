use axum::{
    extract::State,
    http::{header, HeaderMap, HeaderValue},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};

use crate::api::AppState;
use crate::config::Config;
use crate::error::AppResult;
use crate::models::auth::{
    AuthStartRequest, AuthStartResponse, AuthVerifyRequest,
    LogoutRequest, RefreshTokenRequest,
};
use crate::utils::validate::validate_payload;

const REFRESH_COOKIE: &str = "rimskiy_refresh";

pub fn auth_router() -> Router<AppState> {
    Router::new()
        .route("/start", post(start_auth))
        .route("/verify", post(verify_auth))
        .route("/refresh", post(refresh_token))
        .route("/logout", post(logout))
}

fn refresh_cookie_header(token: &str, config: &Config) -> AppResult<HeaderValue> {
    let max_age = config.jwt_refresh_expiration_minutes * 60;
    let secure = if config.app_env.is_production() {
        "; Secure"
    } else {
        ""
    };
    HeaderValue::from_str(&format!(
        "{REFRESH_COOKIE}={token}; HttpOnly; Path=/api/auth; Max-Age={max_age}; SameSite=Strict{secure}"
    ))
    .map_err(|e| crate::error::AppError::Internal(format!("Invalid cookie: {}", e)))
}

fn clear_refresh_cookie(config: &Config) -> AppResult<HeaderValue> {
    let secure = if config.app_env.is_production() {
        "; Secure"
    } else {
        ""
    };
    HeaderValue::from_str(&format!(
        "{REFRESH_COOKIE}=; HttpOnly; Path=/api/auth; Max-Age=0; SameSite=Strict{secure}"
    ))
    .map_err(|e| crate::error::AppError::Internal(format!("Invalid cookie: {}", e)))
}

fn extract_refresh_token(headers: &HeaderMap, body_token: Option<&str>) -> Option<String> {
    if let Some(token) = body_token.filter(|t| !t.is_empty()) {
        return Some(token.to_string());
    }
    headers.get(header::COOKIE).and_then(|value| {
        value.to_str().ok().and_then(|cookies| {
            cookies.split(';').find_map(|part| {
                let part = part.trim();
                part.strip_prefix(&format!("{REFRESH_COOKIE}="))
                    .map(|v| v.to_string())
            })
        })
    })
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
    validate_payload(&payload)?;
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
) -> AppResult<Response> {
    validate_payload(&payload)?;
    let response = state
        .auth_service
        .verify_auth(
            &payload.phone,
            &payload.code,
            &state.user_repository,
            &state.user_plate_repository,
        )
        .await?;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        refresh_cookie_header(&response.refresh_token, &state.config)?,
    );

    Ok((headers, Json(response)).into_response())
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
    headers: HeaderMap,
    body: Option<Json<RefreshTokenRequest>>,
) -> AppResult<Response> {
    let token = extract_refresh_token(
        &headers,
        body.as_ref().map(|b| b.token.as_str()),
    )
    .ok_or_else(|| crate::error::AppError::Auth("Refresh token required".to_string()))?;

    let response = state.auth_service.refresh_token(&token).await?;

    let mut out_headers = HeaderMap::new();
    out_headers.insert(
        header::SET_COOKIE,
        refresh_cookie_header(&response.refresh_token, &state.config)?,
    );

    Ok((out_headers, Json(response)).into_response())
}

/// Выход — отзыв refresh token
#[utoipa::path(
    post,
    path = "/api/auth/logout",
    request_body = LogoutRequest,
    responses(
        (status = 200, description = "Выход выполнен"),
        (status = 401, description = "Токен неверен"),
    ),
    tag = "auth"
)]
pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Option<Json<LogoutRequest>>,
) -> AppResult<Response> {
    let token = extract_refresh_token(
        &headers,
        body.as_ref().and_then(|b| b.token.as_deref()),
    )
    .ok_or_else(|| crate::error::AppError::Auth("Refresh token required".to_string()))?;

    state.auth_service.logout(&token).await?;

    let mut out_headers = HeaderMap::new();
    out_headers.insert(
        header::SET_COOKIE,
        clear_refresh_cookie(&state.config)?,
    );

    Ok((out_headers, Json(serde_json::json!({"success": true}))).into_response())
}
