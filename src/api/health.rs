use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde_json::json;

use crate::api::AppState;

pub fn health_router() -> Router<AppState> {
    Router::new()
        .route("/live", get(health_live))
        .route("/ready", get(health_ready))
}

async fn health_live() -> &'static str {
    "OK"
}

async fn health_ready(State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut checks = json!({
        "database": "unknown",
        "redis": "not_configured"
    });

    // Проверка PostgreSQL
    match sqlx::query("SELECT 1").execute(&*state.db_pool).await {
        Ok(_) => checks["database"] = json!("ok"),
        Err(e) => {
            tracing::error!("Health check DB failed: {}", e);
            checks["database"] = json!("error");
            return Err(StatusCode::SERVICE_UNAVAILABLE);
        }
    }

    // Проверка Redis
    if let Some(redis) = &state.redis {
        match redis.ping().await {
            Ok(_) => checks["redis"] = json!("ok"),
            Err(e) => {
                tracing::error!("Health check Redis failed: {}", e);
                checks["redis"] = json!("error");
                return Err(StatusCode::SERVICE_UNAVAILABLE);
            }
        }
    }

    Ok(Json(json!({ "status": "ready", "checks": checks })))
}
