package com.rimskiy.shared.ui.components

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Remove
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
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
    
    // Популярные времена для быстрого выбора
    val quickTimes = listOf("08:00", "09:00", "12:00", "18:00", "20:00", "22:00")
    
    val scrollState = rememberScrollState()

    Dialog(onDismissRequest = onDismiss) {
        Card(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            shape = MaterialTheme.shapes.large,
            colors = CardDefaults.cardColors(
                containerColor = MaterialTheme.colorScheme.surface
            )
        ) {
            Column(
                modifier = Modifier
                    .padding(24.dp)
                    .verticalScroll(scrollState),
                verticalArrangement = Arrangement.spacedBy(20.dp)
            ) {
                Text(
                    text = title,
                    style = MaterialTheme.typography.titleLarge
                )

                Divider()

                // Быстрый выбор популярных времен
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    Text(
                        text = "Быстрый выбор",
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        quickTimes.chunked(3).forEach { rowTimes ->
                            Column(
                                modifier = Modifier.weight(1f),
                                verticalArrangement = Arrangement.spacedBy(8.dp)
                            ) {
                                rowTimes.forEach { time ->
                                    val (h, m) = parseTime(time)
                                    AssistChip(
                                        onClick = {
                                            selectedHour = h
                                            selectedMinute = m
                                        },
                                        label = { Text(time) },
                                        modifier = Modifier.fillMaxWidth(),
                                        colors = AssistChipDefaults.assistChipColors(
                                            containerColor = if (selectedHour == h && selectedMinute == m)
                                                MaterialTheme.colorScheme.primaryContainer
                                            else
                                                MaterialTheme.colorScheme.surfaceVariant
                                        )
                                    )
                                }
                            }
                        }
                    }
                }

                Divider()

                // Ручной выбор времени
                Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
                    Text(
                        text = "Ручной выбор",
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    
                    // Выбор часов
                    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        Text(
                            text = "Часы",
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.spacedBy(12.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            IconButton(
                                onClick = {
                                    selectedHour = (selectedHour - 1).coerceIn(0, 23)
                                },
                                modifier = Modifier.size(56.dp)
                            ) {
                                Icon(
                                    imageVector = Icons.Default.Remove,
                                    contentDescription = "Уменьшить час",
                                    modifier = Modifier.size(28.dp)
                                )
                            }
                            
                            OutlinedTextField(
                                value = selectedHour.toString().padStart(2, '0'),
                                onValueChange = { value ->
                                    val newHour = value.toIntOrNull()?.coerceIn(0, 23) ?: selectedHour
                                    selectedHour = newHour
                                },
                                modifier = Modifier
                                    .weight(1f)
                                    .width(100.dp),
                                singleLine = true
                            )
                            
                            IconButton(
                                onClick = {
                                    selectedHour = (selectedHour + 1).coerceIn(0, 23)
                                },
                                modifier = Modifier.size(56.dp)
                            ) {
                                Icon(
                                    imageVector = Icons.Default.Add,
                                    contentDescription = "Увеличить час",
                                    modifier = Modifier.size(28.dp)
                                )
                            }
                        }
                    }

                    // Выбор минут
                    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        Text(
                            text = "Минуты",
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.spacedBy(12.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            IconButton(
                                onClick = {
                                    selectedMinute = ((selectedMinute - 5) + 60) % 60
                                },
                                modifier = Modifier.size(56.dp)
                            ) {
                                Icon(
                                    imageVector = Icons.Default.Remove,
                                    contentDescription = "Уменьшить минуты",
                                    modifier = Modifier.size(28.dp)
                                )
                            }
                            
                            OutlinedTextField(
                                value = selectedMinute.toString().padStart(2, '0'),
                                onValueChange = { value ->
                                    val newMinute = value.toIntOrNull()?.coerceIn(0, 59) ?: selectedMinute
                                    selectedMinute = newMinute
                                },
                                modifier = Modifier
                                    .weight(1f)
                                    .width(100.dp),
                                singleLine = true
                            )
                            
                            IconButton(
                                onClick = {
                                    selectedMinute = (selectedMinute + 5) % 60
                                },
                                modifier = Modifier.size(56.dp)
                            ) {
                                Icon(
                                    imageVector = Icons.Default.Add,
                                    contentDescription = "Увеличить минуты",
                                    modifier = Modifier.size(28.dp)
                                )
                            }
                        }
                    }
                }

                // Предпросмотр времени
                Card(
                    modifier = Modifier.fillMaxWidth(),
                    colors = CardDefaults.cardColors(
                        containerColor = MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.5f)
                    )
                ) {
                    Column(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(20.dp),
                        horizontalAlignment = Alignment.CenterHorizontally
                    ) {
                        Text(
                            text = "Выбранное время",
                            style = MaterialTheme.typography.labelMedium,
                            color = MaterialTheme.colorScheme.onPrimaryContainer
                        )
                        Spacer(modifier = Modifier.height(8.dp))
                        Text(
                            text = formatTime(selectedHour, selectedMinute),
                            style = MaterialTheme.typography.displayMedium,
                            color = MaterialTheme.colorScheme.onPrimaryContainer,
                            fontWeight = FontWeight.Bold
                        )
                    }
                }

                Divider()

                // Кнопки действий
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    OutlinedButton(
                        onClick = {
                            onTimeSelected(null)
                            onDismiss()
                        },
                        modifier = Modifier.weight(1f)
                    ) {
                        Text("Очистить")
                    }
                    Button(
                        onClick = {
                            onTimeSelected(formatTime(selectedHour, selectedMinute))
                            onDismiss()
                        },
                        modifier = Modifier.weight(1f)
                    ) {
                        Text("Выбрать")
                    }
                }
            }
        }
    }
}
