use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use base64::Engine;

use crate::api::AppState;

/// Basic auth для /metrics (если заданы METRICS_AUTH_USER и METRICS_AUTH_PASSWORD)
pub async fn metrics_auth_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let (user, pass) = match (
        state.config.metrics_auth_user.as_deref(),
        state.config.metrics_auth_password.as_deref(),
    ) {
        (Some(u), Some(p)) if !u.is_empty() && !p.is_empty() => (u, p),
        _ => return next.run(request).await,
    };

    let authorized = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Basic "))
        .and_then(|encoded| base64::engine::general_purpose::STANDARD.decode(encoded).ok())
        .and_then(|decoded| String::from_utf8(decoded).ok())
        .map(|credentials| credentials == format!("{}:{}", user, pass))
        .unwrap_or(false);

    if authorized {
        next.run(request).await
    } else {
        (
            StatusCode::UNAUTHORIZED,
            [(header::WWW_AUTHENTICATE, "Basic realm=\"metrics\"")],
            "Unauthorized",
        )
            .into_response()
    }
}
