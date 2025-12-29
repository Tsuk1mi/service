package com.rimskiy.shared.di

import android.content.Context
import com.rimskiy.shared.platform.PlatformActions

// В Android это будет получаться через Application Context
// Для упрощения используем singleton
object AndroidContextHolder {
    var context: Context? = null
}

actual fun getPlatformActions(): PlatformActions {
    val context = AndroidContextHolder.context
        ?: throw IllegalStateException("Android Context not initialized")
    // AndroidActions - это actual class PlatformActions
    return com.rimskiy.shared.platform.PlatformActions(context)
}

