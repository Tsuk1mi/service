use crate::api::AppState;
use crate::config::Config;
use axum::{extract::State, http::HeaderMap, response::Json, routing::get, Router};
use serde_json::{json, Value};

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

fn env_nonempty(key: &str) -> bool {
    std::env::var(key)
        .ok()
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false)
}

/// Integration availability flags for `/server-info` (env + config).
pub fn integrations_from_config(config: &Config) -> Value {
    let sms = config
        .sms_api_url
        .as_ref()
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);
    let telegram = env_nonempty("TELEGRAM_BOT_TOKEN")
        || env_nonempty("TELEGRAM_BOT_USERNAME")
        || config
            .telegram_bot_http_url
            .as_ref()
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);
    let ocr = env_nonempty("OCR_API_URL");
    let fcm = config
        .fcm_server_key
        .as_ref()
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);
    let redis = config.redis_url.is_some();
    let rabbitmq = config.rabbitmq_url.is_some();

    build_integrations(sms, telegram, ocr, fcm, redis, rabbitmq)
}

pub fn build_integrations(
    sms: bool,
    telegram: bool,
    ocr: bool,
    fcm: bool,
    redis: bool,
    rabbitmq: bool,
) -> Value {
    json!({
        "sms": sms,
        "telegram": telegram,
        "ocr": ocr,
        "fcm": fcm,
        "redis": redis,
        "rabbitmq": rabbitmq,
    })
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
        "integrations": integrations_from_config(&state.config),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::test_support::test_app_state;
    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[test]
    fn integrations_flags_shape() {
        let v = build_integrations(true, false, true, false, true, false);
        assert_eq!(v["sms"], true);
        assert_eq!(v["telegram"], false);
        assert_eq!(v["ocr"], true);
        assert_eq!(v["fcm"], false);
        assert_eq!(v["redis"], true);
        assert_eq!(v["rabbitmq"], false);
    }

    #[tokio::test]
    async fn server_info_returns_integrations_without_db() {
        let app = server_info_router().with_state(test_app_state());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/server-info")
                    .header("host", "example.test")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), 1024 * 64).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["port"], 8080);
        assert!(json["integrations"].is_object());
        assert_eq!(json["integrations"]["redis"], false);
        assert_eq!(json["integrations"]["sms"], false);
        assert!(json["server_url"].as_str().unwrap().contains("example.test"));
    }
}
