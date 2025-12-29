use axum::{extract::State, response::Json, routing::get, Router};
use serde_json::json;
use crate::api::AppState;
use crate::utils::network::get_server_url;

pub fn server_info_router() -> Router<AppState> {
    Router::new()
        .route("/server-info", get(get_server_info))
}

async fn get_server_info(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let server_url = get_server_url(state.config.server_port);
    
    Json(json!({
        "server_url": server_url,
        "port": state.config.server_port,
    }))
}

