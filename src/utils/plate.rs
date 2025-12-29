/// Нормализует номер автомобиля (удаляет пробелы, приводит к верхнему регистру)
pub fn normalize_plate(plate: &str) -> String {
    plate.replace([' ', '-'], "").to_uppercase()
}

/// Проверяет формат российского номера автомобиля
/// Формат: А123БВ777 (1 буква, 3 цифры, 2 буквы, 2-3 цифры)
/// Поддерживает как кириллические, так и латинские буквы
pub fn validate_plate(plate: &str) -> bool {
    let normalized = normalize_plate(plate);

    // Используем chars().count() вместо len() для правильного подсчета символов (не байт)
    let char_count = normalized.chars().count();
    if !(8..=9).contains(&char_count) {
        tracing::warn!(
            "Plate length invalid: {} chars (expected 8-9) for '{}'",
            char_count,
            normalized
        );
        return false;
    }

    // Проверяем формат: буква, 3 цифры, 2 буквы, 2-3 цифры
    let chars: Vec<char> = normalized.chars().collect();

    tracing::info!(
        "Validating plate '{}', length: {}, chars: {:?}",
        normalized,
        chars.len(),
        chars
    );

    // Функция для проверки буквы (кириллица или латиница)
    // В России используются кириллические буквы (А-Я, Ё)
    // Также возможны латинские буквы в некоторых случаях
    let is_letter = |c: &char| -> bool {
        // Проверяем кириллические буквы (А-Я, Ё)
        let code = *c as u32;
        // А = 0x0410, Я = 0x042F, Ё = 0x0401
        let is_cyrillic = (0x0410..=0x042F).contains(&code) || code == 0x0401;
        let is_latin = c.is_ascii_alphabetic();

        if !is_cyrillic && !is_latin {
            tracing::warn!(
                "Character '{}' (U+{:04X}) is not a letter (cyrillic: {}, latin: {})",
                c,
                code,
                is_cyrillic,
                is_latin
            );
        }

        is_cyrillic || is_latin
    };

    // Первая буква
    if !is_letter(&chars[0]) {
        tracing::warn!(
            "First character '{}' (U+{:04X}) is not a letter for plate '{}'",
            chars[0],
            chars[0] as u32,
            normalized
        );
        return false;
    }

    tracing::info!("First char '{}' is a letter", chars[0]);

    // Три цифры
    if !chars[1..4].iter().all(|c| c.is_ascii_digit()) {
        tracing::warn!(
            "Characters 1-3 are not all digits: {:?} for plate '{}'",
            &chars[1..4],
            normalized
        );
        return false;
    }

    tracing::info!("Chars 1-3 are digits: {:?}", &chars[1..4]);

    // Две буквы
    for (i, c) in chars[4..6].iter().enumerate() {
        if !is_letter(c) {
            tracing::warn!(
                "Character {} '{}' (U+{:04X}) at position {} is not a letter for plate '{}'",
                i + 4,
                c,
                *c as u32,
                i + 4,
                normalized
            );
            return false;
        }
    }

    tracing::info!("Chars 4-5 are letters: {:?}", &chars[4..6]);

    // Последние 2-3 цифры
    if !chars[6..].iter().all(|c| c.is_ascii_digit()) {
        tracing::warn!(
            "Last characters are not all digits: {:?} for plate '{}'",
            &chars[6..],
            normalized
        );
        return false;
    }

    tracing::info!("Last chars are digits: {:?}", &chars[6..]);
    tracing::info!("Plate '{}' validated successfully", normalized);
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
