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
import androidx.compose.material.icons.filled.ArrowForward
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.DirectionsCar
import androidx.compose.material.icons.filled.Info
import androidx.compose.material.icons.filled.Notifications
import androidx.compose.material.icons.filled.Schedule
import androidx.compose.material.icons.filled.Search
import androidx.compose.material.icons.filled.Warning
import androidx.compose.material3.*
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ElevatedCard
import androidx.compose.material3.OutlinedButton
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
import com.rimskiy.shared.ui.components.TimePickerDialog
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
                .padding(horizontal = 16.dp, vertical = 8.dp)
                .fillMaxSize(),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            ElevatedCard(
                modifier = Modifier.fillMaxWidth(),
                elevation = CardDefaults.elevatedCardElevation(defaultElevation = 3.dp)
            ) {
                Column(
                    modifier = Modifier.padding(20.dp),
                    verticalArrangement = Arrangement.spacedBy(16.dp)
                ) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(12.dp)
                    ) {
                        Icon(
                            imageVector = Icons.Default.Warning,
                            contentDescription = null,
                            tint = MaterialTheme.colorScheme.error,
                            modifier = Modifier.size(28.dp)
                        )
                        Text(
                            text = "Детали блокировки",
                            style = MaterialTheme.typography.titleLarge,
                            color = MaterialTheme.colorScheme.onSurface
                        )
                    }
                    Divider(
                        color = MaterialTheme.colorScheme.outline.copy(alpha = 0.2f),
                        thickness = 1.dp
                    )
                    
                    InfoRow("Номер автомобиля", block.blocked_plate.replace("+", ""))
                    InfoRow("Заблокировал", block.blocker.name ?: block.blocker.telegram ?: "Неизвестно")
                    block.blocker_owner_type?.let { InfoRow("Тип владельца", it) }
                    block.blocker_owner_info?.let { 
                        InfoRow("Информация", it.toString())
                    }
                    InfoRow("Дата блокировки", DateUtils.formatDateShort(block.created_at))
                }
            }
        }
    }
}

@Composable
private fun InfoRow(label: String, value: String) {
    Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
        Text(
            text = label,
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )
        Text(
            text = value,
            style = MaterialTheme.typography.bodyLarge,
            color = MaterialTheme.colorScheme.onSurface
        )
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
                .padding(horizontal = 16.dp, vertical = 8.dp)
                .fillMaxSize(),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            ElevatedCard(
                modifier = Modifier.fillMaxWidth(),
                elevation = CardDefaults.elevatedCardElevation(defaultElevation = 2.dp)
            ) {
                Column(
                    modifier = Modifier.padding(20.dp),
                    verticalArrangement = Arrangement.spacedBy(16.dp)
                ) {
                    Text(
                        text = "Проверка блокировки",
                        style = MaterialTheme.typography.titleLarge,
                        color = MaterialTheme.colorScheme.onSurface
                    )
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
                }
            }

            error?.let { errorText ->
                ElevatedCard(
                    modifier = Modifier.fillMaxWidth(),
                    elevation = CardDefaults.elevatedCardElevation(defaultElevation = 2.dp),
                    colors = CardDefaults.cardColors(
                        containerColor = MaterialTheme.colorScheme.errorContainer
                    )
                ) {
                    Row(
                        modifier = Modifier.padding(16.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(12.dp)
                    ) {
                        Icon(
                            imageVector = Icons.Default.Info,
                            contentDescription = null,
                            tint = MaterialTheme.colorScheme.onErrorContainer,
                            modifier = Modifier.size(24.dp)
                        )
                        Text(
                            text = errorText,
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onErrorContainer
                        )
                    }
                }
            }

            result?.let { block ->
                ElevatedCard(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clickable { onNavigateToBlocker(block) },
                    elevation = CardDefaults.elevatedCardElevation(defaultElevation = 3.dp),
                    colors = CardDefaults.cardColors(
                        containerColor = MaterialTheme.colorScheme.primaryContainer
                    )
                ) {
                    Column(
                        modifier = Modifier.padding(20.dp),
                        verticalArrangement = Arrangement.spacedBy(12.dp)
                    ) {
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(12.dp)
                        ) {
                            Icon(
                                imageVector = Icons.Default.Warning,
                                contentDescription = null,
                                tint = MaterialTheme.colorScheme.onPrimaryContainer,
                                modifier = Modifier.size(24.dp)
                            )
                            Text(
                                text = "Блокировка найдена",
                                style = MaterialTheme.typography.titleMedium,
                                color = MaterialTheme.colorScheme.onPrimaryContainer
                            )
                        }
                        Divider(
                            color = MaterialTheme.colorScheme.onPrimaryContainer.copy(alpha = 0.2f),
                            thickness = 1.dp
                        )
                        InfoRow("Номер", block.blocked_plate.replace("+", ""))
                        InfoRow("Заблокировал", block.blocker.name ?: block.blocker.telegram ?: "Неизвестно")
                        InfoRow("Дата", DateUtils.formatDateShort(block.created_at))
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
                contentPadding = PaddingValues(horizontal = 16.dp, vertical = 8.dp),
                verticalArrangement = Arrangement.spacedBy(16.dp)
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
    ElevatedCard(
        modifier = Modifier.fillMaxWidth(),
        elevation = CardDefaults.elevatedCardElevation(defaultElevation = 2.dp),
        colors = CardDefaults.cardColors(
            containerColor = if (notification.read) MaterialTheme.colorScheme.surfaceVariant
            else MaterialTheme.colorScheme.primaryContainer
        )
    ) {
        Column(
            modifier = Modifier.padding(20.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                Icon(
                    imageVector = Icons.Default.Notifications,
                    contentDescription = null,
                    tint = if (notification.read)
                        MaterialTheme.colorScheme.onSurfaceVariant
                    else
                        MaterialTheme.colorScheme.onPrimaryContainer,
                    modifier = Modifier.size(24.dp)
                )
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = notification.title ?: "Уведомление",
                        style = MaterialTheme.typography.titleMedium,
                        color = if (notification.read)
                            MaterialTheme.colorScheme.onSurfaceVariant
                        else
                            MaterialTheme.colorScheme.onPrimaryContainer
                    )
                    notification.message?.let {
                        Text(
                            text = it,
                            style = MaterialTheme.typography.bodyMedium,
                            color = if (notification.read)
                                MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.8f)
                            else
                                MaterialTheme.colorScheme.onPrimaryContainer.copy(alpha = 0.9f)
                        )
                    }
                }
                if (!notification.read) {
                    OutlinedButton(onClick = onMarkRead) {
                        Text("Прочесть")
                    }
                }
            }
            notification.created_at?.let {
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(6.dp)
                ) {
                    Icon(
                        imageVector = Icons.Default.Schedule,
                        contentDescription = null,
                        modifier = Modifier.size(16.dp),
                        tint = if (notification.read)
                            MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
                        else
                            MaterialTheme.colorScheme.onPrimaryContainer.copy(alpha = 0.7f)
                    )
                    Text(
                        text = DateUtils.formatDateShort(it),
                        style = MaterialTheme.typography.bodySmall,
                        color = if (notification.read)
                            MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
                        else
                            MaterialTheme.colorScheme.onPrimaryContainer.copy(alpha = 0.7f)
                    )
                }
            }
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
            error?.let { errorText ->
                ElevatedCard(
                    modifier = Modifier.fillMaxWidth(),
                    elevation = CardDefaults.elevatedCardElevation(defaultElevation = 2.dp),
                    colors = CardDefaults.cardColors(
                        containerColor = MaterialTheme.colorScheme.errorContainer
                    )
                ) {
                    Row(
                        modifier = Modifier.padding(16.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(12.dp)
                    ) {
                        Icon(
                            imageVector = Icons.Default.Warning,
                            contentDescription = null,
                            tint = MaterialTheme.colorScheme.onErrorContainer,
                            modifier = Modifier.size(24.dp)
                        )
                        Text(
                            text = errorText,
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onErrorContainer
                        )
                    }
                }
            }

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

        if (plates.isEmpty() && !isLoading) {
            ElevatedCard(
                modifier = Modifier.fillMaxWidth(),
                elevation = CardDefaults.elevatedCardElevation(defaultElevation = 2.dp),
                colors = CardDefaults.cardColors(
                    containerColor = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5f)
                )
            ) {
                Column(
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.spacedBy(12.dp),
                    modifier = Modifier.padding(24.dp)
                ) {
                    Icon(
                        imageVector = Icons.Default.DirectionsCar,
                        contentDescription = null,
                        modifier = Modifier.size(48.dp),
                        tint = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Text(
                        text = "Нет добавленных номеров",
                        style = MaterialTheme.typography.titleMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Text(
                        text = "Добавьте номер автомобиля выше",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }
        } else {
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
    var showTimePicker by remember { mutableStateOf(false) }

    ElevatedCard(
        modifier = Modifier.fillMaxWidth(),
        elevation = CardDefaults.elevatedCardElevation(defaultElevation = 2.dp),
        colors = CardDefaults.cardColors(
            containerColor = if (plate.is_primary) MaterialTheme.colorScheme.primaryContainer
            else MaterialTheme.colorScheme.surface
        )
    ) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                Icon(
                    imageVector = Icons.Default.DirectionsCar,
                    contentDescription = null,
                    tint = if (plate.is_primary)
                        MaterialTheme.colorScheme.onPrimaryContainer
                    else
                        MaterialTheme.colorScheme.primary,
                    modifier = Modifier.size(28.dp)
                )
                Column(Modifier.weight(1f)) {
                    Text(
                        text = plate.plate,
                        style = MaterialTheme.typography.titleLarge,
                        color = if (plate.is_primary)
                            MaterialTheme.colorScheme.onPrimaryContainer
                        else
                            MaterialTheme.colorScheme.onSurface
                    )
                    Text(
                        text = if (plate.is_primary) "Основной номер" else "Дополнительный номер",
                        style = MaterialTheme.typography.bodySmall,
                        color = if (plate.is_primary)
                            MaterialTheme.colorScheme.onPrimaryContainer.copy(alpha = 0.8f)
                        else
                            MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    if (plate.departure_time != null) {
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(6.dp)
                        ) {
                            Icon(
                                imageVector = Icons.Default.Schedule,
                                contentDescription = null,
                                modifier = Modifier.size(16.dp),
                                tint = if (plate.is_primary)
                                    MaterialTheme.colorScheme.onPrimaryContainer.copy(alpha = 0.8f)
                                else
                                    MaterialTheme.colorScheme.primary
                            )
                            Text(
                                text = "Выезд: ${plate.departure_time}",
                                style = MaterialTheme.typography.bodySmall,
                                color = if (plate.is_primary)
                                    MaterialTheme.colorScheme.onPrimaryContainer.copy(alpha = 0.8f)
                                else
                                    MaterialTheme.colorScheme.primary
                            )
                        }
                    }
                }
                Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                    IconButton(onClick = onSetPrimary, enabled = !plate.is_primary) {
                        Icon(Icons.Default.Check, contentDescription = "Сделать основным")
                    }
                    IconButton(onClick = onDelete) {
                        Icon(Icons.Default.Delete, contentDescription = "Удалить")
                    }
                }
            }
            Divider(
                color = if (plate.is_primary)
                    MaterialTheme.colorScheme.onPrimaryContainer.copy(alpha = 0.2f)
                else
                    MaterialTheme.colorScheme.outline.copy(alpha = 0.2f),
                thickness = 1.dp
            )
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 12.dp, vertical = 8.dp),
                verticalArrangement = Arrangement.spacedBy(12.dp)
            ) {
            // Улучшенная карточка выбора времени
            Card(
                modifier = Modifier
                    .fillMaxWidth()
                    .clickable {
                        showTimePicker = true
                    },
                colors = CardDefaults.cardColors(
                    containerColor = if (departure.isNotBlank())
                        MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.3f)
                    else
                        MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5f)
                ),
                elevation = CardDefaults.cardElevation(defaultElevation = 0.dp),
                shape = MaterialTheme.shapes.medium
            ) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(16.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    Icon(
                        imageVector = Icons.Default.Schedule,
                        contentDescription = null,
                        tint = if (departure.isNotBlank())
                            MaterialTheme.colorScheme.primary
                        else
                            MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier = Modifier.size(24.dp)
                    )
                    Column(
                        modifier = Modifier.weight(1f),
                        horizontalAlignment = Alignment.Start
                    ) {
                        Text(
                            text = "Время выезда",
                            style = MaterialTheme.typography.labelMedium,
                            color = if (departure.isNotBlank())
                                MaterialTheme.colorScheme.onPrimaryContainer
                            else
                                MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        if (departure.isNotBlank()) {
                            Text(
                                text = departure,
                                style = MaterialTheme.typography.headlineSmall,
                                color = MaterialTheme.colorScheme.onPrimaryContainer
                            )
                        } else {
                            Text(
                                text = "Нажмите для выбора",
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    }
                    Icon(
                        imageVector = Icons.Default.ArrowForward,
                        contentDescription = null,
                        tint = if (departure.isNotBlank())
                            MaterialTheme.colorScheme.primary
                        else
                            MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier = Modifier.size(20.dp)
                    )
                }
            }
            
            // Кнопка сохранения с улучшенным дизайном
            Button(
                onClick = {
                    onUpdateDeparture(departure)
                },
                enabled = departure.isNotBlank(),
                modifier = Modifier.fillMaxWidth(),
                colors = ButtonDefaults.buttonColors(
                    containerColor = MaterialTheme.colorScheme.primary
                )
            ) {
                Icon(
                    imageVector = Icons.Default.Check,
                    contentDescription = null,
                    modifier = Modifier.size(18.dp)
                )
                Spacer(modifier = Modifier.width(8.dp))
                Text("Сохранить время")
            }
            }
        }
    }
    
    // Диалог выбора времени
    if (showTimePicker) {
        TimePickerDialog(
            initialTime = departure.ifBlank { null },
            onTimeSelected = { time ->
                departure = time ?: ""
                departureError = null
            },
            onDismiss = { showTimePicker = false },
            title = "Время выезда"
        )
    }
}

