use crate::api::AppState;
use crate::utils::apk::find_latest_apk;
use axum::{
    body::Bytes,
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use tokio::fs;

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
    // Определяем путь к APK файлу
    //
    // APP_APK_PATH может указывать как на файл, так и на директорию с версиями:
    // app-release-v1.0.56.apk, app-release-v1.0.55.apk, ...
    let path_or_dir =
        state.config.app_apk_path.clone().unwrap_or_else(|| {
            "./android/app/build/outputs/apk/release/app-release.apk".to_string()
        });

    let selected = find_latest_apk(&path_or_dir)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;

    let apk_path = selected.path;

    // Проверяем существование файла
    if !apk_path.exists() {
        tracing::warn!("APK файл не найден: {:?}", apk_path);
        return Err(StatusCode::NOT_FOUND);
    }

    // Читаем файл
    let file_contents = fs::read(&apk_path).await.map_err(|e| {
        tracing::error!("Ошибка при чтении APK файла: {:?}, ошибка: {}", apk_path, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Получаем имя файла для заголовка Content-Disposition
    let filename = apk_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("app-release.apk");

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

    tracing::info!("APK файл успешно отправлен: {}", filename);
    Ok((StatusCode::OK, headers, Bytes::from(file_contents)))
}
