package com.rimskiy.shared.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Info
import androidx.compose.material.icons.filled.Notifications
import androidx.compose.material.icons.filled.Search
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import com.rimskiy.shared.data.model.BlockWithBlockerInfo
import com.rimskiy.shared.data.model.NotificationResponse
import com.rimskiy.shared.data.model.UserPlateResponse
import com.rimskiy.shared.domain.usecase.*
import com.rimskiy.shared.utils.DateUtils
import kotlinx.coroutines.launch
import androidx.compose.material3.ExperimentalMaterial3Api

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun BlockNotificationDetailsScreen(
    block: BlockWithBlockerInfo,
    onNavigateBack: () -> Unit,
    platformActions: com.rimskiy.shared.platform.PlatformActions? = null
) {
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Детали блокировки") },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(Icons.Default.ArrowBack, contentDescription = "Назад")
                    }
                }
            )
        }
    ) { padding ->
        Column(
            modifier = Modifier
                .padding(padding)
                .padding(16.dp)
                .fillMaxSize(),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Text("Номер: ${block.blocked_plate}", style = MaterialTheme.typography.titleMedium)
            Text("Заблокировал: ${block.blocker.name ?: block.blocker.telegram ?: "Неизвестно"}")
            block.blocker_owner_type?.let { Text("Тип владельца: $it") }
            block.blocker_owner_info?.let { Text("Инфо: $it") }
            Text("Дата: ${DateUtils.formatDateShort(block.created_at)}")
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun CheckMyBlockScreen(
    onNavigateBack: () -> Unit,
    onNavigateToBlocker: (BlockWithBlockerInfo) -> Unit,
    checkBlockUseCase: CheckBlockUseCase
) {
    var plate by remember { mutableStateOf("") }
    var result by remember { mutableStateOf<BlockWithBlockerInfo?>(null) }
    var error by remember { mutableStateOf<String?>(null) }
    var isLoading by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Проверка блокировки") },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(Icons.Default.ArrowBack, contentDescription = "Назад")
                    }
                }
            )
        }
    ) { padding ->
        Column(
            modifier = Modifier
                .padding(padding)
                .padding(16.dp)
                .fillMaxSize(),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            OutlinedTextField(
                value = plate,
                onValueChange = { plate = it },
                label = { Text("Номер авто") },
                leadingIcon = { Icon(Icons.Default.Search, contentDescription = null) },
                keyboardOptions = KeyboardOptions(imeAction = ImeAction.Search, keyboardType = KeyboardType.Text),
                keyboardActions = KeyboardActions(
                    onSearch = {
                        scope.launch {
                            isLoading = true
                            error = null
                            result = null
                            checkBlockUseCase(plate).fold(
                                onSuccess = { resp ->
                                    result = resp.block
                                    if (!resp.is_blocked) {
                                        error = "Блокировок не найдено"
                                    }
                                },
                                onFailure = { e -> error = e.message ?: "Ошибка проверки" }
                            )
                            isLoading = false
                        }
                    }
                ),
                modifier = Modifier.fillMaxWidth(),
                singleLine = true
            )

            Button(
                onClick = {
                    scope.launch {
                        isLoading = true
                        error = null
                        result = null
                        checkBlockUseCase(plate).fold(
                            onSuccess = { resp ->
                                result = resp.block
                                if (!resp.is_blocked) error = "Блокировок не найдено"
                            },
                            onFailure = { e -> error = e.message ?: "Ошибка проверки" }
                        )
                        isLoading = false
                    }
                },
                enabled = plate.isNotBlank() && !isLoading,
                modifier = Modifier.fillMaxWidth()
            ) {
                if (isLoading) {
                    CircularProgressIndicator(modifier = Modifier.size(20.dp), color = MaterialTheme.colorScheme.onPrimary)
                    Spacer(Modifier.width(8.dp))
                }
                Text("Проверить")
            }

            error?.let {
                AssistChip(onClick = {}, label = { Text(it) }, leadingIcon = {
                    Icon(Icons.Default.Info, contentDescription = null)
                })
            }

            result?.let { block ->
                Card(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clickable { onNavigateToBlocker(block) },
                    colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.primaryContainer)
                ) {
                    Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                        Text("Номер: ${block.blocked_plate}", style = MaterialTheme.typography.titleMedium)
                        Text("Заблокировал: ${block.blocker.name ?: block.blocker.telegram ?: "Неизвестно"}")
                        Text("Дата: ${DateUtils.formatDateShort(block.created_at)}")
                    }
                }
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun BlockNotificationScreen(
    onNavigateBack: () -> Unit,
    getNotificationsUseCase: GetNotificationsUseCase,
    markNotificationReadUseCase: MarkNotificationReadUseCase,
    markAllNotificationsReadUseCase: MarkAllNotificationsReadUseCase,
    screenRefreshKey: Int
) {
    var notifications by remember { mutableStateOf<List<NotificationResponse>>(emptyList()) }
    var isLoading by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    val scope = rememberCoroutineScope()

    fun load() {
        scope.launch {
            isLoading = true
            error = null
            getNotificationsUseCase().fold(
                onSuccess = { notifications = it },
                onFailure = { e -> error = e.message ?: "Ошибка загрузки уведомлений" }
            )
            isLoading = false
        }
    }

    LaunchedEffect(screenRefreshKey) { load() }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Уведомления") },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(Icons.Default.ArrowBack, contentDescription = "Назад")
                    }
                },
                actions = {
                    TextButton(onClick = {
                        scope.launch {
                            markAllNotificationsReadUseCase()
                            load()
                        }
                    }) { Text("Прочитано") }
                }
            )
        }
    ) { padding ->
        Column(
            modifier = Modifier
                .padding(padding)
                .fillMaxSize()
        ) {
            if (isLoading) {
                LinearProgressIndicator(modifier = Modifier.fillMaxWidth())
            }
            error?.let { msg ->
                Text(msg, color = MaterialTheme.colorScheme.error, modifier = Modifier.padding(16.dp))
            }
            LazyColumn(
                modifier = Modifier.fillMaxSize(),
                contentPadding = PaddingValues(16.dp),
                verticalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                items(notifications) { n ->
                    NotificationItem(
                        notification = n,
                        onMarkRead = {
                            scope.launch {
                                markNotificationReadUseCase(n.id)
                                load()
                            }
                        }
                    )
                }
            }
        }
    }
}

@Composable
private fun NotificationItem(notification: NotificationResponse, onMarkRead: () -> Unit) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(
            containerColor = if (notification.read) MaterialTheme.colorScheme.surfaceVariant
            else MaterialTheme.colorScheme.primaryContainer
        )
    ) {
        Column(Modifier.padding(12.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Icon(Icons.Default.Notifications, contentDescription = null)
                Text(notification.title ?: "Уведомление", style = MaterialTheme.typography.titleSmall)
                Spacer(Modifier.weight(1f))
                if (!notification.read) {
                    TextButton(onClick = onMarkRead) { Text("Прочесть") }
                }
            }
            notification.message?.let { Text(it) }
            notification.created_at?.let { Text(DateUtils.formatDateShort(it), style = MaterialTheme.typography.bodySmall) }
        }
    }
}

@Composable
fun UserPlatesSection(
    getUserPlatesUseCase: GetUserPlatesUseCase,
    createUserPlateUseCase: CreateUserPlateUseCase,
    deleteUserPlateUseCase: DeleteUserPlateUseCase,
    setPrimaryPlateUseCase: SetPrimaryPlateUseCase,
    updateUserPlateDepartureUseCase: UpdateUserPlateDepartureUseCase,
    recognizePlateUseCase: RecognizePlateUseCase?,
    platformActions: com.rimskiy.shared.platform.PlatformActions? = null,
    screenRefreshKey: Int = 0,
    onPlateChanged: (String) -> Unit = {}
) {
    var plates by remember { mutableStateOf<List<UserPlateResponse>>(emptyList()) }
    var newPlate by remember { mutableStateOf("") }
    var error by remember { mutableStateOf<String?>(null) }
    var isLoading by remember { mutableStateOf(false) }
    var isRecognizing by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()

    fun load() {
        scope.launch {
            isLoading = true
            error = null
            getUserPlatesUseCase().fold(
                onSuccess = { list ->
                    plates = list
                    list.firstOrNull { it.is_primary }?.plate?.let(onPlateChanged)
                },
                onFailure = { e -> error = e.message ?: "Ошибка загрузки номеров" }
            )
            isLoading = false
        }
    }

    LaunchedEffect(screenRefreshKey) { load() }

    Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
        if (isLoading) {
            LinearProgressIndicator(modifier = Modifier.fillMaxWidth())
        }
        error?.let { Text(it, color = MaterialTheme.colorScheme.error) }

        OutlinedTextField(
            value = newPlate,
            onValueChange = { newPlate = it.uppercase() },
            label = { Text("Новый номер") },
            singleLine = true,
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done, keyboardType = KeyboardType.Text),
            keyboardActions = KeyboardActions(onDone = { /* handled by button */ })
        )
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Button(
                onClick = {
                    scope.launch {
                        if (newPlate.isBlank()) {
                            error = "Введите номер"
                            return@launch
                        }
                        isLoading = true
                        error = null
                        createUserPlateUseCase(com.rimskiy.shared.data.model.CreateUserPlateRequest(newPlate, plates.isEmpty())).fold(
                            onSuccess = {
                                newPlate = ""
                                load()
                            },
                            onFailure = { e -> error = e.message ?: "Ошибка добавления номера" }
                        )
                        isLoading = false
                    }
                },
                enabled = newPlate.isNotBlank() && !isLoading
            ) {
                Text("Добавить")
            }
            if (recognizePlateUseCase != null && platformActions != null) {
                OutlinedButton(
                    onClick = {
                        platformActions.takePhoto { bytes ->
                            if (bytes != null) {
                                scope.launch {
                                    isRecognizing = true
                                    recognizePlateUseCase(bytes).fold(
                                        onSuccess = { resp ->
                                            if (resp.success && resp.plate != null) {
                                                newPlate = resp.plate
                                            } else {
                                                error = resp.error ?: "Не удалось распознать"
                                            }
                                        },
                                        onFailure = { e -> error = e.message ?: "Ошибка распознавания" }
                                    )
                                    isRecognizing = false
                                }
                            } else {
                                error = "Фото не получено"
                            }
                        }
                    },
                    enabled = !isRecognizing && !isLoading
                ) {
                    if (isRecognizing) {
                        CircularProgressIndicator(modifier = Modifier.size(16.dp))
                        Spacer(Modifier.width(8.dp))
                    }
                    Text("Считать с фото")
                }
            }
        }

        Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
            plates.forEach { plate ->
                PlateItem(
                    plate = plate,
                    onUpdateDeparture = { newTime ->
                        scope.launch {
                            isLoading = true
                            updateUserPlateDepartureUseCase(plate.id, if (newTime.isBlank()) null else newTime)
                                .fold(
                                    onSuccess = { load() },
                                    onFailure = { e -> error = e.message ?: "Ошибка сохранения времени" }
                                )
                            isLoading = false
                        }
                    },
                    onSetPrimary = {
                        scope.launch {
                            isLoading = true
                            setPrimaryPlateUseCase(plate.id)
                            load()
                            isLoading = false
                        }
                    },
                    onDelete = {
                        scope.launch {
                            isLoading = true
                            deleteUserPlateUseCase(plate.id)
                            load()
                            isLoading = false
                        }
                    }
                )
            }
        }
    }
}

@Composable
private fun PlateItem(
    plate: UserPlateResponse,
    onUpdateDeparture: (String) -> Unit,
    onSetPrimary: () -> Unit,
    onDelete: () -> Unit
) {
    var departure by remember(plate.id) { mutableStateOf(plate.departure_time ?: "") }
    var departureError by remember { mutableStateOf<String?>(null) }

    Card(
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(
            containerColor = if (plate.is_primary) MaterialTheme.colorScheme.primaryContainer
            else MaterialTheme.colorScheme.surface
        )
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(12.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Column(Modifier.weight(1f)) {
                Text(plate.plate, style = MaterialTheme.typography.titleMedium)
                Text(if (plate.is_primary) "Основной" else "Дополнительный", style = MaterialTheme.typography.bodySmall)
                if (plate.departure_time != null) {
                    Text("Время выезда: ${plate.departure_time}", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.primary)
                }
            }
            IconButton(onClick = onSetPrimary, enabled = !plate.is_primary) {
                Icon(Icons.Default.Info, contentDescription = "Сделать основным")
            }
            IconButton(onClick = onDelete) {
                Icon(Icons.Default.Delete, contentDescription = "Удалить")
            }
        }
        Divider()
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 12.dp, vertical = 8.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            OutlinedTextField(
                value = departure,
                onValueChange = { value ->
                    val digits = value.filter { it.isDigit() }.take(4)
                    departure = when (digits.length) {
                        0 -> ""
                        1 -> "0$digits:"
                        2 -> "$digits:"
                        3 -> "${digits.take(2)}:${digits.takeLast(1)}"
                        else -> "${digits.take(2)}:${digits.takeLast(2)}"
                    }
                    departureError = null
                },
                label = { Text("Время выезда (ЧЧ:ММ)") },
                placeholder = { Text("18:30") },
                singleLine = true,
                isError = departureError != null
            )
            Button(
                onClick = {
                    if (departure.isNotEmpty() && !departure.matches(Regex("^\\d{2}:\\d{2}$"))) {
                        departureError = "Формат ЧЧ:ММ"
                    } else {
                        onUpdateDeparture(departure)
                    }
                },
                enabled = departureError == null
            ) {
                Text("Сохранить время")
            }
        }
    }
}

