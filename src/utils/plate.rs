/// Нормализует номер автомобиля (удаляет пробелы, приводит к верхнему регистру)
pub fn normalize_plate(plate: &str) -> String {
    plate.replace([' ', '-'], "").to_uppercase()
}

/// Проверяет формат российского номера автомобиля
/// Формат: А123БВ777 (1 буква, 3 цифры, 2 буквы, 2-3 цифры)
/// Поддерживает как кириллические, так и латинские буквы
pub fn validate_plate(plate: &str) -> bool {
    let normalized = normalize_plate(plate);

    let char_count = normalized.chars().count();
    if !(8..=9).contains(&char_count) {
        return false;
    }

    let chars: Vec<char> = normalized.chars().collect();

    let is_letter = |c: &char| -> bool {
        let code = *c as u32;
        let is_cyrillic = (0x0410..=0x042F).contains(&code) || code == 0x0401;
        let is_latin = c.is_ascii_alphabetic();
        is_cyrillic || is_latin
    };

    if !is_letter(&chars[0]) {
        return false;
    }

    if !chars[1..4].iter().all(|c| c.is_ascii_digit()) {
        return false;
    }

    if !chars[4..6].iter().all(is_letter) {
        return false;
    }

    if !chars[6..].iter().all(|c| c.is_ascii_digit()) {
        return false;
    }

    true
}

/// Форматирует номер автомобиля для отображения
/// А123БВ777 -> А 123 БВ 777
pub fn format_plate(plate: &str) -> String {
    let normalized = normalize_plate(plate);

    if normalized.len() == 9 {
        format!(
            "{} {} {} {}",
            &normalized[0..1],
            &normalized[1..4],
            &normalized[4..6],
            &normalized[6..9]
        )
    } else if normalized.len() == 8 {
        format!(
            "{} {} {} {}",
            &normalized[0..1],
            &normalized[1..4],
            &normalized[4..6],
            &normalized[6..8]
        )
    } else {
        normalized
    }
}
