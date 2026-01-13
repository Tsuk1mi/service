use axum::{extract::Request, middleware::Next, response::Response};
use uuid::Uuid;

use crate::api::AppState;
use crate::auth::jwt::verify_token;
use crate::error::AppError;

#[derive(Clone, Debug)]
pub struct AuthState {
    pub user_id: Uuid,
}

pub async fn auth_middleware(
    axum::extract::State(state): axum::extract::State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let path = request.uri().path();

    // Извлекаем заголовок Authorization
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            tracing::warn!("[Middleware] Missing Authorization header for {}", path);
            AppError::Auth("Missing Authorization header".to_string())
        })?;

    // Проверяем формат Bearer token
    if !auth_header.starts_with("Bearer ") {
        tracing::warn!(
            "[Middleware] Invalid Authorization header format (expected 'Bearer <token>') for {}",
            path
        );
        return Err(AppError::Auth(
            "Invalid Authorization header format. Expected 'Bearer <token>'".to_string(),
        ));
    }

    // Извлекаем токен
    let token = auth_header[7..].trim(); // Обрезаем "Bearer " и возможные пробелы

    if token.is_empty() {
        tracing::warn!(
            "[Middleware] Empty token in Authorization header for {}",
            path
        );
        return Err(AppError::Auth(
            "Empty token in Authorization header".to_string(),
        ));
    }

    // Верифицируем токен
    let claims = verify_token(token, &state.config).map_err(|e| {
        tracing::warn!("[Middleware] Token verification failed for {}: {}", path, e);
        e
    })?;

    // Добавляем user_id в extensions для использования в handlers
    request.extensions_mut().insert(AuthState {
        user_id: claims.sub,
    });

    // Продолжаем обработку запроса
    let response = next.run(request).await;

    let status = response.status();
    if !status.is_success() {
        tracing::warn!(
            "[Middleware] Request completed with status {} for user {}",
            status,
            claims.sub
        );
    }

    Ok(response)
}

pub fn extract_user_id(request: &Request) -> Option<Uuid> {
    request
        .extensions()
        .get::<AuthState>()
        .map(|state| state.user_id)
}
