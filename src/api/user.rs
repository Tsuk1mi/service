use axum::{
    extract::{Extension, Query, State},
    response::Json,
    routing::{get, put, Router, post},
};
use serde::Deserialize;
use serde::Serialize;

use crate::api::AppState;
use crate::auth::middleware::AuthState;
use crate::error::AppResult;
use crate::models::user::{UpdateUserRequest, UserResponse, PublicUserInfo};
use crate::repository::user_repository::UserRepository;

pub fn user_router() -> Router<AppState> {
    Router::new()
        .route("/me", get(get_profile))
        .route("/me", put(update_profile))
        .route("/push-token", post(register_push_token))
        .route("/by-plate", get(get_user_by_plate))
}

#[derive(Deserialize)]
pub struct GetUserByPlateQuery {
    pub plate: String,
}

/// Получить профиль текущего пользователя
#[utoipa::path(
    get,
    path = "/api/users/me",
    responses(
        (status = 200, description = "Профиль пользователя", body = UserResponse),
        (status = 401, description = "Не авторизован"),
    ),
    security(("bearer_token" = [])),
    tag = "users"
)]
pub async fn get_profile(
    State(state): State<AppState>,
    Extension(auth_state): Extension<AuthState>,
) -> AppResult<Json<UserResponse>> {
    let user_id = auth_state.user_id;

    let response = state.user_service.get_profile(
        user_id,
        &state.user_repository,
        &state.user_plate_repository,
    ).await?;
    Ok(Json(response))
}

/// Обновить профиль текущего пользователя
#[utoipa::path(
    put,
    path = "/api/users/me",
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "Профиль обновлен", body = UserResponse),
        (status = 400, description = "Неверные данные"),
        (status = 401, description = "Не авторизован"),
    ),
    security(("bearer_token" = [])),
    tag = "users"
)]
pub async fn update_profile(
    State(state): State<AppState>,
    Extension(auth_state): Extension<AuthState>,
    Json(payload): Json<UpdateUserRequest>,
) -> AppResult<Json<UserResponse>> {
    let user_id = auth_state.user_id;

    tracing::info!("API: update_profile called for user {}", user_id);
    tracing::debug!("API: update_profile payload: name={:?}, phone={:?}, telegram={:?}, plate={:?}, owner_type={:?}, owner_info={:?}", 
        payload.name.as_ref().map(|s| if s.is_empty() { "<empty>" } else { s.as_str() }),
        payload.phone.as_ref().map(|_| "<present>"),
        payload.telegram.as_ref().map(|s| if s.is_empty() { "<empty>" } else { s.as_str() }),
        payload.plate.as_ref().map(|s| s.as_str()),
        payload.owner_type.as_ref().map(|s| s.as_str()),
        payload.owner_info.is_some()
    );

    let response = state.user_service.update_profile(
        user_id,
        payload,
        &state.user_repository,
        &state.user_plate_repository,
    ).await.map_err(|e| {
        tracing::error!("API: Failed to update profile for user {}: {:?}", user_id, e);
        e
    })?;

    tracing::info!("API: Profile updated successfully for user {}", user_id);
    Ok(Json(response))
}

/// Получить публичную информацию о пользователе по номеру автомобиля
#[utoipa::path(
    get,
    path = "/api/users/by-plate",
    params(
        ("plate" = String, Query, description = "Номер автомобиля")
    ),
    responses(
        (status = 200, description = "Информация о пользователе", body = PublicUserInfo),
        (status = 404, description = "Пользователь не найден"),
        (status = 401, description = "Не авторизован"),
    ),
    security(("bearer_token" = [])),
    tag = "users"
)]
pub async fn get_user_by_plate(
    State(state): State<AppState>,
    Query(params): Query<GetUserByPlateQuery>,
) -> AppResult<Json<Option<PublicUserInfo>>> {
    let user_info = state.user_service.get_user_by_plate(
        &params.plate,
        &state.user_repository,
        &state.user_plate_repository,
    ).await?;
    
    Ok(Json(user_info))
}

#[derive(Deserialize, Serialize)]
pub struct PushTokenRequest {
    pub token: String,
}

/// Зарегистрировать push token для текущего пользователя
#[utoipa::path(
    post,
    path = "/api/users/push-token",
    request_body = PushTokenRequest,
    responses(
        (status = 200, description = "Токен сохранён"),
        (status = 400, description = "Неверные данные"),
        (status = 401, description = "Не авторизован"),
    ),
    security(("bearer_token" = [])),
    tag = "users"
)]
pub async fn register_push_token(
    State(state): State<AppState>,
    Extension(auth_state): Extension<AuthState>,
    Json(payload): Json<PushTokenRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let user_id = auth_state.user_id;
    let token = payload.token.trim();
    if token.is_empty() {
        return Err(crate::error::AppError::Validation("Пустой push token".into()));
    }

    // Обновляем только токен
    let update = crate::repository::user_repository::UpdateUserData {
        name: None,
        phone_encrypted: None,
        phone_hash: None,
        telegram: None,
        plate: None,
        show_contacts: None,
        owner_type: None,
        owner_info: None,
        departure_time: None,
        push_token: Some(token.to_string()),
    };

    state.user_repository.update(user_id, &update).await?;
    Ok(Json(serde_json::json!({"message": "push token saved"})))
}

