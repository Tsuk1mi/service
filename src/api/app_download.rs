use crate::api::AppState;
use crate::utils::apk::find_latest_apk;
use axum::{
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use tokio::fs::File;
use tokio_util::io::ReaderStream;

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
pub async fn download_app(State(state): State<AppState>) -> Result<impl IntoResponse, StatusCode> {
    // Определяем путь к APK файлу/директории
    let configured_path =
        state.config.app_apk_path.clone().unwrap_or_else(|| {
            "./android/app/build/outputs/apk/release/app-release.apk".to_string()
        });

    // APP_APK_PATH может быть как файлом, так и директорией с несколькими APK.
    // В случае директории выбираем "самый свежий" APK (по semver из имени, иначе fallback).
    let candidate = match find_latest_apk(&configured_path).await {
        Some(c) => c,
        None => {
            // Дополнительный fallback: если default файл не найден, попробуем директорию release/
            if configured_path.ends_with("app-release.apk") {
                if let Some(c) = find_latest_apk("./android/app/build/outputs/apk/release").await {
                    c
                } else {
                    tracing::warn!("APK не найден: path_or_dir={}", configured_path);
                    return Err(StatusCode::NOT_FOUND);
                }
            } else {
                tracing::warn!("APK не найден: path_or_dir={}", configured_path);
                return Err(StatusCode::NOT_FOUND);
            }
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
