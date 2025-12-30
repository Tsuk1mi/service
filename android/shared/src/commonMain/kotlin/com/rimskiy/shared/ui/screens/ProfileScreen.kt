package com.rimskiy.shared.ui.screens

import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.unit.dp
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import com.rimskiy.shared.data.model.UserResponse
import com.rimskiy.shared.data.model.UpdateUserRequest
import com.rimskiy.shared.domain.usecase.GetProfileUseCase
import com.rimskiy.shared.domain.usecase.UpdateProfileUseCase
import com.rimskiy.shared.domain.usecase.*
import com.rimskiy.shared.utils.PhoneUtils
import com.rimskiy.shared.utils.PlateUtils
import com.rimskiy.shared.ui.components.TimePickerDialog
import com.rimskiy.shared.data.local.SettingsManager
import com.rimskiy.shared.platform.createSettings
import kotlinx.coroutines.launch
import kotlinx.serialization.json.JsonPrimitive

@OptIn(ExperimentalComposeUiApi::class, ExperimentalMaterial3Api::class)
@Composable
fun ProfileScreen(
    onNavigateToMyBlocks: () -> Unit,
    onNavigateToBlockedBy: () -> Unit,
    onNavigateToBlockNotification: () -> Unit,
    onLogout: () -> Unit,
    getProfileUseCase: GetProfileUseCase,
    updateProfileUseCase: UpdateProfileUseCase,
    getUserPlatesUseCase: GetUserPlatesUseCase,
    createUserPlateUseCase: CreateUserPlateUseCase,
    deleteUserPlateUseCase: DeleteUserPlateUseCase,
    setPrimaryPlateUseCase: SetPrimaryPlateUseCase,
    updateUserPlateDepartureUseCase: UpdateUserPlateDepartureUseCase,
    recognizePlateUseCase: RecognizePlateUseCase? = null,
    platformActions: com.rimskiy.shared.platform.PlatformActions? = null,
    screenRefreshKey: Int = 0
) {
    var isLoading by remember { mutableStateOf(false) }
    var user by remember { mutableStateOf<UserResponse?>(null) }
    var name by remember { mutableStateOf("") }
    var phone by remember { mutableStateOf("") }
    var phoneTextFieldValue by remember { mutableStateOf(TextFieldValue("")) }
    var telegram by remember { mutableStateOf("") }
    var plate by remember { mutableStateOf("") }
    var plateError by remember { mutableStateOf<String?>(null) }
    var phoneError by remember { mutableStateOf<String?>(null) }
    var error by remember { mutableStateOf<String?>(null) }
    var message by remember { mutableStateOf<String?>(null) }
    val scope = rememberCoroutineScope()
    val focusManager = LocalFocusManager.current
    val keyboardController = LocalSoftwareKeyboardController.current
    val scrollState = rememberScrollState()
    val settingsManager = remember { SettingsManager(createSettings()) }
    var notificationMethod by remember { mutableStateOf(settingsManager.notificationMethod) }
    
    fun hideKeyboard() {
        focusManager.clearFocus()
        keyboardController?.hide()
    }

    // Функция для загрузки профиля
    suspend fun loadProfile() {
        isLoading = true
        error = null
        getProfileUseCase().fold(
            onSuccess = { profile ->
                user = profile
                name = profile.name ?: ""
                phone = profile.phone ?: ""
                phoneTextFieldValue = TextFieldValue(phone)
                telegram = profile.telegram ?: ""
                plate = profile.plate
                isLoading = false
            },
            onFailure = { e ->
                error = e.message ?: "Ошибка загрузки профиля"
                isLoading = false
            }
        )
    }

    LaunchedEffect(screenRefreshKey) {
        loadProfile()
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .pointerInput(Unit) {
                detectTapGestures(onTap = { hideKeyboard() })
            }
    ) {
        // Заголовок с выходом
        TopAppBar(
            title = { Text("Профиль") },
            actions = {
                TextButton(onClick = onLogout) {
                    Text("Выйти")
                }
            },
            colors = TopAppBarDefaults.topAppBarColors(
                containerColor = MaterialTheme.colorScheme.surface,
                titleContentColor = MaterialTheme.colorScheme.onSurface
            )
        )

        if (isLoading && user == null) {
            Box(
                modifier = Modifier.fillMaxSize(),
                contentAlignment = Alignment.Center
            ) {
                CircularProgressIndicator()
            }
        } else {
            Column(
                modifier = Modifier
                    .fillMaxSize()
                    .verticalScroll(scrollState)
                    .padding(16.dp),
                verticalArrangement = Arrangement.spacedBy(16.dp)
            ) {
                // Сообщения об ошибках и успехе
                error?.let { errorText ->
                    Card(
                        colors = CardDefaults.cardColors(
                            containerColor = MaterialTheme.colorScheme.errorContainer
                        ),
                        modifier = Modifier.fillMaxWidth()
                    ) {
                        Row(
                            modifier = Modifier.padding(16.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Icon(
                                imageVector = Icons.Default.Warning,
                                contentDescription = null,
                                tint = MaterialTheme.colorScheme.onErrorContainer
                            )
                            Spacer(modifier = Modifier.width(12.dp))
                            Text(
                                text = errorText,
                                color = MaterialTheme.colorScheme.onErrorContainer,
                                style = MaterialTheme.typography.bodyMedium
                            )
                        }
                    }
                }

                message?.let { messageText ->
                    Card(
                        colors = CardDefaults.cardColors(
                            containerColor = MaterialTheme.colorScheme.primaryContainer
                        ),
                        modifier = Modifier.fillMaxWidth()
                    ) {
                        Row(
                            modifier = Modifier.padding(16.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Icon(
                                imageVector = Icons.Default.CheckCircle,
                                contentDescription = null,
                                tint = MaterialTheme.colorScheme.onPrimaryContainer
                            )
                            Spacer(modifier = Modifier.width(12.dp))
                            Text(
                                text = messageText,
                                color = MaterialTheme.colorScheme.onPrimaryContainer,
                                style = MaterialTheme.typography.bodyMedium
                            )
                        }
                    }
                }

                // Карточка с контактной информацией
                ElevatedCard(
                    modifier = Modifier.fillMaxWidth(),
                    elevation = CardDefaults.elevatedCardElevation(defaultElevation = 2.dp),
                    colors = CardDefaults.cardColors(
                        containerColor = MaterialTheme.colorScheme.surface
                    )
                ) {
                    Column(
                        modifier = Modifier.padding(20.dp),
                        verticalArrangement = Arrangement.spacedBy(16.dp)
                    ) {
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            modifier = Modifier.fillMaxWidth()
                        ) {
                            Icon(
                                imageVector = Icons.Default.Person,
                                contentDescription = null,
                                tint = MaterialTheme.colorScheme.primary,
                                modifier = Modifier.size(24.dp)
                            )
                            Spacer(modifier = Modifier.width(12.dp))
                            Text(
                                text = "Контактная информация",
                                style = MaterialTheme.typography.titleLarge,
                                color = MaterialTheme.colorScheme.onSurface
                            )
                        }
                        
                        Divider(
                            color = MaterialTheme.colorScheme.outline.copy(alpha = 0.2f),
                            thickness = 1.dp
                        )
                        
                        OutlinedTextField(
                            value = name,
                            onValueChange = { name = it },
                            label = { Text("Имя") },
                            leadingIcon = {
                                Icon(Icons.Default.Person, contentDescription = null)
                            },
                            modifier = Modifier.fillMaxWidth(),
                            singleLine = true
                        )

                        OutlinedTextField(
                            value = phoneTextFieldValue,
                            onValueChange = { newValue ->
                                val newText = newValue.text
                                val cursorPosition = newValue.selection.start
                                
                                val processedText = when {
                                    newText.isEmpty() -> ""
                                    newText.startsWith("+7") -> newText
                                    newText.startsWith("8") && newText.length > 1 -> "+7${newText.substring(1)}"
                                    newText.startsWith("9") && !newText.startsWith("+") -> "+7$newText"
                                    newText.startsWith("7") && newText.length > 1 && !newText.startsWith("+") -> "+$newText"
                                    !newText.startsWith("+") && newText.isNotEmpty() && newText.first().isDigit() -> "+7$newText"
                                    else -> newText
                                }
                                
                                val offsetChange = processedText.length - newText.length
                                val newCursorPosition = (cursorPosition + offsetChange).coerceIn(0, processedText.length)
                                
                                phone = processedText
                                phoneTextFieldValue = TextFieldValue(
                                    text = processedText,
                                    selection = TextRange(newCursorPosition)
                                )
                                phoneError = null
                            },
                            label = { Text("Телефон") },
                            placeholder = { Text("+7 (900) 123-45-67") },
                            leadingIcon = {
                                Icon(Icons.Default.Phone, contentDescription = null)
                            },
                            modifier = Modifier.fillMaxWidth(),
                            isError = phoneError != null,
                            supportingText = phoneError?.let { { Text(it) } },
                            singleLine = true,
                            keyboardOptions = KeyboardOptions(
                                keyboardType = KeyboardType.Phone,
                                imeAction = ImeAction.Next
                            ),
                            keyboardActions = KeyboardActions(
                                onNext = { hideKeyboard() }
                            )
                        )

                        OutlinedTextField(
                            value = telegram,
                            onValueChange = { telegram = it },
                            label = { Text("Telegram") },
                            placeholder = { Text("@username") },
                            leadingIcon = {
                                Icon(Icons.Default.Send, contentDescription = null)
                            },
                            modifier = Modifier.fillMaxWidth(),
                            singleLine = true,
                            keyboardOptions = KeyboardOptions(
                                keyboardType = KeyboardType.Text,
                                imeAction = ImeAction.Done
                            ),
                            keyboardActions = KeyboardActions(
                                onDone = { hideKeyboard() }
                            )
                        )
                    }
                }

                // Карточка с автомобилями
                ElevatedCard(
                    modifier = Modifier.fillMaxWidth(),
                    elevation = CardDefaults.elevatedCardElevation(defaultElevation = 2.dp),
                    colors = CardDefaults.cardColors(
                        containerColor = MaterialTheme.colorScheme.surface
                    )
                ) {
                    Column(
                        modifier = Modifier.padding(20.dp),
                        verticalArrangement = Arrangement.spacedBy(16.dp)
                    ) {
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            modifier = Modifier.fillMaxWidth()
                        ) {
                            Icon(
                                imageVector = Icons.Default.Home,
                                contentDescription = null,
                                tint = MaterialTheme.colorScheme.primary,
                                modifier = Modifier.size(24.dp)
                            )
                            Spacer(modifier = Modifier.width(12.dp))
                            Text(
                                text = "Мои автомобили",
                                style = MaterialTheme.typography.titleLarge,
                                color = MaterialTheme.colorScheme.onSurface
                            )
                        }
                        
                        Divider(
                            color = MaterialTheme.colorScheme.outline.copy(alpha = 0.2f),
                            thickness = 1.dp
                        )
                        
                        UserPlatesSection(
                            getUserPlatesUseCase = getUserPlatesUseCase,
                            createUserPlateUseCase = createUserPlateUseCase,
                            deleteUserPlateUseCase = deleteUserPlateUseCase,
                            setPrimaryPlateUseCase = setPrimaryPlateUseCase,
                            updateUserPlateDepartureUseCase = updateUserPlateDepartureUseCase,
                            recognizePlateUseCase = recognizePlateUseCase,
                            platformActions = platformActions,
                            screenRefreshKey = screenRefreshKey,
                            onPlateChanged = { newPlate ->
                                plate = newPlate
                            }
                        )
                    }
                }

                // Карточка с настройками уведомлений
                ElevatedCard(
                    modifier = Modifier.fillMaxWidth(),
                    elevation = CardDefaults.elevatedCardElevation(defaultElevation = 2.dp),
                    colors = CardDefaults.cardColors(
                        containerColor = MaterialTheme.colorScheme.surface
                    )
                ) {
                    Column(
                        modifier = Modifier.padding(20.dp),
                        verticalArrangement = Arrangement.spacedBy(16.dp)
                    ) {
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            modifier = Modifier.fillMaxWidth()
                        ) {
                            Icon(
                                imageVector = Icons.Default.Notifications,
                                contentDescription = null,
                                tint = MaterialTheme.colorScheme.primary,
                                modifier = Modifier.size(24.dp)
                            )
                            Spacer(modifier = Modifier.width(12.dp))
                            Text(
                                text = "Уведомления о блокировках",
                                style = MaterialTheme.typography.titleLarge,
                                color = MaterialTheme.colorScheme.onSurface
                            )
                        }
                        
                        Divider(
                            color = MaterialTheme.colorScheme.outline.copy(alpha = 0.2f),
                            thickness = 1.dp
                        )
                        
                        Text(
                            text = "Выберите способ получения уведомлений о блокировках:",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        
                        Column(
                            verticalArrangement = Arrangement.spacedBy(8.dp)
                        ) {
                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.SpaceBetween
                            ) {
                                Row(
                                    modifier = Modifier.weight(1f),
                                    verticalAlignment = Alignment.CenterVertically,
                                    horizontalArrangement = Arrangement.spacedBy(12.dp)
                                ) {
                                    RadioButton(
                                        selected = notificationMethod == SettingsManager.NotificationMethod.ANDROID_PUSH,
                                        onClick = {
                                            notificationMethod = SettingsManager.NotificationMethod.ANDROID_PUSH
                                            settingsManager.notificationMethod = SettingsManager.NotificationMethod.ANDROID_PUSH
                                        }
                                    )
                                    Column(modifier = Modifier.weight(1f)) {
                                        Text(
                                            text = "Android Push",
                                            style = MaterialTheme.typography.bodyLarge
                                        )
                                        Text(
                                            text = "Уведомления через приложение",
                                            style = MaterialTheme.typography.bodySmall,
                                            color = MaterialTheme.colorScheme.onSurfaceVariant
                                        )
                                    }
                                }
                            }
                            
                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.SpaceBetween
                            ) {
                                Row(
                                    modifier = Modifier.weight(1f),
                                    verticalAlignment = Alignment.CenterVertically,
                                    horizontalArrangement = Arrangement.spacedBy(12.dp)
                                ) {
                                    RadioButton(
                                        selected = notificationMethod == SettingsManager.NotificationMethod.TELEGRAM,
                                        onClick = {
                                            notificationMethod = SettingsManager.NotificationMethod.TELEGRAM
                                            settingsManager.notificationMethod = SettingsManager.NotificationMethod.TELEGRAM
                                        }
                                    )
                                    Column(modifier = Modifier.weight(1f)) {
                                        Text(
                                            text = "Telegram",
                                            style = MaterialTheme.typography.bodyLarge
                                        )
                                        Text(
                                            text = "Уведомления через Telegram бота",
                                            style = MaterialTheme.typography.bodySmall,
                                            color = MaterialTheme.colorScheme.onSurfaceVariant
                                        )
                                    }
                                }
                            }
                        }
                    }
                }

                // Кнопка сохранения
                Button(
                    onClick = {
                        hideKeyboard()
                        var hasError = false
                        if (phone.isNotBlank()) {
                            val normalizedPhone = PhoneUtils.normalizePhone(phone)
                            if (!PhoneUtils.validatePhone(normalizedPhone)) {
                                phoneError = "Неверный формат номера телефона"
                                hasError = true
                            }
                        }
                        if (hasError) return@Button
                        
                        isLoading = true
                        error = null
                        message = null
                        phoneError = null
                        plateError = null
                        scope.launch {
                            val currentPlate = user?.plate ?: ""
                            val plateToSend = if (currentPlate.isNotBlank()) {
                                PlateUtils.normalizePlate(currentPlate)
                            } else {
                                if (plate.isNotBlank()) {
                                    PlateUtils.normalizePlate(plate)
                                } else {
                                    plateError = "Номер автомобиля обязателен. Добавьте автомобиль в разделе 'Мои автомобили'."
                                    isLoading = false
                                    return@launch
                                }
                            }
                            
                            val normalizedPhone = if (phone.isNotBlank()) {
                                PhoneUtils.normalizePhone(phone)
                            } else null
                            
                            val normalizedTelegram = telegram.ifBlank { null }?.removePrefix("@")
                            
                            updateProfileUseCase(
                                UpdateUserRequest(
                                    name = name.ifBlank { null },
                                    phone = normalizedPhone,
                                    telegram = normalizedTelegram,
                                    plate = plateToSend,
                                    show_contacts = null,
                                    owner_type = null,
                                    owner_info = null,
                                    departure_time = null
                                )
                            ).fold(
                                onSuccess = {
                                    message = "Профиль успешно обновлен"
                                    error = null
                                    isLoading = false
                                    scope.launch {
                                        loadProfile()
                                    }
                                },
                                onFailure = { e ->
                                    error = e.message ?: "Ошибка обновления профиля"
                                    message = null
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
                    if (isLoading) {
                        CircularProgressIndicator(
                            modifier = Modifier.size(20.dp),
                            color = MaterialTheme.colorScheme.onPrimary
                        )
                        Spacer(modifier = Modifier.width(8.dp))
                    }
                    Text(
                        "Сохранить",
                        style = MaterialTheme.typography.labelLarge
                    )
                }
            }
        }
    }
}
