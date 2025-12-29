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
}

