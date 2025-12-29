package com.rimskiy.shared.utils

object DateUtils {
    fun formatDateShort(isoString: String?): String {
        if (isoString.isNullOrBlank()) return "-"
        return try {
            // Пробуем укоротить ISO строку
            isoString.substring(0, 16).replace('T', ' ')
        } catch (_: Exception) {
            isoString
        }
    }
}

