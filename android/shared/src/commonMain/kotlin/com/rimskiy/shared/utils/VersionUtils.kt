package com.rimskiy.shared.utils

object VersionUtils {
    // Простое сравнение версий вида "1.2.3"
    fun compare(v1: String?, v2: String?): Int {
        if (v1 == null || v2 == null) return 0
        val a = v1.split(".")
        val b = v2.split(".")
        val max = maxOf(a.size, b.size)
        for (i in 0 until max) {
            val x = a.getOrNull(i)?.toIntOrNull() ?: 0
            val y = b.getOrNull(i)?.toIntOrNull() ?: 0
            if (x != y) return x.compareTo(y)
        }
        return 0
    }
    
    /**
     * Вычисляет следующую версию (увеличивает последний компонент)
     * Например: "1.0.0" -> "1.0.1", "1.2.3" -> "1.2.4"
     */
    fun incrementVersion(version: String): String {
        val parts = version.split(".")
        if (parts.isEmpty()) return version
        
        val lastIndex = parts.size - 1
        val lastPart = parts[lastIndex].toIntOrNull() ?: 0
        val incremented = lastPart + 1
        
        val newParts = parts.toMutableList()
        newParts[lastIndex] = incremented.toString()
        
        return newParts.joinToString(".")
    }
    
    /**
     * Проверяет, является ли версия следующей после текущей (увеличение на 1 в последнем компоненте)
     */
    fun isNextVersion(current: String, candidate: String): Boolean {
        val nextVersion = incrementVersion(current)
        return compare(candidate, nextVersion) == 0
    }
}

