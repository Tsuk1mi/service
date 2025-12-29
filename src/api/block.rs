use axum::{
    extract::{Extension, Path, Query, State},
    response::Json,
    routing::{delete, get, post, Router},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::api::AppState;
use crate::auth::middleware::AuthState;
use crate::error::AppResult;
use crate::models::block::{Block, BlockWithBlockerInfo, CheckBlockResponse, CreateBlockRequest};

pub fn block_router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_block))
        .route("/", get(get_my_blocks))
        .route("/my", get(get_blocks_for_my_plate))
        .route("/check", get(check_block))
        .route("/:id/warn-owner", post(warn_owner))
        .route("/:id", delete(delete_block))
}

#[derive(Deserialize)]
pub struct GetBlocksQuery {
    pub my_plate: Option<String>,
}

/// Создать блокировку автомобиля
#[utoipa::path(
    post,
    path = "/api/blocks",
    request_body = CreateBlockRequest,
    responses(
        (status = 200, description = "Блокировка создана", body = Block),
        (status = 400, description = "Неверные данные"),
        (status = 401, description = "Не авторизован"),
    ),
    security(("bearer_token" = [])),
    tag = "blocks"
)]
pub async fn create_block(
    State(state): State<AppState>,
    Extension(auth_state): Extension<AuthState>,
    Json(payload): Json<CreateBlockRequest>,
) -> AppResult<Json<Block>> {
    let blocker_id = auth_state.user_id;

    tracing::info!(
        "API: create_block called for user {} with plate {}",
        blocker_id,
        payload.blocked_plate
    );

    let block = state
        .block_service
        .create_block(
            blocker_id,
            payload,
            &state.block_repository,
            &state.notification_repository,
            &state.user_repository,
            &state.user_plate_repository,
            &state.telephony_service,
        )
        .await
        .map_err(|e| {
            tracing::error!("API: Failed to create block: {:?}", e);
            e
        })?;

    tracing::info!("API: Block created successfully: {}", block.id);
    Ok(Json(block))
}

/// Получить список тех, кто перекрыл мои автомобили
#[utoipa::path(
    get,
    path = "/api/blocks/my",
    params(
        ("my_plate" = Option<String>, Query, description = "Фильтр по номеру автомобиля (опционально)")
    ),
    responses(
        (status = 200, description = "Список блокировок", body = Vec<BlockWithBlockerInfo>),
        (status = 401, description = "Не авторизован"),
    ),
    security(("bearer_token" = [])),
    tag = "blocks"
)]
pub async fn get_blocks_for_my_plate(
    State(state): State<AppState>,
    Extension(auth_state): Extension<AuthState>,
    Query(params): Query<GetBlocksQuery>,
) -> AppResult<Json<Vec<BlockWithBlockerInfo>>> {
    let user_id = auth_state.user_id;

    let blocks = state
        .block_service
        .get_blocks_for_my_plate(
            user_id,
            params.my_plate,
            &state.block_repository,
            &state.user_repository,
            &state.user_plate_repository,
        )
        .await?;

    Ok(Json(blocks))
}

/// Получить список автомобилей, которые перекрыл текущий пользователь
#[utoipa::path(
    get,
    path = "/api/blocks",
    responses(
        (status = 200, description = "Список созданных блокировок", body = Vec<Block>),
        (status = 401, description = "Не авторизован"),
    ),
    security(("bearer_token" = [])),
    tag = "blocks"
)]
pub async fn get_my_blocks(
    State(state): State<AppState>,
    Extension(auth_state): Extension<AuthState>,
) -> AppResult<Json<Vec<Block>>> {
    let blocker_id = auth_state.user_id;

    let blocks = state
        .block_service
        .get_my_blocks(blocker_id, &state.block_repository)
        .await?;

    Ok(Json(blocks))
}

/// Удалить блокировку
#[utoipa::path(
    delete,
    path = "/api/blocks/{id}",
    params(
        ("id" = Uuid, Path, description = "ID блокировки")
    ),
    responses(
        (status = 200, description = "Блокировка удалена"),
        (status = 401, description = "Не авторизован"),
        (status = 404, description = "Блокировка не найдена"),
    ),
    security(("bearer_token" = [])),
    tag = "blocks"
)]
pub async fn delete_block(
    State(state): State<AppState>,
    Extension(auth_state): Extension<AuthState>,
    Path(block_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let blocker_id = auth_state.user_id;

    state
        .block_service
        .delete_block(
            block_id,
            blocker_id,
            &state.block_repository,
            &state.notification_repository,
            &state.user_repository,
            &state.user_plate_repository,
        )
        .await?;

    Ok(Json(
        serde_json::json!({ "message": "Block deleted successfully" }),
    ))
}

#[derive(Deserialize)]
pub struct CheckBlockQuery {
    pub plate: String,
}

/// Проверить, заблокирована ли машина
#[utoipa::path(
    get,
    path = "/api/blocks/check",
    params(
        ("plate" = String, Query, description = "Номер автомобиля для проверки")
    ),
    responses(
        (status = 200, description = "Результат проверки", body = CheckBlockResponse),
        (status = 401, description = "Не авторизован"),
    ),
    security(("bearer_token" = [])),
    tag = "blocks"
)]
pub async fn check_block(
    State(state): State<AppState>,
    Query(params): Query<CheckBlockQuery>,
) -> AppResult<Json<CheckBlockResponse>> {
    let response = state
        .block_service
        .check_block(
            &params.plate,
            &state.block_repository,
            &state.user_repository,
        )
        .await?;

    Ok(Json(response))
}

/// Предупредить владельца заблокированного автомобиля (звонок)
#[utoipa::path(
    post,
    path = "/api/blocks/{id}/warn-owner",
    params(
        ("id" = Uuid, Path, description = "ID блокировки")
    ),
    responses(
        (status = 200, description = "Владелец предупрежден"),
        (status = 401, description = "Не авторизован"),
        (status = 404, description = "Блокировка не найдена"),
    ),
    security(("bearer_token" = [])),
    tag = "blocks"
)]
pub async fn warn_owner(
    State(state): State<AppState>,
    Extension(auth_state): Extension<AuthState>,
    Path(block_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let blocker_id = auth_state.user_id;

    tracing::info!(
        "API: warn_owner called for user {} and block {}",
        blocker_id,
        block_id
    );

    state
        .block_service
        .warn_owner(
            block_id,
            blocker_id,
            &state.block_repository,
            &state.user_repository,
            &state.user_plate_repository,
            &state.telephony_service,
        )
        .await
        .map_err(|e| {
            tracing::error!("API: Failed to warn owner: {:?}", e);
            e
        })?;

    tracing::info!("API: Owner warned successfully for block {}", block_id);
    Ok(Json(
        serde_json::json!({ "message": "Owner warned successfully" }),
    ))
}
