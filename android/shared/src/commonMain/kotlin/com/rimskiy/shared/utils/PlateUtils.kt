package com.rimskiy.shared.utils

object PlateUtils {
    fun normalizePlate(plate: String): String {
        return plate.replace(" ", "").replace("-", "").uppercase()
    }

    fun validatePlate(plate: String): Boolean {
        val normalized = normalizePlate(plate)
        if (normalized.length < 8 || normalized.length > 9) return false
        
        val chars = normalized.toCharArray()
        
        // Первая буква
        if (!chars[0].isLetter()) return false
        
        // Три цифры
        if (!chars[1].isDigit() || !chars[2].isDigit() || !chars[3].isDigit()) return false
        
        // Две буквы
        if (!chars[4].isLetter() || !chars[5].isLetter()) return false
        
        // Последние 2-3 цифры
        val remaining = normalized.substring(6)
        return remaining.all { it.isDigit() }
    }

    fun formatPlate(plate: String): String {
        val normalized = normalizePlate(plate)
        return when (normalized.length) {
            9 -> "${normalized[0]} ${normalized.substring(1, 4)} ${normalized.substring(4, 6)} ${normalized.substring(6, 9)}"
            8 -> "${normalized[0]} ${normalized.substring(1, 4)} ${normalized.substring(4, 6)} ${normalized.substring(6, 8)}"
            else -> normalized
        }
    }
}

