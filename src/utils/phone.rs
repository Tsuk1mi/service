use sha2::{Digest, Sha256};

/// SHA-256 хэш нормализованного номера телефона (для поиска без расшифровки)
pub fn phone_hash(phone: &str) -> String {
    let normalized = normalize_phone(phone);
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Нормализует номер телефона (удаляет пробелы, дефисы, скобки)
/// Автоматически заменяет 8 или 7 на +7
pub fn normalize_phone(phone: &str) -> String {
    let cleaned: String = phone
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '+')
        .collect();

    // Автозамена 8 или 7 на +7
    if cleaned.starts_with("+7") {
        // Уже правильный формат
        cleaned
    } else if cleaned.starts_with('8') && cleaned.len() > 1 {
        // Заменяем 8 на +7 (для всех номеров, начинающихся с 8)
        format!("+7{}", &cleaned[1..])
    } else if cleaned.starts_with('7') && cleaned.len() > 1 {
        // Если номер начинается с 7 (не +7), добавляем +
        format!("+{}", cleaned)
    } else if !cleaned.is_empty()
        && cleaned
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        && !cleaned.starts_with('+')
    {
        // Если номер начинается с цифры (не с +), добавляем +7
        format!("+7{}", cleaned)
    } else {
        cleaned
    }
}

/// Проверяет формат номера телефона
pub fn validate_phone(phone: &str) -> bool {
    let normalized = normalize_phone(phone);
    // Российские номера: +7XXXXXXXXXX или 8XXXXXXXXXX
    // Международные: начинаются с +
    normalized.len() >= 10
        && (normalized.starts_with('+')
            || normalized.starts_with('8')
            || normalized.starts_with('7'))
}

/// Форматирует номер телефона для отображения
pub fn format_phone(phone: &str) -> String {
    let normalized = normalize_phone(phone);

    if normalized.starts_with("+7") && normalized.len() == 12 {
        // +7 (XXX) XXX-XX-XX
        format!(
            "+7 ({}) {}-{}-{}",
            &normalized[2..5],
            &normalized[5..8],
            &normalized[8..10],
            &normalized[10..12]
        )
    } else if normalized.starts_with("8") && normalized.len() == 11 {
        format!(
            "8 ({}) {}-{}-{}",
            &normalized[1..4],
            &normalized[4..7],
            &normalized[7..9],
            &normalized[9..11]
        )
    } else {
        normalized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phone_hash_is_deterministic() {
        let h1 = phone_hash("+79001234567");
        let h2 = phone_hash("8 900 123-45-67");
        assert_eq!(h1, h2);
    }

    #[test]
    fn normalize_russian_phone() {
        assert_eq!(normalize_phone("89001234567"), "+79001234567");
        assert_eq!(normalize_phone("+79001234567"), "+79001234567");
    }
}
