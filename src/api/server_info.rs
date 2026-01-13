use crate::api::AppState;
use axum::{
    extract::State,
    http::HeaderMap,
    response::Json,
    routing::get,
    Router,
};
use serde_json::json;

pub fn server_info_router() -> Router<AppState> {
    Router::new().route("/server-info", get(get_server_info))
}

fn build_base_url_from_headers(headers: &HeaderMap) -> Option<String> {
    // Prefer reverse-proxy headers if present (nginx, cloudflare, etc.)
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

fn append_cache_bust_version(url: String, version: Option<&str>) -> String {
    let Some(version) = version.filter(|v| !v.trim().is_empty()) else {
        return url;
    };

    // Don't add if already present
    if url.contains("v=") {
        return url;
    }

    let joiner = if url.contains('?') { "&" } else { "?" };
    format!("{url}{joiner}v={version}")
}

async fn get_server_info(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<serde_json::Value> {
    // Build the URL based on how the client is actually reaching us.
    // This avoids broken defaults like 192.168.1.1 and works behind proxies.
    let server_url = build_base_url_from_headers(&headers)
        .unwrap_or_else(|| format!("http://localhost:{}", state.config.server_port));

    // Получаем URL для скачивания APK
    let app_download_url_raw = state
        .config
        .app_download_url
        .clone()
        .unwrap_or_else(|| format!("{}/api/app/download", server_url));

    // Получаем username бота из токена (если есть)
    let telegram_bot_username = std::env::var("TELEGRAM_BOT_USERNAME").ok();

    // Автоматически определяем версию приложения на основе версии сервера
    // Если release_client_version не указан, используем server_version
    let auto_release_version = state
        .config
        .release_client_version
        .clone()
        .or_else(|| Some(env!("CARGO_PKG_VERSION").to_string()));

    // Add a stable cache-busting query parameter so phones/CDNs don't serve stale APKs
    // when a new release is available at the same endpoint.
    let app_download_url =
        append_cache_bust_version(app_download_url_raw, auto_release_version.as_deref());

    Json(json!({
        "server_url": server_url,
        "port": state.config.server_port,
        "server_version": env!("CARGO_PKG_VERSION"),
        "min_client_version": state.config.min_client_version,
        "release_client_version": auto_release_version,
        "app_download_url": app_download_url,
        "telegram_bot_username": telegram_bot_username,
    }))
}
