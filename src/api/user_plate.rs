use axum::{
    extract::{Extension, Path, State},
    response::Json,
    routing::{delete, get, patch, post, Router},
};
use uuid::Uuid;

use crate::api::AppState;
use crate::auth::middleware::AuthState;
use crate::error::AppResult;
use crate::models::user_plate::{
    CreateUserPlateRequest, UpdateUserPlateRequest, UserPlateResponse,
};
use crate::repository::UserPlateRepository;
use crate::service::validation_service::ValidationService;
use chrono::NaiveTime;

pub fn user_plate_router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_user_plate))
        .route("/", get(get_user_plates))
        .route("/:id/primary", post(set_primary_plate))
        .route("/:id", patch(update_user_plate))
        .route("/:id", delete(delete_user_plate))
}

/// Добавить автомобиль пользователя
#[utoipa::path(
    post,
    path = "/api/user/plates",
    request_body = CreateUserPlateRequest,
    responses(
        (status = 200, description = "Автомобиль добавлен", body = UserPlateResponse),
        (status = 401, description = "Не авторизован"),
    ),
    security(("bearer_token" = [])),
    tag = "user_plates"
)]
pub async fn create_user_plate(
    State(state): State<AppState>,
    Extension(auth_state): Extension<AuthState>,
    Json(mut payload): Json<CreateUserPlateRequest>,
) -> AppResult<Json<UserPlateResponse>> {
    let user_id = auth_state.user_id;

    tracing::info!(
        "API: create_user_plate called for user {} with plate {}",
        user_id,
        payload.plate
    );

    // Нормализация и валидация
    payload.normalize();
    let normalized_plate = ValidationService::validate_plate(&payload.plate).map_err(|e| {
        tracing::error!("Plate validation failed: {:?}", e);
        e
    })?;

    let is_primary = payload.is_primary.unwrap_or(false);
    let departure_time = payload
        .departure_time
        .as_ref()
        .map(|t| NaiveTime::parse_from_str(t, "%H:%M"))
        .transpose()
        .map_err(|_| {
            crate::error::AppError::Validation(
                "Некорректное время выезда, используйте HH:MM".to_string(),
            )
        })?;

    let user_plate = state
        .user_plate_repository
        .create(user_id, &normalized_plate, is_primary, departure_time)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create user plate: {:?}", e);
            e
        })?;

    tracing::info!("User plate created successfully: {}", user_plate.id);
    Ok(Json(user_plate.to_response()))
}

/// Список автомобилей пользователя
#[utoipa::path(
    get,
    path = "/api/user/plates",
    responses(
        (status = 200, description = "Список автомобилей", body = Vec<UserPlateResponse>),
        (status = 401, description = "Не авторизован"),
    ),
    security(("bearer_token" = [])),
    tag = "user_plates"
)]
pub async fn get_user_plates(
    State(state): State<AppState>,
    Extension(auth_state): Extension<AuthState>,
) -> AppResult<Json<Vec<UserPlateResponse>>> {
    let user_id = auth_state.user_id;

    let plates = state
        .user_plate_repository
        .find_by_user_id(user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get user plates: {:?}", e);
            e
        })?;

    let responses = plates.iter().map(|p| p.to_response()).collect();
    Ok(Json(responses))
}

async fn set_primary_plate(
    State(state): State<AppState>,
    Extension(auth_state): Extension<AuthState>,
    Path(plate_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let user_id = auth_state.user_id;

    tracing::info!(
        "API: set_primary_plate called for user {} and plate {}",
        user_id,
        plate_id
    );

    state
        .user_plate_repository
        .set_primary(plate_id, user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to set primary plate: {:?}", e);
            e
        })?;

    tracing::info!("Primary plate set successfully");
    Ok(Json(
        serde_json::json!({ "message": "Primary plate set successfully" }),
    ))
}

async fn delete_user_plate(
    State(state): State<AppState>,
    Extension(auth_state): Extension<AuthState>,
    Path(plate_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let user_id = auth_state.user_id;

    tracing::info!(
        "API: delete_user_plate called for user {} and plate {}",
        user_id,
        plate_id
    );

    state
        .user_plate_repository
        .delete(plate_id, user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete user plate: {:?}", e);
            e
        })?;

    tracing::info!("User plate deleted successfully");
    Ok(Json(
        serde_json::json!({ "message": "User plate deleted successfully" }),
    ))
}

async fn update_user_plate(
    State(state): State<AppState>,
    Extension(auth_state): Extension<AuthState>,
    Path(plate_id): Path<Uuid>,
    Json(payload): Json<UpdateUserPlateRequest>,
) -> AppResult<Json<UserPlateResponse>> {
    let user_id = auth_state.user_id;

    // PATCH semantics:
    // - None -> поле не передано, не изменять
    // - Some(None) -> очистить
    // - Some(Some("HH:MM")) -> установить
    match payload.departure_time {
        None => {
            // Ничего не меняем — возвращаем текущую запись (и проверяем владение)
            let existing = state
                .user_plate_repository
                .find_by_id(plate_id)
                .await?
                .ok_or_else(|| {
                    crate::error::AppError::NotFound("User plate not found".to_string())
                })?;

            if existing.user_id != user_id {
                // Не раскрываем существование чужих записей
                return Err(crate::error::AppError::NotFound(
                    "User plate not found".to_string(),
                ));
            }

            Ok(Json(existing.to_response()))
        }
        Some(None) => {
            let updated = state
                .user_plate_repository
                .update_departure_time(plate_id, user_id, None)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to clear departure_time: {:?}", e);
                    e
                })?;
            Ok(Json(updated.to_response()))
        }
        Some(Some(t)) => {
            let t = t.trim();
            let time = if t.is_empty() {
                None
            } else {
                Some(NaiveTime::parse_from_str(t, "%H:%M").map_err(|_| {
                    crate::error::AppError::Validation(
                        "Некорректное время выезда, используйте HH:MM".to_string(),
                    )
                })?)
            };

            let updated = state
                .user_plate_repository
                .update_departure_time(plate_id, user_id, time)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to update user plate: {:?}", e);
                    e
                })?;

            Ok(Json(updated.to_response()))
        }
    }
}
