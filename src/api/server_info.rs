use crate::api::AppState;
use axum::{extract::State, http::HeaderMap, response::Json, routing::get, Router};
use serde_json::json;

pub fn server_info_router() -> Router<AppState> {
    Router::new().route("/server-info", get(get_server_info))
}

fn build_base_url_from_headers(headers: &HeaderMap) -> Option<String> {
    let host = headers
        .get("x-forwarded-host")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            headers
                .get("host")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
        })?;

    let proto = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or("http");

    Some(format!("{}://{}", proto, host))
}

async fn get_server_info(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<serde_json::Value> {
    let server_url = build_base_url_from_headers(&headers)
        .unwrap_or_else(|| format!("http://localhost:{}", state.config.server_port));

    let web_app_url = state
        .config
        .web_app_url
        .clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| server_url.clone());

    let telegram_bot_username = std::env::var("TELEGRAM_BOT_USERNAME").ok();

    Json(json!({
        "server_url": server_url,
        "port": state.config.server_port,
        "server_version": env!("CARGO_PKG_VERSION"),
        "web_app_url": web_app_url,
        "telegram_bot_username": telegram_bot_username,
    }))
}
