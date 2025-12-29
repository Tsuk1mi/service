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
    ddnsPassword: String? = null
) {
    val settingsManager = remember { SettingsManager(createSettings()) }

    // Автоопределение URL сервера
    val initialUrl = remember {
        settingsManager.baseUrl ?: baseUrl ?: "http://10.0.2.2:3000"
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
            checkBlockUseCase = appModule.checkBlockUseCase,
            logoutUseCase = appModule.logoutUseCase,
            recognizePlateUseCase = appModule.recognizePlateUseCase,
            getNotificationsUseCase = appModule.getNotificationsUseCase,
            markNotificationReadUseCase = appModule.markNotificationReadUseCase,
            markAllNotificationsReadUseCase = appModule.markAllNotificationsReadUseCase,
            getUserPlatesUseCase = appModule.getUserPlatesUseCase,
            createUserPlateUseCase = appModule.createUserPlateUseCase,
            deleteUserPlateUseCase = appModule.deleteUserPlateUseCase,
            setPrimaryPlateUseCase = appModule.setPrimaryPlateUseCase,
            getUserByPlateUseCase = appModule.getUserByPlateUseCase
        )
    }
}
