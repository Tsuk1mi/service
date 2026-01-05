/// Определяет локальный IP адрес для доступа с мобильных устройств
///
/// Сначала проверяет переменную окружения `SERVER_HOST`. Если она не задана или содержит
/// служебные значения (0.0.0.0, 127.0.0.1, localhost), возвращает fallback значение.
pub fn get_local_ip() -> Option<String> {
    if let Ok(ip) = std::env::var("SERVER_HOST") {
        if ip != "0.0.0.0" && ip != "127.0.0.1" && ip != "localhost" {
            return Some(ip);
        }
    }

    Some("192.168.1.1".to_string())
}

/// Формирует полный URL сервера для доступа с мобильных устройств
pub fn get_server_url(port: u16) -> String {
    let ip = get_local_ip().unwrap_or_else(|| "192.168.1.1".to_string());
    format!("http://{}:{}", ip, port)
}
