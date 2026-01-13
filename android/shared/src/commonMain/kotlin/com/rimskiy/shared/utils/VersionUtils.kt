package com.rimskiy.shared.utils

object VersionUtils {
    // Сравнение версий клиента.
    // Важно: поддерживаем строки вроде "v1.2.3", "1.2.3+45", "1.2.3 (45)"
    // (берём первые 3 числовых компонента).
    fun compare(v1: String?, v2: String?): Int {
        val a = parse3(v1)
        val b = parse3(v2)
        for (i in 0..2) {
            if (a[i] != b[i]) return a[i].compareTo(b[i])
        }
        return 0
    }

    private fun parse3(v: String?): IntArray {
        if (v.isNullOrBlank()) return intArrayOf(0, 0, 0)
        val nums = Regex("\\d+").findAll(v).mapNotNull { it.value.toIntOrNull() }.toList()
        return intArrayOf(
            nums.getOrNull(0) ?: 0,
            nums.getOrNull(1) ?: 0,
            nums.getOrNull(2) ?: 0
        )
    }
    
    /**
     * Вычисляет следующую версию (увеличивает последний компонент)
     * Например: "1.0.0" -> "1.0.1", "1.2.3" -> "1.2.4"
     */
    fun incrementVersion(version: String): String {
        val parsed = parse3(version)
        parsed[2] = parsed[2] + 1
        return "${parsed[0]}.${parsed[1]}.${parsed[2]}"
    }
    
    /**
     * Проверяет, является ли версия следующей после текущей (увеличение на 1 в последнем компоненте)
     */
    fun isNextVersion(current: String, candidate: String): Boolean {
        val nextVersion = incrementVersion(current)
        return compare(candidate, nextVersion) == 0
    }
}

