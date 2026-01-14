use crate::api::AppState;
use crate::utils::apk::{find_latest_apk_cached, parse_version_from_filename};
use axum::{extract::State, http::HeaderMap, response::Json, routing::get, Router};
use serde_json::json;
use std::time::Duration;

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

fn filename_from_url(url: &str) -> Option<String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return None;
    }
    let no_fragment = trimmed.split('#').next().unwrap_or(trimmed);
    let no_query = no_fragment.split('?').next().unwrap_or(no_fragment);
    let last = no_query.rsplit('/').next().unwrap_or(no_query).trim();
    if last.is_empty() {
        None
    } else {
        Some(last.to_string())
    }
}

async fn detect_release_client_version(state: &AppState) -> Option<String> {
    // 1) Явная настройка имеет приоритет
    if let Some(v) = state
        .config
        .release_client_version
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        return Some(v.to_string());
    }

    // 1.5) Если APK хранится в Nexus и ссылка известна — попробуем распарсить версию из имени файла URL.
    if let Some(url) = state
        .config
        .nexus_apk_url
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        if let Some(fname) = filename_from_url(url) {
            if let Some((maj, min, pat)) = parse_version_from_filename(&fname) {
                return Some(format!("{maj}.{min}.{pat}"));
            }
        }
    }

    // 2) Пытаемся определить версию из имени APK (app-release-vX.Y.Z.apk)
    // ВАЖНО: используем кэш, чтобы не сканировать FS на каждом запросе `/server-info`.
    let configured_path =
        state.config.app_apk_path.clone().unwrap_or_else(|| {
            "./android/app/build/outputs/apk/release/app-release.apk".to_string()
        });

    // TTL небольшой: чтобы подхватывать свежие APK без рестарта, но не грузить FS.
    let ttl = Duration::from_secs(60);
    if let Some(c) = find_latest_apk_cached(&state.apk_cache, &configured_path, ttl).await {
        if let Some((maj, min, pat)) = c.version {
            return Some(format!("{maj}.{min}.{pat}"));
        }
    }

    // 3) Фолбэк: если ничего не нашли, оставляем как раньше (версия сервера),
    // но это может не совпадать с версией Android-клиента.
    Some(env!("CARGO_PKG_VERSION").to_string())
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

    // Определяем версию "клиентского релиза" (для кнопки обновления):
    // - приоритет RELEASE_CLIENT_VERSION из env
    // - иначе пробуем распарсить из имени APK (app-release-vX.Y.Z.apk)
    // - иначе fallback на версию сервера (не идеально, но лучше чем null)
    let auto_release_version = detect_release_client_version(&state).await;

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
