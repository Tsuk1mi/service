package com.rimskiy.shared.ui.screens

import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.foundation.clickable
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import androidx.compose.ui.ExperimentalComposeUiApi
import com.rimskiy.shared.data.model.Block
import com.rimskiy.shared.domain.usecase.CreateBlockUseCase
import com.rimskiy.shared.domain.usecase.DeleteBlockUseCase
import com.rimskiy.shared.domain.usecase.GetMyBlocksUseCase
import com.rimskiy.shared.domain.usecase.GetProfileUseCase
import com.rimskiy.shared.domain.usecase.GetUserByPlateUseCase
import com.rimskiy.shared.domain.usecase.RecognizePlateUseCase
import com.rimskiy.shared.domain.usecase.WarnOwnerUseCase
import com.rimskiy.shared.utils.DateUtils
import com.rimskiy.shared.utils.PlateUtils
import com.rimskiy.shared.ui.components.TimePickerDialog
import com.rimskiy.shared.data.local.SettingsManager
import com.rimskiy.shared.platform.createSettings
import kotlinx.coroutines.launch

@OptIn(ExperimentalComposeUiApi::class, ExperimentalMaterial3Api::class)
@Composable
fun MyBlocksScreen(
    onNavigateBack: () -> Unit,
    getMyBlocksUseCase: GetMyBlocksUseCase,
    createBlockUseCase: CreateBlockUseCase,
    deleteBlockUseCase: DeleteBlockUseCase,
    warnOwnerUseCase: WarnOwnerUseCase,
    recognizePlateUseCase: RecognizePlateUseCase,
    getProfileUseCase: GetProfileUseCase,
    getUserByPlateUseCase: GetUserByPlateUseCase,
    platformActions: com.rimskiy.shared.platform.PlatformActions? = null
) {
    var isLoading by remember { mutableStateOf(false) }
    var isRefreshing by remember { mutableStateOf(false) }
    var blocks by remember { mutableStateOf<List<Block>>(emptyList()) }
    var newPlate by remember { mutableStateOf("") }
    var error by remember { mutableStateOf<String?>(null) }
    var showDeleteDialog by remember { mutableStateOf<Block?>(null) }
    var plateError by remember { mutableStateOf<String?>(null) }
    var isRecognizing by remember { mutableStateOf(false) }
    var departureTime by remember { mutableStateOf("") }
    var departureTimeError by remember { mutableStateOf<String?>(null) }
    var departureTimeFromProfile by remember { mutableStateOf<String?>(null) }
    var showDepartureTimeDialog by remember { mutableStateOf<Pair<String, String>?>(null) } // (plate, departure_time) для диалога
    var showTimePicker by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()
    val focusManager = LocalFocusManager.current
    val keyboardController = LocalSoftwareKeyboardController.current
    val settingsManager = remember { SettingsManager(createSettings()) }
    
    // Загружаем профиль для получения времени выезда
    LaunchedEffect(Unit) {
        getProfileUseCase().fold(
            onSuccess = { profile ->
                departureTimeFromProfile = profile.departure_time
            },
            onFailure = { }
        )
    }
    
    suspend fun loadBlocks() {
        isRefreshing = true
        getMyBlocksUseCase().fold(
            onSuccess = { result ->
                blocks = result
                isRefreshing = false
                error = null
            },
            onFailure = { e ->
                error = e.message ?: "Ошибка загрузки"
                isRefreshing = false
            }
        )
    }

    LaunchedEffect(Unit) {
        isLoading = true
        loadBlocks()
        isLoading = false
    }
    
    suspend fun deleteBlock(block: Block) {
        isLoading = true
        error = null
        deleteBlockUseCase(block.id).fold(
            onSuccess = {
                loadBlocks()
                isLoading = false
            },
            onFailure = { e ->
                error = e.message ?: "Ошибка удаления"
                isLoading = false
            }
        )
    }

    fun hideKeyboard() {
        focusManager.clearFocus()
        keyboardController?.hide()
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Мои блокировки") },
                actions = {
                    IconButton(
                        onClick = {
                            scope.launch { loadBlocks() }
                        },
                        enabled = !isLoading
                    ) {
                        Icon(Icons.Default.Refresh, contentDescription = "Обновить")
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = MaterialTheme.colorScheme.surface,
                    titleContentColor = MaterialTheme.colorScheme.onSurface
                )
            )
        }
    ) { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .padding(16.dp)
                .pointerInput(Unit) {
                    detectTapGestures(onTap = { hideKeyboard() })
                },
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            // Сообщение об ошибке
            error?.let { errorText ->
                Card(
                    colors = CardDefaults.cardColors(
                        containerColor = MaterialTheme.colorScheme.errorContainer
                    ),
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Row(
                        modifier = Modifier.padding(12.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Icon(
                            imageVector = Icons.Default.Warning,
                            contentDescription = null,
                            tint = MaterialTheme.colorScheme.onErrorContainer,
                            modifier = Modifier.size(20.dp)
                        )
                        Spacer(modifier = Modifier.width(8.dp))
                        Text(
                            text = errorText,
                            color = MaterialTheme.colorScheme.onErrorContainer,
                            style = MaterialTheme.typography.bodySmall
                        )
                    }
                }
            }

            // Карточка для добавления новой блокировки
            Card(
                modifier = Modifier.fillMaxWidth(),
                elevation = CardDefaults.cardElevation(defaultElevation = 1.dp),
                shape = MaterialTheme.shapes.medium
            ) {
                Column(
                    modifier = Modifier.padding(16.dp),
                    verticalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        modifier = Modifier.fillMaxWidth()
                    ) {
                            Icon(
                                imageVector = Icons.Default.Add,
                                contentDescription = null,
                                tint = MaterialTheme.colorScheme.primary
                            )
                        Spacer(modifier = Modifier.width(8.dp))
                        Text(
                            text = "Добавить блокировку",
                            style = MaterialTheme.typography.titleMedium
                        )
                    }
                    
                    Divider()
                    
                    OutlinedTextField(
                        value = newPlate,
                        onValueChange = { 
                            newPlate = it.replace("+", "")
                            plateError = null
                            error = null
                        },
                        label = { Text("Номер автомобиля") },
                        placeholder = { Text("А123БВ777") },
                        colors = OutlinedTextFieldDefaults.colors(
                            focusedBorderColor = MaterialTheme.colorScheme.primary.copy(alpha = 0.5f),
                            unfocusedBorderColor = MaterialTheme.colorScheme.outline.copy(alpha = 0.3f)
                        ),
                        leadingIcon = {
                            Icon(Icons.Default.Home, contentDescription = null)
                        },
                        modifier = Modifier.fillMaxWidth(),
                        isError = plateError != null,
                        supportingText = plateError?.let { { Text(it) } },
                        singleLine = true,
                        keyboardOptions = KeyboardOptions(
                            imeAction = ImeAction.Done
                        ),
                        keyboardActions = KeyboardActions(onDone = { hideKeyboard() }),
                        trailingIcon = {
                            if (isRecognizing) {
                                CircularProgressIndicator(modifier = Modifier.size(20.dp))
                            } else {
                                IconButton(
                                    onClick = {
                                        hideKeyboard()
                                        isRecognizing = true
                                        plateError = null
                                        error = null
                                        platformActions?.takePhoto { imageBytes ->
                                            if (imageBytes != null) {
                                                scope.launch {
                                                    try {
                                                        recognizePlateUseCase(imageBytes).fold(
                                                            onSuccess = { response ->
                                                                if (response.success && response.plate != null) {
                                                                    newPlate = response.plate
                                                                    plateError = null
                                                                } else {
                                                                    plateError = response.error ?: "Не удалось распознать номер"
                                                                }
                                                                isRecognizing = false
                                                            },
                                                            onFailure = { e ->
                                                                plateError = e.message ?: "Ошибка распознавания"
                                                                isRecognizing = false
                                                            }
                                                        )
                                                    } catch (e: Exception) {
                                                        plateError = "Ошибка: ${e.message}"
                                                        isRecognizing = false
                                                    }
                                                }
                                            } else {
                                                plateError = "Не удалось получить фото"
                                                isRecognizing = false
                                            }
                                        }
                                    },
                                    enabled = !isLoading && !isRecognizing
                                ) {
                                    Icon(
                                        imageVector = Icons.Default.Add,
                                        contentDescription = "Считать номер с фото",
                                        tint = MaterialTheme.colorScheme.primary
                                    )
                                }
                            }
                        }
                    )
                    
                    // Время разблокировки - улучшенный UI
                    Card(
                        modifier = Modifier
                            .fillMaxWidth()
                            .clickable {
                                hideKeyboard()
                                showTimePicker = true
                            },
                        colors = CardDefaults.cardColors(
                            containerColor = if (departureTime.isNotBlank())
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
                                tint = if (departureTime.isNotBlank())
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
                                    text = if (departureTime.isNotBlank()) {
                                        "Время разблокировки"
                                    } else {
                                        "Время разблокировки"
                                    },
                                    style = MaterialTheme.typography.labelMedium,
                                    color = if (departureTime.isNotBlank())
                                        MaterialTheme.colorScheme.onPrimaryContainer
                                    else
                                        MaterialTheme.colorScheme.onSurfaceVariant
                                )
                                if (departureTime.isNotBlank()) {
                                    Text(
                                        text = departureTime,
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
                                tint = if (departureTime.isNotBlank())
                                    MaterialTheme.colorScheme.primary
                                else
                                    MaterialTheme.colorScheme.onSurfaceVariant,
                                modifier = Modifier.size(20.dp)
                            )
                        }
                    }
                    
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        Button(
                                onClick = {
                                    hideKeyboard()
                                    val normalizedPlate = PlateUtils.normalizePlate(newPlate)
                                    if (!PlateUtils.validatePlate(normalizedPlate)) {
                                        plateError = "Неверный формат номера"
                                        return@Button
                                    }
                                    isLoading = true
                                error = null
                                plateError = null
                                
                                // Создаем блокировку и проверяем время выезда владельца
                                scope.launch {
                                    val notificationMethod = when (settingsManager.notificationMethod) {
                                        SettingsManager.NotificationMethod.ANDROID_PUSH -> "android_push"
                                        SettingsManager.NotificationMethod.TELEGRAM -> "telegram"
                                        else -> "android_push"
                                    }
                                    createBlockUseCase(newPlate, false, departureTime.ifBlank { null }, notificationMethod).fold(
                                        onSuccess = {
                                            newPlate = ""
                                            departureTime = ""
                                            error = null
                                            isLoading = false
                                            loadBlocks()
                                            
                                            // После успешного создания блокировки проверяем время выезда владельца
                                            getUserByPlateUseCase(normalizedPlate).fold(
                                                onSuccess = { userInfo ->
                                                    // Если у владельца есть время выезда, показываем диалог
                                                    if (userInfo?.departure_time != null) {
                                                        showDepartureTimeDialog = Pair(normalizedPlate, userInfo.departure_time)
                                                    }
                                                },
                                                onFailure = { 
                                                    // Если не удалось получить информацию о времени выезда - это не критично
                                                }
                                            )
                                        },
                                        onFailure = { e ->
                                            error = e.message ?: "Ошибка добавления"
                                            isLoading = false
                                        }
                                    )
                                }
                            },
                            enabled = !isLoading && newPlate.isNotBlank(),
                            modifier = Modifier.weight(1f)
                        ) {
                            if (isLoading) {
                                CircularProgressIndicator(
                                    modifier = Modifier.size(18.dp),
                                    color = MaterialTheme.colorScheme.onPrimary
                                )
                                Spacer(modifier = Modifier.width(8.dp))
                            }
                            Text("Добавить")
                        }
                        
                        OutlinedButton(
                            onClick = {
                                hideKeyboard()
                                isRecognizing = true
                                platformActions?.takePhoto { imageBytes ->
                                    if (imageBytes != null) {
                                        scope.launch {
                                            isRecognizing = true
                                            error = null
                                            recognizePlateUseCase(imageBytes).fold(
                                                onSuccess = { response ->
                                                    if (response.success && response.plate != null) {
                                                        newPlate = response.plate
                                                        plateError = null
                                                    } else {
                                                        error = response.error ?: "Не удалось распознать номер"
                                                    }
                                                    isRecognizing = false
                                                },
                                                onFailure = { e ->
                                                    error = e.message ?: "Ошибка распознавания"
                                                    isRecognizing = false
                                                }
                                            )
                                        }
                                    } else {
                                        error = "Не удалось получить фото"
                                        isRecognizing = false
                                    }
                                }
                            },
                            enabled = !isLoading && !isRecognizing,
                            modifier = Modifier.weight(1f)
                        ) {
                            if (isRecognizing) {
                                CircularProgressIndicator(
                                    modifier = Modifier.size(18.dp),
                                    strokeWidth = 2.dp
                                )
                            } else {
                                Icon(
                                    imageVector = Icons.Default.Add,
                                    contentDescription = null,
                                    modifier = Modifier.size(18.dp)
                                )
                            }
                            Spacer(modifier = Modifier.width(4.dp))
                            Text(if (isRecognizing) "Распознавание..." else "Считать")
                        }
                    }
                    
                    // Карточка с временем выезда (показывается только при вводе номера)
                    if (newPlate.isNotBlank() && departureTimeFromProfile != null) {
                        Card(
                            modifier = Modifier.fillMaxWidth(),
                            colors = CardDefaults.cardColors(
                                containerColor = MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.3f)
                            ),
                            elevation = CardDefaults.cardElevation(defaultElevation = 0.dp),
                            shape = MaterialTheme.shapes.small
                        ) {
                            Row(
                                modifier = Modifier.padding(12.dp),
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(8.dp)
                            ) {
                                Icon(
                                    imageVector = Icons.Default.Info,
                                    contentDescription = null,
                                    tint = MaterialTheme.colorScheme.onPrimaryContainer,
                                    modifier = Modifier.size(20.dp)
                                )
                                Column(modifier = Modifier.weight(1f)) {
                                    Text(
                                        text = "Ваше время выезда",
                                        style = MaterialTheme.typography.labelMedium,
                                        color = MaterialTheme.colorScheme.onPrimaryContainer
                                    )
                                    Text(
                                        text = departureTimeFromProfile ?: "",
                                        style = MaterialTheme.typography.bodyMedium,
                                        color = MaterialTheme.colorScheme.onPrimaryContainer
                                    )
                                }
                            }
                        }
                    }
                }
            }

            if (isLoading && blocks.isEmpty()) {
                Box(
                    modifier = Modifier.fillMaxSize(),
                    contentAlignment = Alignment.Center
                ) {
                    CircularProgressIndicator()
                }
            } else if (blocks.isEmpty() && !isLoading) {
                Card(
                    modifier = Modifier.fillMaxSize(),
                    colors = CardDefaults.cardColors(
                        containerColor = MaterialTheme.colorScheme.surfaceVariant
                    )
                ) {
                    Box(
                        modifier = Modifier.fillMaxSize(),
                        contentAlignment = Alignment.Center
                    ) {
                        Column(
                            horizontalAlignment = Alignment.CenterHorizontally,
                            verticalArrangement = Arrangement.spacedBy(12.dp),
                            modifier = Modifier.padding(32.dp)
                        ) {
                            Icon(
                                imageVector = Icons.Default.Info,
                                contentDescription = null,
                                modifier = Modifier.size(64.dp),
                                tint = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                            Text(
                                text = "Нет блокировок",
                                style = MaterialTheme.typography.titleMedium,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                            Text(
                                text = "Добавьте номер автомобиля выше",
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    }
                }
            } else {
                if (isRefreshing) {
                    LinearProgressIndicator(modifier = Modifier.fillMaxWidth())
                }
                
                LazyColumn(
                    verticalArrangement = Arrangement.spacedBy(12.dp),
                    modifier = Modifier.fillMaxSize()
                ) {
                    items(blocks) { block ->
                        Card(
                            modifier = Modifier.fillMaxWidth(),
                            elevation = CardDefaults.cardElevation(defaultElevation = 2.dp),
                            shape = MaterialTheme.shapes.medium,
                            colors = CardDefaults.cardColors(
                                containerColor = MaterialTheme.colorScheme.surface
                            )
                        ) {
                            Column(
                                modifier = Modifier.padding(16.dp),
                                verticalArrangement = Arrangement.spacedBy(12.dp)
                            ) {
                                Row(
                                    modifier = Modifier.fillMaxWidth(),
                                    horizontalArrangement = Arrangement.SpaceBetween,
                                    verticalAlignment = Alignment.CenterVertically
                                ) {
                                    Row(
                                        modifier = Modifier.weight(1f),
                                        verticalAlignment = Alignment.CenterVertically
                                    ) {
                                        Card(
                                            colors = CardDefaults.cardColors(
                                                containerColor = MaterialTheme.colorScheme.primaryContainer
                                            ),
                                            modifier = Modifier.size(48.dp)
                                        ) {
                                            Box(
                                                modifier = Modifier.fillMaxSize(),
                                                contentAlignment = Alignment.Center
                                            ) {
                                                Text(
                                                    text = block.blocked_plate.replace("+", "").take(1),
                                                    style = MaterialTheme.typography.titleLarge,
                                                    color = MaterialTheme.colorScheme.onPrimaryContainer
                                                )
                                            }
                                        }
                                        Spacer(modifier = Modifier.width(12.dp))
                                        Column(modifier = Modifier.weight(1f)) {
                                            Text(
                                                text = block.blocked_plate.replace("+", ""),
                                                style = MaterialTheme.typography.titleLarge,
                                                color = MaterialTheme.colorScheme.onSurface
                                            )
                                            Row(
                                                verticalAlignment = Alignment.CenterVertically,
                                                horizontalArrangement = Arrangement.spacedBy(4.dp)
                                            ) {
                                                Icon(
                                                    imageVector = Icons.Default.Schedule,
                                                    contentDescription = null,
                                                    modifier = Modifier.size(14.dp),
                                                    tint = MaterialTheme.colorScheme.onSurfaceVariant
                                                )
                                                Text(
                                                    text = DateUtils.formatDateShort(block.created_at),
                                                    style = MaterialTheme.typography.bodySmall,
                                                    color = MaterialTheme.colorScheme.onSurfaceVariant
                                                )
                                            }
                                        }
                                    }
                                    IconButton(
                                        onClick = { showDeleteDialog = block },
                                        modifier = Modifier.size(40.dp)
                                    ) {
                                        Icon(
                                            imageVector = Icons.Default.Delete,
                                            contentDescription = "Удалить",
                                            tint = MaterialTheme.colorScheme.error,
                                            modifier = Modifier.size(24.dp)
                                        )
                                    }
                                }
                                
                                Divider()

                                Card(
                                    modifier = Modifier.fillMaxWidth(),
                                    colors = CardDefaults.cardColors(
                                        containerColor = MaterialTheme.colorScheme.secondaryContainer.copy(alpha = 0.3f)
                                    ),
                                    elevation = CardDefaults.cardElevation(defaultElevation = 0.dp)
                                ) {
                                    Row(
                                        modifier = Modifier.padding(12.dp),
                                        verticalAlignment = Alignment.CenterVertically,
                                        horizontalArrangement = Arrangement.spacedBy(8.dp)
                                    ) {
                                        Icon(
                                            imageVector = Icons.Default.Info,
                                            contentDescription = null,
                                            tint = MaterialTheme.colorScheme.onSecondaryContainer,
                                            modifier = Modifier.size(18.dp)
                                        )
                                        Text(
                                            text = "Если владелец указал время выезда, уведомление отправится автоматически.",
                                            style = MaterialTheme.typography.bodySmall,
                                            color = MaterialTheme.colorScheme.onSecondaryContainer
                                        )
                                    }
                                }
                                
                                Button(
                                    onClick = {
                                        scope.launch {
                                            isLoading = true
                                            warnOwnerUseCase(block.id).fold(
                                                onSuccess = {
                                                    isLoading = false
                                                },
                                                onFailure = { e ->
                                                    error = e.message ?: "Ошибка вызова владельца"
                                                    isLoading = false
                                                }
                                            )
                                        }
                                    },
                                    enabled = !isLoading,
                                    modifier = Modifier.fillMaxWidth(),
                                    colors = ButtonDefaults.buttonColors(
                                        containerColor = MaterialTheme.colorScheme.primary
                                    )
                                ) {
                                    Icon(
                                        imageVector = Icons.Default.Phone,
                                        contentDescription = null,
                                        modifier = Modifier.size(20.dp)
                                    )
                                    Spacer(modifier = Modifier.width(8.dp))
                                    Text("Предупредить владельца")
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Диалог подтверждения удаления
    showDeleteDialog?.let { block ->
        AlertDialog(
            onDismissRequest = { showDeleteDialog = null },
            icon = {
                Icon(
                    imageVector = Icons.Default.Delete,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.error
                )
            },
            title = { Text("Удалить блокировку?") },
            text = { Text("Вы уверены, что хотите удалить блокировку для номера ${block.blocked_plate.replace("+", "")}?") },
            confirmButton = {
                TextButton(
                    onClick = {
                        scope.launch {
                            deleteBlock(block)
                            showDeleteDialog = null
                        }
                    }
                ) {
                    Text("Удалить", color = MaterialTheme.colorScheme.error)
                }
            },
            dismissButton = {
                TextButton(onClick = { showDeleteDialog = null }) {
                    Text("Отмена")
                }
            }
        )
    }

    // Диалог с временем выезда владельца
    showDepartureTimeDialog?.let { (plate, departureTime) ->
        AlertDialog(
            onDismissRequest = { showDepartureTimeDialog = null },
            icon = {
                Icon(
                    imageVector = Icons.Default.Info,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.primary
                )
            },
            title = { Text("Время выезда владельца") },
            text = {
                Column(
                    verticalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    Text(
                        "Автомобиль ${plate.replace("+", "")} заблокирован.",
                        style = MaterialTheme.typography.bodyMedium
                    )
                    Text(
                        "Владелец указал время выезда:",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Card(
                        modifier = Modifier.fillMaxWidth(),
                        colors = CardDefaults.cardColors(
                            containerColor = MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.3f)
                        ),
                        elevation = CardDefaults.cardElevation(defaultElevation = 0.dp),
                        shape = MaterialTheme.shapes.small
                    ) {
                        Row(
                            modifier = Modifier.padding(16.dp),
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(12.dp)
                        ) {
                            Icon(
                                imageVector = Icons.Default.Info,
                                contentDescription = null,
                                tint = MaterialTheme.colorScheme.onPrimaryContainer,
                                modifier = Modifier.size(24.dp)
                            )
                            Text(
                                text = departureTime,
                                style = MaterialTheme.typography.headlineMedium,
                                color = MaterialTheme.colorScheme.onPrimaryContainer
                            )
                        }
                    }
                }
            },
            confirmButton = {
                Button(onClick = { showDepartureTimeDialog = null }) {
                    Text("Понятно")
                }
            }
        )
    }
    
    // Диалог выбора времени
    if (showTimePicker) {
        TimePickerDialog(
            initialTime = departureTime.ifBlank { null },
            onTimeSelected = { time ->
                departureTime = time ?: ""
                departureTimeError = null
            },
            onDismiss = { showTimePicker = false },
            title = "Время разблокировки"
        )
    }
}
