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
    var minRequiredVersion by remember { mutableStateOf<String?>(null) }
    var downloadUrl by remember { mutableStateOf<String?>(null) }
    val scope = rememberCoroutineScope()
    
    // Проверяем токен при первом запуске
    LaunchedEffect(Unit) {
        println("[AppNavigation] Checking authentication state...")
        isCheckingAuth = true

        // Версионная проверка
        getServerInfoUseCase().fold(
            onSuccess = { info ->
                val minVersion = info.min_client_version
                if (minVersion != null && com.rimskiy.shared.utils.VersionUtils.compare(appVersion, minVersion) < 0) {
                    minRequiredVersion = minVersion
                    showUpdateDialog = true
                }
                downloadUrl = info.app_download_url
            },
            onFailure = { e ->
                println("[AppNavigation] Failed to fetch server info: ${e.message}")
            }
        )
        
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
            onDismissRequest = { /* блокируем закрытие */ },
            title = { Text("Доступна новая версия") },
            text = {
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    Text("Для работы требуется версия не ниже $minRequiredVersion.")
                    downloadUrl?.let {
                        Text(
                            text = it,
                            color = MaterialTheme.colorScheme.primary,
                            style = MaterialTheme.typography.bodyMedium
                        )
                    }
                }
            },
            confirmButton = {
                TextButton(onClick = { showUpdateDialog = false }) {
                    Text("Понятно")
                }
            },
            dismissButton = {
                downloadUrl?.let { url ->
                    TextButton(onClick = { uriHandler.openUri(url) }) {
                        Text("Скачать")
                    }
                }
            }
        )
    }
    
    when (currentScreen) {
        is Screen.Auth -> {
            AuthScreen(
                onAuthSuccess = { 
                    currentScreen = Screen.Profile
                    selectedBottomNavItem = BottomNavItem.Profile
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
                                        currentScreen = Screen.Profile
                                        selectedBottomNavItem = BottomNavItem.Profile
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
                                        currentScreen = Screen.Profile
                                        selectedBottomNavItem = BottomNavItem.Profile
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
                                        currentScreen = Screen.Profile
                                        selectedBottomNavItem = BottomNavItem.Profile
                                    },
                                    getNotificationsUseCase = getNotificationsUseCase,
                                    markNotificationReadUseCase = markNotificationReadUseCase,
                                    markAllNotificationsReadUseCase = markAllNotificationsReadUseCase,
                                    screenRefreshKey = screenRefreshKey
                                )
                            }
                        }
                        else -> {}
                    }
                }
            }
        }
    }
}

