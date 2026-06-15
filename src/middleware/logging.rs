use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;

use crate::metrics;

pub async fn logging_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path().to_string();
    let query = uri.query().unwrap_or("");

    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .or_else(|| request.headers().get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    tracing::info!(
        method = %method,
        path = %path,
        query = %query,
        client_ip = %client_ip,
        "→ Incoming API request"
    );

    let start = Instant::now();
    let response = next.run(request).await;
    let duration = start.elapsed();
    let status = response.status().as_u16();

    metrics::record_request(method.as_str(), &path, status, duration.as_secs_f64());

    tracing::info!(
        method = %method,
        path = %path,
        status = %status,
        duration_ms = duration.as_millis(),
        "← API response"
    );

    response
}
