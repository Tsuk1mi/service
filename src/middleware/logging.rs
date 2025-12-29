use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;

/// Middleware для логирования всех входящих API запросов
pub async fn logging_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path();
    let query = uri.query().unwrap_or("");

    // Получаем IP адрес клиента (если доступен)
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .or_else(|| request.headers().get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    // Логируем входящий запрос с полной информацией
    tracing::info!(
        method = %method,
        path = %path,
        query = %query,
        client_ip = %client_ip,
        "→ Incoming API request"
    );

    // Также выводим в stdout для гарантированного логирования
    println!(
        "[SERVER] {} {} {} (from: {})",
        method, path, query, client_ip
    );

    let start = Instant::now();
    let response = next.run(request).await;
    let duration = start.elapsed();

    let status = response.status();

    // Логируем ответ
    tracing::info!(
        method = %method,
        path = %path,
        status = %status.as_u16(),
        duration_ms = duration.as_millis(),
        "← API response"
    );

    // Также выводим в stdout
    println!(
        "[SERVER] {} {} -> {} ({}ms)",
        method,
        path,
        status.as_u16(),
        duration.as_millis()
    );

    response
}
