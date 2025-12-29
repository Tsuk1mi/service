use axum::{
    extract::{Extension, Path, Query, State},
    response::Json,
    routing::{get, patch, Router},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::api::AppState;
use crate::auth::middleware::AuthState;
use crate::error::AppResult;
use crate::models::notification::NotificationResponse;
use crate::repository::NotificationRepository;

pub fn notification_router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_notifications))
        .route("/:id/read", patch(mark_notification_read))
        .route("/read-all", patch(mark_all_read))
}

#[derive(Deserialize)]
pub struct GetNotificationsQuery {
    pub unread_only: Option<bool>,
}

async fn get_notifications(
    State(state): State<AppState>,
    Extension(auth_state): Extension<AuthState>,
    Query(params): Query<GetNotificationsQuery>,
) -> AppResult<Json<Vec<NotificationResponse>>> {
    let user_id = auth_state.user_id;
    let unread_only = params.unread_only.unwrap_or(false);

    let notifications = state
        .notification_repository
        .find_by_user_id(user_id, unread_only)
        .await?;

    let responses: Vec<NotificationResponse> = notifications
        .into_iter()
        .map(|n| NotificationResponse {
            id: n.id,
            r#type: n.r#type,
            title: n.title,
            message: n.message,
            data: n.data,
            read: n.read,
            created_at: n.created_at,
        })
        .collect();

    Ok(Json(responses))
}

async fn mark_notification_read(
    State(state): State<AppState>,
    Extension(auth_state): Extension<AuthState>,
    Path(notification_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let user_id = auth_state.user_id;

    state
        .notification_repository
        .mark_as_read(notification_id, user_id)
        .await?;

    Ok(Json(
        serde_json::json!({ "message": "Notification marked as read" }),
    ))
}

async fn mark_all_read(
    State(state): State<AppState>,
    Extension(auth_state): Extension<AuthState>,
) -> AppResult<Json<serde_json::Value>> {
    let user_id = auth_state.user_id;

    state
        .notification_repository
        .mark_all_as_read(user_id)
        .await?;

    Ok(Json(
        serde_json::json!({ "message": "All notifications marked as read" }),
    ))
}
