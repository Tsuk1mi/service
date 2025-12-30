package com.rimskiy.shared.ui.navigation

import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.rimskiy.shared.di.getPlatformActions
import com.rimskiy.shared.domain.repository.IAuthRepository
import com.rimskiy.shared.domain.usecase.*
import com.rimskiy.shared.ui.screens.*
import kotlinx.coroutines.launch

sealed class Screen {
    object Auth : Screen()
    object Home : Screen()
    object Profile : Screen()
    object MyBlocks : Screen()
    object BlockedBy : Screen()
    object BlockNotification : Screen()
    object About : Screen()
    data class BlockNotificationDetails(val block: com.rimskiy.shared.data.model.BlockWithBlockerInfo) : Screen()
}

enum class BottomNavItem(
    val title: String,
    val shortTitle: String,
    val icon: androidx.compose.ui.graphics.vector.ImageVector,
    val screen: Screen
) {
    Home("Главное", "Главная", Icons.Default.Home, Screen.Home),
    Profile("Профиль", "Профиль", Icons.Default.Person, Screen.Profile),
    MyBlocks("Мои блокировки", "Блокировки", Icons.Default.List, Screen.MyBlocks),
    BlockedBy("Меня заблокировали", "Заблокировали", Icons.Default.Warning, Screen.BlockedBy),
    Notifications("Уведомления", "Уведомления", Icons.Default.Notifications, Screen.BlockNotification)
}

@Composable
fun AppNavigation(
    currentBaseUrl: String,
    onChangeBaseUrl: (String) -> Unit,
    appVersion: String,
    authRepository: IAuthRepository,
    startAuthUseCase: StartAuthUseCase,
    verifyAuthUseCase: VerifyAuthUseCase,
    getProfileUseCase: GetProfileUseCase,
    updateProfileUseCase: UpdateProfileUseCase,
    getMyBlocksUseCase: GetMyBlocksUseCase,
    createBlockUseCase: CreateBlockUseCase,
    deleteBlockUseCase: DeleteBlockUseCase,
    warnOwnerUseCase: WarnOwnerUseCase,
    getBlocksForMyPlateUseCase: GetBlocksForMyPlateUseCase,
    logoutUseCase: LogoutUseCase,
    recognizePlateUseCase: RecognizePlateUseCase,
    getNotificationsUseCase: GetNotificationsUseCase,
    markNotificationReadUseCase: MarkNotificationReadUseCase,
    markAllNotificationsReadUseCase: MarkAllNotificationsReadUseCase,
    getUserPlatesUseCase: GetUserPlatesUseCase,
    createUserPlateUseCase: CreateUserPlateUseCase,
    deleteUserPlateUseCase: DeleteUserPlateUseCase,
    setPrimaryPlateUseCase: SetPrimaryPlateUseCase,
    updateUserPlateDepartureUseCase: UpdateUserPlateDepartureUseCase,
    getServerInfoUseCase: GetServerInfoUseCase,
    getUserByPlateUseCase: GetUserByPlateUseCase
) {
    // Состояние проверки авторизации
    var isCheckingAuth by remember { mutableStateOf(true) }
    var currentScreen by remember { mutableStateOf<Screen>(Screen.Auth) }
    var showUpdateDialog by remember { mutableStateOf(false) }
    var showOptionalUpdateDialog by remember { mutableStateOf(false) }
    var minRequiredVersion by remember { mutableStateOf<String?>(null) }
    var releaseVersion by remember { mutableStateOf<String?>(null) }
    var downloadUrl by remember { mutableStateOf<String?>(null) }
    var telegramBotUsername by remember { mutableStateOf<String?>(null) }
    var isForceUpdate by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()
    
    // Функция проверки версии и обновления
    fun checkVersionAndUpdate(info: com.rimskiy.shared.data.model.ServerInfoResponse?) {
        scope.launch {
            getServerInfoUseCase().fold(
                onSuccess = { serverInfo ->
                    val infoToCheck = info ?: serverInfo
                    downloadUrl = infoToCheck.app_download_url
                    telegramBotUsername = infoToCheck.telegram_bot_username
                    
                    // Проверяем обязательное обновление (min_client_version)
                    val minVersion = infoToCheck.min_client_version
                    if (minVersion != null && com.rimskiy.shared.utils.VersionUtils.compare(appVersion, minVersion) < 0) {
                        minRequiredVersion = minVersion
                        isForceUpdate = true
                        showUpdateDialog = true
                        showOptionalUpdateDialog = false
                        return@fold
                    }
                    
                    // Проверяем опциональное обновление (release_client_version)
                    val releaseVersionValue = infoToCheck.release_client_version
                    if (releaseVersionValue != null && com.rimskiy.shared.utils.VersionUtils.compare(appVersion, releaseVersionValue) < 0) {
                        releaseVersion = releaseVersionValue
                        // Показываем опциональное обновление только если нет обязательного
                        if (!showUpdateDialog) {
                            showOptionalUpdateDialog = true
                        }
                    }
                },
                onFailure = { e ->
                    println("[AppNavigation] Failed to fetch server info: ${e.message}")
                }
            )
        }
    }
    
    // Проверяем токен при первом запуске
    LaunchedEffect(Unit) {
        println("[AppNavigation] Checking authentication state...")
        isCheckingAuth = true

        // Версионная проверка
        checkVersionAndUpdate(info = null)
        
        // Если токен отсутствует, сразу показываем экран авторизации
        if (!authRepository.isAuthenticated()) {
            println("[AppNavigation] No token found, showing auth screen")
            currentScreen = Screen.Auth
            isCheckingAuth = false
            return@LaunchedEffect
        }
        
        // Если токен есть, проверяем его валидность на сервере
        println("[AppNavigation] Token found, validating on server...")
        getProfileUseCase().fold(
            onSuccess = { profile ->
                println("[AppNavigation] Token is valid, showing home screen")
                currentScreen = Screen.Home
            },
            onFailure = { error ->
                println("[AppNavigation] Token validation failed: ${error.message}, showing auth screen")
                // Если токен невалиден, очищаем его и показываем экран авторизации
                scope.launch {
                    authRepository.logout()
                }
                currentScreen = Screen.Auth
            }
        )
        isCheckingAuth = false
    }
    
    // Определяем текущий элемент нижней навигации
    val currentBottomNavItem = remember(currentScreen) {
        BottomNavItem.values().find { it.screen == currentScreen }
    }
    var selectedBottomNavItem by remember { mutableStateOf(currentBottomNavItem ?: BottomNavItem.Home) }
    
    // Ключ для принудительного обновления экранов при навигации
    var screenRefreshKey by remember { mutableStateOf(0) }
    
    // Обновляем ключ при изменении экрана для принудительного обновления данных
    LaunchedEffect(currentScreen) {
        screenRefreshKey++
    }
    
    // Периодическая проверка версии (каждые 5 минут)
    LaunchedEffect(Unit) {
        while (true) {
            kotlinx.coroutines.delay(5 * 60 * 1000) // 5 минут
            if (!showUpdateDialog && !isCheckingAuth) {
                checkVersionAndUpdate(info = null)
            }
        }
    }

    // Показываем индикатор загрузки во время проверки авторизации
    if (isCheckingAuth) {
        Box(
            modifier = Modifier.fillMaxSize(),
            contentAlignment = Alignment.Center
        ) {
            CircularProgressIndicator()
        }
        return
    }

    if (showUpdateDialog) {
        val uriHandler = androidx.compose.ui.platform.LocalUriHandler.current
        AlertDialog(
            onDismissRequest = { /* блокируем закрытие - требуется обновление */ },
            icon = {
                Icon(
                    imageVector = androidx.compose.material.icons.Icons.Default.Warning,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.error,
                    modifier = Modifier.size(40.dp)
                )
            },
            title = { 
                Text(
                    text = "Требуется обновление",
                    style = MaterialTheme.typography.titleLarge
                )
            },
            text = {
                Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
                    Text(
                        text = "Для работы приложения требуется версия не ниже $minRequiredVersion.",
                        style = MaterialTheme.typography.bodyMedium
                    )
                    
                    ElevatedCard(
                        modifier = Modifier.fillMaxWidth(),
                        elevation = CardDefaults.elevatedCardElevation(defaultElevation = 2.dp),
                        colors = CardDefaults.cardColors(
                            containerColor = MaterialTheme.colorScheme.errorContainer.copy(alpha = 0.2f)
                        )
                    ) {
                        Column(
                            modifier = Modifier.padding(16.dp),
                            verticalArrangement = Arrangement.spacedBy(8.dp)
                        ) {
                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                horizontalArrangement = Arrangement.SpaceBetween
                            ) {
                                Text(
                                    text = "Текущая версия:",
                                    style = MaterialTheme.typography.labelMedium,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant
                                )
                                Text(
                                    text = appVersion,
                                    style = MaterialTheme.typography.titleMedium,
                                    color = MaterialTheme.colorScheme.error
                                )
                            }
                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                horizontalArrangement = Arrangement.SpaceBetween
                            ) {
                                Text(
                                    text = "Требуется версия:",
                                    style = MaterialTheme.typography.labelMedium,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant
                                )
                                Text(
                                    text = minRequiredVersion ?: "",
                                    style = MaterialTheme.typography.titleMedium,
                                    color = MaterialTheme.colorScheme.primary
                                )
                            }
                        }
                    }
                    
                    downloadUrl?.let { url ->
                        ElevatedCard(
                            modifier = Modifier.fillMaxWidth(),
                            elevation = CardDefaults.elevatedCardElevation(defaultElevation = 1.dp),
                            colors = CardDefaults.cardColors(
                                containerColor = MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.3f)
                            )
                        ) {
                            Row(
                                modifier = Modifier.padding(12.dp),
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(8.dp)
                            ) {
                                Icon(
                                    imageVector = androidx.compose.material.icons.Icons.Default.Link,
                                    contentDescription = null,
                                    tint = MaterialTheme.colorScheme.primary,
                                    modifier = Modifier.size(18.dp)
                                )
                                Text(
                                    text = url,
                                    style = MaterialTheme.typography.bodySmall,
                                    color = MaterialTheme.colorScheme.primary,
                                    maxLines = 2,
                                    overflow = androidx.compose.ui.text.style.TextOverflow.Ellipsis
                                )
                            }
                        }
                    }
                    
                    Text(
                        text = "Пожалуйста, обновите приложение для продолжения работы.",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            },
            confirmButton = {
                downloadUrl?.let { url ->
                    Button(
                        onClick = { 
                            uriHandler.openUri(url)
                            showUpdateDialog = false
                        },
                        colors = ButtonDefaults.buttonColors(
                            containerColor = MaterialTheme.colorScheme.primary
                        ),
                        modifier = Modifier.fillMaxWidth()
                    ) {
                        Icon(
                            imageVector = androidx.compose.material.icons.Icons.Default.Update,
                            contentDescription = null,
                            modifier = Modifier.size(18.dp)
                        )
                        Spacer(modifier = Modifier.width(8.dp))
                        Text("Обновить приложение")
                    }
                } ?: run {
                    OutlinedButton(
                        onClick = { /* нельзя закрыть без обновления */ },
                        modifier = Modifier.fillMaxWidth()
                    ) {
                        Text("Ожидание...")
                    }
                }
            },
            dismissButton = null, // Убираем кнопку закрытия - требуется обновление
            containerColor = MaterialTheme.colorScheme.surface,
            shape = MaterialTheme.shapes.large
        )
    }
    
    // Диалог опционального обновления
    if (showOptionalUpdateDialog) {
        val uriHandler = androidx.compose.ui.platform.LocalUriHandler.current
        AlertDialog(
            onDismissRequest = { showOptionalUpdateDialog = false },
            icon = {
                Icon(
                    imageVector = androidx.compose.material.icons.Icons.Default.Info,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.primary,
                    modifier = Modifier.size(40.dp)
                )
            },
            title = { 
                Text(
                    text = "Доступно обновление",
                    style = MaterialTheme.typography.titleLarge
                )
            },
            text = {
                Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
                    Text(
                        text = "Доступна новая версия приложения: $releaseVersion",
                        style = MaterialTheme.typography.bodyMedium
                    )
                    
                    ElevatedCard(
                        modifier = Modifier.fillMaxWidth(),
                        elevation = CardDefaults.elevatedCardElevation(defaultElevation = 2.dp),
                        colors = CardDefaults.cardColors(
                            containerColor = MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.2f)
                        )
                    ) {
                        Column(
                            modifier = Modifier.padding(16.dp),
                            verticalArrangement = Arrangement.spacedBy(8.dp)
                        ) {
                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                horizontalArrangement = Arrangement.SpaceBetween
                            ) {
                                Text(
                                    text = "Текущая версия:",
                                    style = MaterialTheme.typography.labelMedium,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant
                                )
                                Text(
                                    text = appVersion,
                                    style = MaterialTheme.typography.titleMedium,
                                    color = MaterialTheme.colorScheme.onSurface
                                )
                            }
                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                horizontalArrangement = Arrangement.SpaceBetween
                            ) {
                                Text(
                                    text = "Новая версия:",
                                    style = MaterialTheme.typography.labelMedium,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant
                                )
                                Text(
                                    text = releaseVersion ?: "",
                                    style = MaterialTheme.typography.titleMedium,
                                    color = MaterialTheme.colorScheme.primary
                                )
                            }
                        }
                    }
                    
                    downloadUrl?.let { url ->
                        ElevatedCard(
                            modifier = Modifier.fillMaxWidth(),
                            elevation = CardDefaults.elevatedCardElevation(defaultElevation = 1.dp),
                            colors = CardDefaults.cardColors(
                                containerColor = MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.3f)
                            )
                        ) {
                            Row(
                                modifier = Modifier.padding(12.dp),
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(8.dp)
                            ) {
                                Icon(
                                    imageVector = androidx.compose.material.icons.Icons.Default.Link,
                                    contentDescription = null,
                                    tint = MaterialTheme.colorScheme.primary,
                                    modifier = Modifier.size(18.dp)
                                )
                                Text(
                                    text = url,
                                    style = MaterialTheme.typography.bodySmall,
                                    color = MaterialTheme.colorScheme.primary,
                                    maxLines = 2,
                                    overflow = androidx.compose.ui.text.style.TextOverflow.Ellipsis
                                )
                            }
                        }
                    }
                    
                    Text(
                        text = "Рекомендуем обновить приложение для получения новых функций и исправлений.",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            },
            confirmButton = {
                downloadUrl?.let { url ->
                    Button(
                        onClick = { 
                            uriHandler.openUri(url)
                            showOptionalUpdateDialog = false
                        },
                        colors = ButtonDefaults.buttonColors(
                            containerColor = MaterialTheme.colorScheme.primary
                        ),
                        modifier = Modifier.fillMaxWidth()
                    ) {
                        Icon(
                            imageVector = androidx.compose.material.icons.Icons.Default.Update,
                            contentDescription = null,
                            modifier = Modifier.size(18.dp)
                        )
                        Spacer(modifier = Modifier.width(8.dp))
                        Text("Обновить")
                    }
                } ?: run {
                    OutlinedButton(
                        onClick = { showOptionalUpdateDialog = false },
                        modifier = Modifier.fillMaxWidth()
                    ) {
                        Text("Позже")
                    }
                }
            },
            dismissButton = {
                TextButton(onClick = { showOptionalUpdateDialog = false }) {
                    Text("Позже")
                }
            },
            containerColor = MaterialTheme.colorScheme.surface,
            shape = MaterialTheme.shapes.large
        )
    }
    
    when (currentScreen) {
        is Screen.Auth -> {
            AuthScreen(
                onAuthSuccess = { 
                    currentScreen = Screen.Home
                    selectedBottomNavItem = BottomNavItem.Home
                },
                startAuthUseCase = startAuthUseCase,
                verifyAuthUseCase = verifyAuthUseCase,
                currentBaseUrl = currentBaseUrl,
                onChangeBaseUrl = { newUrl ->
                    scope.launch {
                        authRepository.logout()
                    }
                    currentScreen = Screen.Auth
                    selectedBottomNavItem = BottomNavItem.Profile
                    onChangeBaseUrl(newUrl)
                }
            )
        }
        is Screen.BlockNotificationDetails -> {
            val platformActions = remember { getPlatformActions() }
            val blockDetails = (currentScreen as Screen.BlockNotificationDetails).block
            BlockNotificationDetailsScreen(
                block = blockDetails,
                onNavigateBack = { 
                    currentScreen = Screen.BlockNotification
                    selectedBottomNavItem = BottomNavItem.Notifications
                },
                platformActions = platformActions
            )
        }
        else -> {
            // Основные экраны с нижней навигацией
            Scaffold(
                bottomBar = {
                    NavigationBar(
                        modifier = Modifier.fillMaxWidth(),
                        containerColor = MaterialTheme.colorScheme.surface,
                        contentColor = MaterialTheme.colorScheme.onSurface
                    ) {
                        BottomNavItem.values().forEach { item ->
                            NavigationBarItem(
                                icon = { Icon(item.icon, contentDescription = item.title) },
                                label = {
                                    Text(
                                        text = item.shortTitle,
                                        style = MaterialTheme.typography.labelSmall,
                                        maxLines = 1,
                                        overflow = TextOverflow.Ellipsis
                                    )
                                },
                                selected = selectedBottomNavItem == item,
                                onClick = {
                                    selectedBottomNavItem = item
                                    currentScreen = item.screen
                                },
                                colors = NavigationBarItemDefaults.colors(
                                    selectedIconColor = MaterialTheme.colorScheme.primary,
                                    selectedTextColor = MaterialTheme.colorScheme.primary,
                                    indicatorColor = MaterialTheme.colorScheme.primaryContainer
                                )
                            )
                        }
                    }
                }
            ) { paddingValues ->
                Box(modifier = Modifier.padding(paddingValues)) {
                    when (currentScreen) {
                        is Screen.Home -> {
                            key(Screen.Home, screenRefreshKey) {
                                HomeScreen(
                                    appVersion = appVersion,
                                    minRequiredVersion = minRequiredVersion,
                                    downloadUrl = downloadUrl,
                                    onNavigateToProfile = {
                                        currentScreen = Screen.Profile
                                        selectedBottomNavItem = BottomNavItem.Profile
                                    },
                                    onNavigateToMyBlocks = {
                                        currentScreen = Screen.MyBlocks
                                        selectedBottomNavItem = BottomNavItem.MyBlocks
                                    },
                                    onNavigateToBlockedBy = {
                                        currentScreen = Screen.BlockedBy
                                        selectedBottomNavItem = BottomNavItem.BlockedBy
                                    },
                                    onNavigateToNotifications = {
                                        currentScreen = Screen.BlockNotification
                                        selectedBottomNavItem = BottomNavItem.Notifications
                                    },
                                    onNavigateToAbout = {
                                        currentScreen = Screen.About
                                    }
                                )
                            }
                        }
                        is Screen.Profile -> {
                            // Используем ключ для обновления при возврате на экран
                            key(Screen.Profile, screenRefreshKey) {
                                ProfileScreen(
                                    onNavigateToMyBlocks = { 
                                        currentScreen = Screen.MyBlocks
                                        selectedBottomNavItem = BottomNavItem.MyBlocks
                                    },
                                    onNavigateToBlockedBy = { 
                                        currentScreen = Screen.BlockedBy
                                        selectedBottomNavItem = BottomNavItem.BlockedBy
                                    },
                                    onNavigateToBlockNotification = { 
                                        currentScreen = Screen.BlockNotification
                                        selectedBottomNavItem = BottomNavItem.Notifications
                                    },
                                    onLogout = {
                                        scope.launch {
                                            logoutUseCase()
                                            currentScreen = Screen.Auth
                                        }
                                    },
                                    getProfileUseCase = getProfileUseCase,
                                    updateProfileUseCase = updateProfileUseCase,
                                    getUserPlatesUseCase = getUserPlatesUseCase,
                                    createUserPlateUseCase = createUserPlateUseCase,
                                    deleteUserPlateUseCase = deleteUserPlateUseCase,
                                    setPrimaryPlateUseCase = setPrimaryPlateUseCase,
                                    updateUserPlateDepartureUseCase = updateUserPlateDepartureUseCase,
                                    recognizePlateUseCase = recognizePlateUseCase,
                                    platformActions = remember { getPlatformActions() },
                                    screenRefreshKey = screenRefreshKey
                                )
                            }
                        }
                        is Screen.MyBlocks -> {
                            // Используем ключ для обновления при возврате на экран
                            key(Screen.MyBlocks, screenRefreshKey) {
                                val platformActions = remember { getPlatformActions() }
                                MyBlocksScreen(
                                    onNavigateBack = { 
                                        currentScreen = Screen.Home
                                        selectedBottomNavItem = BottomNavItem.Home
                                    },
                                    getMyBlocksUseCase = getMyBlocksUseCase,
                                    createBlockUseCase = createBlockUseCase,
                                    deleteBlockUseCase = deleteBlockUseCase,
                                    warnOwnerUseCase = warnOwnerUseCase,
                                    recognizePlateUseCase = recognizePlateUseCase,
                                    getProfileUseCase = getProfileUseCase,
                                    getUserByPlateUseCase = getUserByPlateUseCase,
                                    platformActions = platformActions
                                )
                            }
                        }
                        is Screen.BlockedBy -> {
                            val platformActions = remember { getPlatformActions() }
                            // Используем ключ для обновления при возврате на экран
                            key(Screen.BlockedBy, screenRefreshKey) {
                                BlockedByScreen(
                                    onNavigateBack = { 
                                        currentScreen = Screen.Home
                                        selectedBottomNavItem = BottomNavItem.Home
                                    },
                                    getBlocksForMyPlateUseCase = getBlocksForMyPlateUseCase,
                                    platformActions = platformActions
                                )
                            }
                        }
                        is Screen.BlockNotification -> {
                            key(Screen.BlockNotification, screenRefreshKey) {
                                BlockNotificationScreen(
                                    onNavigateBack = { 
                                        currentScreen = Screen.Home
                                        selectedBottomNavItem = BottomNavItem.Home
                                    },
                                    getNotificationsUseCase = getNotificationsUseCase,
                                    markNotificationReadUseCase = markNotificationReadUseCase,
                                    markAllNotificationsReadUseCase = markAllNotificationsReadUseCase,
                                    screenRefreshKey = screenRefreshKey
                                )
                            }
                        }
                        is Screen.About -> {
                            AboutScreen(
                                appVersion = appVersion,
                                minRequiredVersion = minRequiredVersion,
                                downloadUrl = downloadUrl,
                                currentBaseUrl = currentBaseUrl,
                                onBack = {
                                    currentScreen = Screen.Home
                                    selectedBottomNavItem = BottomNavItem.Home
                                }
                            )
                        }
                        else -> {}
                    }
                }
            }
        }
    }
}

