use crate::api::AppState;
use crate::utils::apk::find_latest_apk_cached;
use axum::{
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use std::time::Duration;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

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

/// Роутер для скачивания приложения
pub fn app_download_router() -> Router<AppState> {
    Router::new().route("/download", get(download_app))
}

/// Endpoint для скачивания релиза приложения
#[utoipa::path(
    get,
    path = "/api/app/download",
    responses(
        (status = 200, description = "APK файл"),
        (status = 404, description = "APK файл не найден"),
        (status = 500, description = "Ошибка сервера при чтении файла")
    ),
    tag = "app"
)]
pub async fn download_app(
    State(state): State<AppState>,
) -> Result<(StatusCode, HeaderMap, axum::body::Body), StatusCode> {
    // Если задан Nexus URL — проксируем APK через сервис (клиентам креды не нужны).
    if let Some(nexus_url) = state
        .config
        .nexus_apk_url
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        return proxy_apk_from_nexus(&state, nexus_url).await;
    }

    // Определяем путь к APK файлу/директории
    let configured_path =
        state.config.app_apk_path.clone().unwrap_or_else(|| {
            "./android/app/build/outputs/apk/release/app-release.apk".to_string()
        });

    // APP_APK_PATH может быть как файлом, так и директорией с несколькими APK.
    // В случае директории выбираем "самый свежий" APK (по semver из имени, иначе fallback).
    // ВАЖНО: используем кэш, чтобы не долбить FS при массовых запросах на скачивание/проверку.
    let ttl = Duration::from_secs(60);
    let candidate = match find_latest_apk_cached(&state.apk_cache, &configured_path, ttl).await {
        Some(c) => c,
        None => {
            tracing::warn!("APK не найден: path_or_dir={}", configured_path);
            return Err(StatusCode::NOT_FOUND);
        }
    };

    let apk_path = candidate.path;
    let filename = candidate.filename;

    let meta = tokio::fs::metadata(&apk_path).await.map_err(|e| {
        tracing::error!(
            "Ошибка при получении метаданных APK: {:?}, ошибка: {}",
            apk_path,
            e
        );
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if !meta.is_file() {
        tracing::warn!("APK путь не является файлом: {:?}", apk_path);
        return Err(StatusCode::NOT_FOUND);
    }

    // Открываем файл и стримим его (не читаем целиком в память)
    let file = File::open(&apk_path).await.map_err(|e| {
        tracing::error!(
            "Ошибка при открытии APK файла: {:?}, ошибка: {}",
            apk_path,
            e
        );
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let stream = ReaderStream::new(file);
    let body = axum::body::Body::from_stream(stream);

    // Формируем заголовки
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/vnd.android.package-archive"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename))
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, no-cache, must-revalidate, max-age=0"),
    );
    headers.insert(header::PRAGMA, HeaderValue::from_static("no-cache"));
    headers.insert(header::EXPIRES, HeaderValue::from_static("0"));
    headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&meta.len().to_string())
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    );

    tracing::info!("APK файл успешно отправлен: {}", filename);
    Ok((StatusCode::OK, headers, body))
}

async fn proxy_apk_from_nexus(
    state: &AppState,
    nexus_url: &str,
) -> Result<(StatusCode, HeaderMap, axum::body::Body), StatusCode> {
    let mut req = state.http_client.get(nexus_url);

    // Basic Auth (если задано)
    if let Some(user) = state
        .config
        .nexus_username
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        let pass = state
            .config
            .nexus_password
            .as_deref()
            .map(|s| s.to_string());
        req = req.basic_auth(user, pass);
    }

    let resp = req.send().await.map_err(|e| {
        tracing::error!("Ошибка запроса APK из Nexus: url={}, err={}", nexus_url, e);
        StatusCode::BAD_GATEWAY
    })?;

    let status = resp.status();
    if status.as_u16() == 404 {
        tracing::warn!("APK не найден в Nexus: url={}", nexus_url);
        return Err(StatusCode::NOT_FOUND);
    }
    if !status.is_success() {
        tracing::error!(
            "Nexus вернул ошибку при скачивании APK: url={}, status={}",
            nexus_url,
            status
        );
        return Err(StatusCode::BAD_GATEWAY);
    }

    let headers_in = resp.headers().clone();

    let filename = headers_in
        .get(header::CONTENT_DISPOSITION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| {
            // очень простой парсер filename="..."
            v.split("filename=")
                .nth(1)
                .map(|s| s.trim().trim_matches('"').to_string())
        })
        .or_else(|| filename_from_url(nexus_url))
        .unwrap_or_else(|| "app-release.apk".to_string());

    let content_type = headers_in
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/vnd.android.package-archive");

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(content_type).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename))
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, no-cache, must-revalidate, max-age=0"),
    );
    headers.insert(header::PRAGMA, HeaderValue::from_static("no-cache"));
    headers.insert(header::EXPIRES, HeaderValue::from_static("0"));

    if let Some(len) = headers_in.get(header::CONTENT_LENGTH) {
        headers.insert(header::CONTENT_LENGTH, len.clone());
    }

    let stream = resp.bytes_stream();
    let body = axum::body::Body::from_stream(stream);

    tracing::info!("APK проксирован из Nexus: filename={}, url={}", filename, nexus_url);
    Ok((StatusCode::OK, headers, body))
}
