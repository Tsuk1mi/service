package com.rimskiy.shared.platform

import android.content.Context
import com.russhwolf.settings.Settings
import com.russhwolf.settings.SharedPreferencesSettings

actual fun createSettings(): Settings {
    // Используем SharedPreferences через multiplatform-settings
    // Context будет получен через AndroidContextHolder
    val context = com.rimskiy.shared.di.AndroidContextHolder.context
        ?: throw IllegalStateException("Android Context not initialized")
    return SharedPreferencesSettings(context.getSharedPreferences("rimskiy_prefs", Context.MODE_PRIVATE))
}

actual fun getPlatformName(): String = "Android"
