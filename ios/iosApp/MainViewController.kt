package com.rimskiy.ios

import androidx.compose.ui.window.ComposeUIViewController
import com.rimskiy.shared.ui.RimskiyApp

/**
 * Точка входа для iOS приложения
 * Использует Compose Multiplatform для отображения UI
 */
fun MainViewController() = ComposeUIViewController {
    // TODO: Получить URL из конфигурации или Info.plist
    RimskiyApp(baseUrl = "http://localhost:3000")
}

