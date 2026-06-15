use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// Тип-алиас для результата операций приложения
pub type AppResult<T> = Result<T, AppError>;

/// Ошибки приложения
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl AppError {
    /// Публичное сообщение без внутренних деталей
    pub fn public_message(&self) -> String {
        match self {
            AppError::Database(_) => "Database error".to_string(),
            AppError::Auth(msg) => msg.clone(),
            AppError::Validation(msg) => msg.clone(),
            AppError::NotFound(msg) => msg.clone(),
            AppError::Encryption(_) => "Encryption error".to_string(),
            AppError::RateLimit(msg) => msg.clone(),
            AppError::Internal(_) => "Internal server error".to_string(),
        }
    }

    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::Database(_) | AppError::Encryption(_) | AppError::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::Auth(_) => StatusCode::UNAUTHORIZED,
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::RateLimit(_) => StatusCode::TOO_MANY_REQUESTS,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let error_details = self.to_string();
        let status = self.status_code();
        let error_message = self.public_message();

        match &self {
            AppError::Database(e) => tracing::error!("Database error: {}", e),
            AppError::Encryption(msg) => tracing::error!("Encryption error: {}", msg),
            AppError::Internal(msg) => tracing::error!("Internal error: {}", msg),
            AppError::RateLimit(msg) => tracing::warn!("Rate limit: {}", msg),
            _ => {}
        }

        let expose_details = std::env::var("APP_ENV")
            .map(|e| e.to_lowercase() != "production" && e.to_lowercase() != "prod")
            .unwrap_or(true);

        let body = if expose_details {
            Json(json!({
                "error": error_message,
                "details": error_details
            }))
        } else {
            Json(json!({
                "error": error_message
            }))
        };

        (status, body).into_response()
    }
}
