/// Автоматически определяет локальный IP адрес для доступа с мобильных устройств
/// Упрощённая версия - использует переменные окружения или fallback
pub fn get_local_ip() -> Option<String> {
    // Пробуем получить IP из переменной окружения
    if let Ok(ip) = std::env::var("SERVER_HOST") {
        if ip != "0.0.0.0" && ip != "127.0.0.1" && ip != "localhost" {
            return Some(ip);
        }
    }

    // Fallback: возвращаем 192.168.1.1 (типичный адрес роутера)
    // В реальном приложении здесь можно использовать get_if_addrs для автоматического определения
    Some("192.168.1.1".to_string())
}

/// Получает полный URL сервера для мобильных устройств
pub fn get_server_url(port: u16) -> String {
    let ip = get_local_ip().unwrap_or_else(|| "192.168.1.1".to_string());
    format!("http://{}:{}", ip, port)
}
