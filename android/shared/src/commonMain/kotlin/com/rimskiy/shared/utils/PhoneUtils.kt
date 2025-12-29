package com.rimskiy.shared.utils

object PhoneUtils {
    fun normalizePhone(phone: String): String {
        var cleaned = phone.filter { it.isDigit() || it == '+' }
        
        // Автозамена 8 или 7 на +7
        when {
            cleaned.startsWith("+7") -> {
                // Уже правильный формат
            }
            cleaned.startsWith("8") && cleaned.length > 1 -> {
                // Заменяем 8 на +7 (для всех номеров, начинающихся с 8)
                cleaned = "+7${cleaned.substring(1)}"
            }
            cleaned.startsWith("7") && cleaned.length > 1 -> {
                // Если номер начинается с 7 (не +7), добавляем +
                cleaned = "+$cleaned"
            }
            cleaned.isNotEmpty() && cleaned.first().isDigit() && !cleaned.startsWith("+") -> {
                // Если номер начинается с цифры (не с +), добавляем +7
                cleaned = "+7$cleaned"
            }
        }
        
        return cleaned
    }

    fun validatePhone(phone: String): Boolean {
        val normalized = normalizePhone(phone)
        return normalized.length >= 10 && 
               (normalized.startsWith("+") || normalized.startsWith("8") || normalized.startsWith("7"))
    }

    fun formatPhone(phone: String): String {
        val normalized = normalizePhone(phone)
        return when {
            normalized.startsWith("+7") && normalized.length == 12 -> {
                "+7 (${normalized.substring(2, 5)}) ${normalized.substring(5, 8)}-${normalized.substring(8, 10)}-${normalized.substring(10, 12)}"
            }
            normalized.startsWith("8") && normalized.length == 11 -> {
                "8 (${normalized.substring(1, 4)}) ${normalized.substring(4, 7)}-${normalized.substring(7, 9)}-${normalized.substring(9, 11)}"
            }
            else -> normalized
        }
    }
}

