package com.rimskiy.shared.ui

import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.runtime.*
import com.rimskiy.shared.di.AppModule
import com.rimskiy.shared.ui.navigation.AppNavigation
import com.rimskiy.shared.ui.theme.RimskiyTheme
import com.rimskiy.shared.platform.createSettings
import com.rimskiy.shared.data.local.SettingsManager

/**
 * Главный компонент приложения
 * Применяет SOLID принципы:
 * - Single Responsibility: отвечает только за инициализацию и композицию UI
 * - Dependency Inversion: использует AppModule для получения зависимостей
 */
@Composable
fun RimskiyApp(
    baseUrl: String? = null,
    ddnsUsername: String? = null,
    ddnsPassword: String? = null,
    appVersion: String
) {
    val settingsManager = remember { SettingsManager(createSettings()) }

    // Автоопределение URL сервера
    val initialUrl = remember {
        // Если в настройках другой URL, но в сборке задан baseUrl — принудительно используем baseUrl
        val stored = settingsManager.baseUrl
        val effective = baseUrl ?: stored ?: "http://89.111.169.83:3000"
        if (baseUrl != null && stored != baseUrl) {
            settingsManager.baseUrl = baseUrl
        }
        effective
    }

    var serverUrl by remember { mutableStateOf(initialUrl) }

    LaunchedEffect(serverUrl) {
        println("[RimskiyApp] Selected server URL: $serverUrl")
        settingsManager.baseUrl = serverUrl
    }
    
    // Используем remember для сохранения экземпляра AppModule между рекомпозициями
    val appModule = remember(serverUrl, ddnsUsername, ddnsPassword) { 
        println("[RimskiyApp] Creating AppModule with baseUrl: $serverUrl, ddnsAuth: ${ddnsUsername != null}")
        AppModule(serverUrl, ddnsUsername, ddnsPassword) 
    }
    
    RimskiyTheme {
        AppNavigation(
            currentBaseUrl = serverUrl,
            onChangeBaseUrl = { newUrl ->
                val normalized = newUrl.trim()
                if (normalized.isNotEmpty()) {
                    serverUrl = normalized
                }
            },
            appVersion = appVersion,
            authRepository = appModule.authRepository,
            startAuthUseCase = appModule.startAuthUseCase,
            verifyAuthUseCase = appModule.verifyAuthUseCase,
            getProfileUseCase = appModule.getProfileUseCase,
            updateProfileUseCase = appModule.updateProfileUseCase,
            getMyBlocksUseCase = appModule.getMyBlocksUseCase,
            createBlockUseCase = appModule.createBlockUseCase,
            deleteBlockUseCase = appModule.deleteBlockUseCase,
            warnOwnerUseCase = appModule.warnOwnerUseCase,
            getBlocksForMyPlateUseCase = appModule.getBlocksForMyPlateUseCase,
            logoutUseCase = appModule.logoutUseCase,
            recognizePlateUseCase = appModule.recognizePlateUseCase,
            getNotificationsUseCase = appModule.getNotificationsUseCase,
            markNotificationReadUseCase = appModule.markNotificationReadUseCase,
            markAllNotificationsReadUseCase = appModule.markAllNotificationsReadUseCase,
            getUserPlatesUseCase = appModule.getUserPlatesUseCase,
            createUserPlateUseCase = appModule.createUserPlateUseCase,
            deleteUserPlateUseCase = appModule.deleteUserPlateUseCase,
            setPrimaryPlateUseCase = appModule.setPrimaryPlateUseCase,
            updateUserPlateDepartureUseCase = appModule.updateUserPlateDepartureUseCase,
            getServerInfoUseCase = appModule.getServerInfoUseCase,
            getUserByPlateUseCase = appModule.getUserByPlateUseCase
        )
    }
}
