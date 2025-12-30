package com.rimskiy.shared.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Remove
import androidx.compose.material.icons.filled.Schedule
import androidx.compose.material3.*
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ElevatedCard
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog

/**
 * Парсит строку времени в формате HH:MM в часы и минуты
 */
fun parseTime(timeString: String?): Pair<Int, Int> {
    if (timeString.isNullOrBlank()) {
        return Pair(0, 0)
    }
    val parts = timeString.split(":")
    if (parts.size == 2) {
        val hour = parts[0].toIntOrNull() ?: 0
        val minute = parts[1].toIntOrNull() ?: 0
        return Pair(hour.coerceIn(0, 23), minute.coerceIn(0, 59))
    }
    return Pair(0, 0)
}

/**
 * Форматирует часы и минуты в строку HH:MM
 */
fun formatTime(hour: Int, minute: Int): String {
    return String.format("%02d:%02d", hour.coerceIn(0, 23), minute.coerceIn(0, 59))
}

/**
 * Диалог выбора времени с улучшенным UI
 */
@Composable
fun TimePickerDialog(
    initialTime: String?,
    onTimeSelected: (String?) -> Unit,
    onDismiss: () -> Unit,
    title: String = "Выберите время"
) {
    val (hour, minute) = parseTime(initialTime)
    var selectedHour by remember { mutableStateOf(hour) }
    var selectedMinute by remember { mutableStateOf(minute) }
    
    val scrollState = rememberScrollState()

    Dialog(onDismissRequest = onDismiss) {
        ElevatedCard(
            modifier = Modifier
                .fillMaxWidth(0.9f)
                .padding(16.dp),
            elevation = CardDefaults.elevatedCardElevation(defaultElevation = 8.dp),
            shape = MaterialTheme.shapes.large,
            colors = CardDefaults.cardColors(
                containerColor = MaterialTheme.colorScheme.surface
            )
        ) {
            Column(
                modifier = Modifier
                    .padding(20.dp)
                    .verticalScroll(scrollState),
                verticalArrangement = Arrangement.spacedBy(16.dp)
            ) {
                // Заголовок с иконкой
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    Icon(
                        imageVector = Icons.Default.Schedule,
                        contentDescription = null,
                        modifier = Modifier.size(28.dp),
                        tint = MaterialTheme.colorScheme.primary
                    )
                    Text(
                        text = title,
                        style = MaterialTheme.typography.titleLarge,
                        fontWeight = FontWeight.Bold
                    )
                }

                Divider(
                    modifier = Modifier.padding(vertical = 4.dp),
                    thickness = 1.dp
                )

                // Ручной выбор времени - компактный layout
                Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                    Text(
                        text = "Ручной выбор",
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    
                    // Компактный выбор часов и минут в одной строке
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(16.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        // Выбор часов
                        Column(
                            modifier = Modifier.weight(1f),
                            horizontalAlignment = Alignment.CenterHorizontally,
                            verticalArrangement = Arrangement.spacedBy(4.dp)
                        ) {
                            Text(
                                text = "Часы",
                                style = MaterialTheme.typography.labelSmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                            Row(
                                horizontalArrangement = Arrangement.spacedBy(8.dp),
                                verticalAlignment = Alignment.CenterVertically
                            ) {
                                IconButton(
                                    onClick = {
                                        selectedHour = (selectedHour - 1).coerceIn(0, 23)
                                    },
                                    modifier = Modifier.size(40.dp)
                                ) {
                                    Icon(
                                        imageVector = Icons.Default.Remove,
                                        contentDescription = "Уменьшить час",
                                        modifier = Modifier.size(20.dp)
                                    )
                                }
                                
                                OutlinedTextField(
                                    value = selectedHour.toString().padStart(2, '0'),
                                    onValueChange = { value ->
                                        val newHour = value.toIntOrNull()?.coerceIn(0, 23) ?: selectedHour
                                        selectedHour = newHour
                                    },
                                    modifier = Modifier.width(60.dp),
                                    singleLine = true,
                                    textStyle = TextStyle(
                                        fontSize = 18.sp,
                                        fontWeight = FontWeight.Bold
                                    )
                                )
                                
                                IconButton(
                                    onClick = {
                                        selectedHour = (selectedHour + 1).coerceIn(0, 23)
                                    },
                                    modifier = Modifier.size(40.dp)
                                ) {
                                    Icon(
                                        imageVector = Icons.Default.Add,
                                        contentDescription = "Увеличить час",
                                        modifier = Modifier.size(20.dp)
                                    )
                                }
                            }
                        }
                        
                        // Разделитель
                        Text(
                            text = ":",
                            style = MaterialTheme.typography.headlineMedium,
                            modifier = Modifier.padding(vertical = 8.dp)
                        )
                        
                        // Выбор минут
                        Column(
                            modifier = Modifier.weight(1f),
                            horizontalAlignment = Alignment.CenterHorizontally,
                            verticalArrangement = Arrangement.spacedBy(4.dp)
                        ) {
                            Text(
                                text = "Минуты",
                                style = MaterialTheme.typography.labelSmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                            Row(
                                horizontalArrangement = Arrangement.spacedBy(8.dp),
                                verticalAlignment = Alignment.CenterVertically
                            ) {
                                IconButton(
                                    onClick = {
                                        selectedMinute = ((selectedMinute - 5) + 60) % 60
                                    },
                                    modifier = Modifier.size(40.dp)
                                ) {
                                    Icon(
                                        imageVector = Icons.Default.Remove,
                                        contentDescription = "Уменьшить минуты",
                                        modifier = Modifier.size(20.dp)
                                    )
                                }
                                
                                OutlinedTextField(
                                    value = selectedMinute.toString().padStart(2, '0'),
                                    onValueChange = { value ->
                                        val newMinute = value.toIntOrNull()?.coerceIn(0, 59) ?: selectedMinute
                                        selectedMinute = newMinute
                                    },
                                    modifier = Modifier.width(60.dp),
                                    singleLine = true,
                                    textStyle = TextStyle(
                                        fontSize = 18.sp,
                                        fontWeight = FontWeight.Bold
                                    )
                                )
                                
                                IconButton(
                                    onClick = {
                                        selectedMinute = (selectedMinute + 5) % 60
                                    },
                                    modifier = Modifier.size(40.dp)
                                ) {
                                    Icon(
                                        imageVector = Icons.Default.Add,
                                        contentDescription = "Увеличить минуты",
                                        modifier = Modifier.size(20.dp)
                                    )
                                }
                            }
                        }
                    }
                }

                // Предпросмотр времени - улучшенный дизайн с градиентом
                ElevatedCard(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(16.dp)),
                    colors = CardDefaults.cardColors(
                        containerColor = MaterialTheme.colorScheme.primaryContainer
                    ),
                    elevation = CardDefaults.elevatedCardElevation(defaultElevation = 4.dp)
                ) {
                    Column(
                        modifier = Modifier
                            .fillMaxWidth()
                            .background(
                                Brush.verticalGradient(
                                    colors = listOf(
                                        MaterialTheme.colorScheme.primaryContainer,
                                        MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.8f)
                                    )
                                )
                            )
                            .padding(20.dp),
                        horizontalAlignment = Alignment.CenterHorizontally,
                        verticalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        Text(
                            text = "Выбрано время",
                            style = MaterialTheme.typography.labelLarge,
                            color = MaterialTheme.colorScheme.onPrimaryContainer.copy(alpha = 0.8f)
                        )
                        Text(
                            text = formatTime(selectedHour, selectedMinute),
                            style = MaterialTheme.typography.displaySmall,
                            color = MaterialTheme.colorScheme.onPrimaryContainer,
                            fontWeight = FontWeight.Bold,
                            textAlign = TextAlign.Center
                        )
                    }
                }

                Divider()

                // Кнопки действий - улучшенный дизайн
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    OutlinedButton(
                        onClick = {
                            selectedHour = 0
                            selectedMinute = 0
                            onTimeSelected(null)
                            onDismiss()
                        },
                        modifier = Modifier.weight(1f),
                        colors = ButtonDefaults.outlinedButtonColors(
                            contentColor = MaterialTheme.colorScheme.error
                        )
                    ) {
                        Text(
                            "Очистить",
                            fontWeight = FontWeight.Medium
                        )
                    }
                    Button(
                        onClick = {
                            onTimeSelected(formatTime(selectedHour, selectedMinute))
                            onDismiss()
                        },
                        modifier = Modifier.weight(1f),
                        colors = ButtonDefaults.buttonColors(
                            containerColor = MaterialTheme.colorScheme.primary
                        )
                    ) {
                        Icon(
                            imageVector = Icons.Default.Schedule,
                            contentDescription = null,
                            modifier = Modifier.size(18.dp)
                        )
                        Spacer(modifier = Modifier.width(8.dp))
                        Text(
                            "Выбрать",
                            fontWeight = FontWeight.Bold
                        )
                    }
                }
            }
        }
    }
}
