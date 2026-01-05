use crate::api::AppState;
use crate::utils::network::get_server_url;
use axum::{extract::State, response::Json, routing::get, Router};
use serde_json::json;
pub fn server_info_router() -> Router<AppState> {
    Router::new().route("/server-info", get(get_server_info))
}

async fn get_server_info(State(state): State<AppState>) -> Json<serde_json::Value> {
    let server_url = get_server_url(state.config.server_port);

    // Получаем URL для скачивания APK
    let app_download_url = state
        .config
        .app_download_url
        .clone()
        .unwrap_or_else(|| format!("{}/api/app/download", server_url));

    // Получаем username бота из токена (если есть)
    let telegram_bot_username = std::env::var("TELEGRAM_BOT_USERNAME").ok();

    // Автоматически определяем версию приложения:
    // 1) RELEASE_CLIENT_VERSION (если задан)
    // 2) по имени APK в директории/пути APP_APK_PATH (app-release-vX.Y.Z.apk)
    // 3) версия сервера (fallback)
    let apk_version = if state.config.release_client_version.is_none() {
        if let Some(path_or_dir) = state.config.app_apk_path.as_ref() {
            if let Some(candidate) = crate::utils::apk::find_latest_apk(path_or_dir).await {
                candidate
                    .version
                    .map(|(a, b, c)| format!("{}.{}.{}", a, b, c))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let auto_release_version = state
        .config
        .release_client_version
        .clone()
        .or(apk_version)
        .or_else(|| Some(env!("CARGO_PKG_VERSION").to_string()));

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
