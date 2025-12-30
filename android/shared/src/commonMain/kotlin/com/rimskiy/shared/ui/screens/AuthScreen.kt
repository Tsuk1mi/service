package com.rimskiy.shared.ui.screens

import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ElevatedCard
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
import com.rimskiy.shared.domain.usecase.StartAuthUseCase
import com.rimskiy.shared.domain.usecase.VerifyAuthUseCase
import com.rimskiy.shared.utils.PhoneUtils
import kotlinx.coroutines.launch

@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun AuthScreen(
    onAuthSuccess: () -> Unit,
    startAuthUseCase: StartAuthUseCase,
    verifyAuthUseCase: VerifyAuthUseCase,
    currentBaseUrl: String,
    onChangeBaseUrl: (String) -> Unit
) {
    var phone by remember { mutableStateOf("") }
    var phoneTextFieldValue by remember { mutableStateOf(TextFieldValue("")) }
    var code by remember { mutableStateOf("") }
    var codeSent by remember { mutableStateOf(false) }
    var isLoading by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    var receivedCode by remember { mutableStateOf<String?>(null) }
    var phoneError by remember { mutableStateOf<String?>(null) }
    val scope = rememberCoroutineScope()
    val focusManager = LocalFocusManager.current
    val keyboardController = LocalSoftwareKeyboardController.current

    fun hideKeyboard() {
        focusManager.clearFocus()
        keyboardController?.hide()
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .pointerInput(Unit) {
                detectTapGestures(onTap = { hideKeyboard() })
            },
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        // Логотип или иконка
        ElevatedCard(
            colors = CardDefaults.cardColors(
                containerColor = MaterialTheme.colorScheme.primaryContainer
            ),
            modifier = Modifier.size(100.dp)
        ) {
            Box(
                modifier = Modifier.fillMaxSize(),
                contentAlignment = Alignment.Center
            ) {
                Icon(
                    imageVector = Icons.Default.Lock,
                    contentDescription = null,
                    modifier = Modifier.size(56.dp),
                    tint = MaterialTheme.colorScheme.onPrimaryContainer
                )
            }
        }
        
        Spacer(modifier = Modifier.height(32.dp))
        
        Text(
            text = "Добро пожаловать!",
            style = MaterialTheme.typography.headlineLarge,
            color = MaterialTheme.colorScheme.onSurface
        )
        
        Spacer(modifier = Modifier.height(8.dp))
        
        Text(
            text = "Войдите в свой аккаунт для продолжения",
            style = MaterialTheme.typography.bodyLarge,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )

        Spacer(modifier = Modifier.height(40.dp))

        // Карточка с формой авторизации
        ElevatedCard(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 24.dp),
            elevation = CardDefaults.elevatedCardElevation(defaultElevation = 4.dp),
            colors = CardDefaults.cardColors(
                containerColor = MaterialTheme.colorScheme.surface
            )
        ) {
            Column(
                modifier = Modifier.padding(28.dp),
                verticalArrangement = Arrangement.spacedBy(20.dp)
            ) {
                if (!codeSent) {
                    // Ввод телефона
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
                            error = null
                        },
                        label = { Text("Номер телефона") },
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
                            imeAction = ImeAction.Done
                        ),
                        keyboardActions = KeyboardActions(
                            onDone = { hideKeyboard() }
                        )
                    )

                    // Ошибка
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
                                    tint = MaterialTheme.colorScheme.onErrorContainer,
                                    modifier = Modifier.size(24.dp)
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

                    Button(
                        onClick = {
                            hideKeyboard()
                            val normalizedPhone = PhoneUtils.normalizePhone(phone)
                            if (!PhoneUtils.validatePhone(normalizedPhone)) {
                                phoneError = "Неверный формат номера телефона"
                                return@Button
                            }
                            
                            isLoading = true
                            error = null
                            phoneError = null
                            scope.launch {
                                startAuthUseCase(phone).fold(
                                    onSuccess = { response ->
                                        codeSent = true
                                        receivedCode = if (response.code.isNotBlank()) response.code else null
                                        isLoading = false
                                    },
                                    onFailure = { e ->
                                        error = e.message ?: "Ошибка отправки кода"
                                        isLoading = false
                                    }
                                )
                            }
                        },
                        enabled = !isLoading && phone.isNotBlank(),
                        modifier = Modifier.fillMaxWidth()
                    ) {
                        if (isLoading) {
                            CircularProgressIndicator(
                                modifier = Modifier.size(20.dp),
                                color = MaterialTheme.colorScheme.onPrimary
                            )
                            Spacer(modifier = Modifier.width(8.dp))
                        }
                        Text("Отправить код")
                    }

                } else {
                    // Ввод кода
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Icon(
                            imageVector = Icons.Default.Send,
                            contentDescription = null,
                            tint = MaterialTheme.colorScheme.primary
                        )
                        Spacer(modifier = Modifier.width(8.dp))
                        Text(
                            text = "Введите код из SMS",
                            style = MaterialTheme.typography.titleMedium
                        )
                    }

                    // Показываем код только в dev режиме
                    receivedCode?.let { code ->
                        Card(
                            colors = CardDefaults.cardColors(
                                containerColor = MaterialTheme.colorScheme.primaryContainer
                            ),
                            modifier = Modifier.fillMaxWidth()
                        ) {
                            Column(
                                modifier = Modifier.padding(16.dp),
                                horizontalAlignment = Alignment.CenterHorizontally
                            ) {
                                Text(
                                    text = "Код подтверждения (dev):",
                                    style = MaterialTheme.typography.bodySmall,
                                    color = MaterialTheme.colorScheme.onPrimaryContainer
                                )
                                Text(
                                    text = code,
                                    style = MaterialTheme.typography.headlineMedium,
                                    color = MaterialTheme.colorScheme.onPrimaryContainer
                                )
                            }
                        }
                    }

                    OutlinedTextField(
                        value = code,
                        onValueChange = { 
                            code = it
                            error = null
                        },
                        label = { Text("Код подтверждения") },
                        placeholder = { Text("1234") },
                        leadingIcon = {
                            Icon(Icons.Default.Lock, contentDescription = null)
                        },
                        modifier = Modifier.fillMaxWidth(),
                        singleLine = true,
                        keyboardOptions = KeyboardOptions(
                            keyboardType = KeyboardType.Number,
                            imeAction = ImeAction.Done
                        ),
                        keyboardActions = KeyboardActions(
                            onDone = { hideKeyboard() }
                        )
                    )

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

                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        OutlinedButton(
                            onClick = {
                                codeSent = false
                                code = ""
                                error = null
                            },
                            modifier = Modifier.weight(1f)
                        ) {
                            Text("Назад")
                        }
                        
                        Button(
                            onClick = {
                                hideKeyboard()
                                isLoading = true
                                error = null
                                scope.launch {
                                    verifyAuthUseCase(phone, code).fold(
                                        onSuccess = {
                                            isLoading = false
                                            onAuthSuccess()
                                        },
                                        onFailure = { e ->
                                            error = e.message ?: "Ошибка подтверждения кода"
                                            isLoading = false
                                        }
                                    )
                                }
                            },
                            enabled = !isLoading && code.isNotBlank(),
                            modifier = Modifier.weight(1f)
                        ) {
                            if (isLoading) {
                                CircularProgressIndicator(
                                    modifier = Modifier.size(20.dp),
                                    color = MaterialTheme.colorScheme.onPrimary
                                )
                                Spacer(modifier = Modifier.width(8.dp))
                            }
                            Text("Подтвердить")
                        }
                    }
                }
            }
        }
    }
}
