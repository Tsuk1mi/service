package com.rimskiy.shared.ui.components

import androidx.compose.foundation.layout.*
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
 * Диалог выбора времени с простым UI (для всех платформ)
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
                modifier = Modifier.padding(24.dp),
                verticalArrangement = Arrangement.spacedBy(16.dp)
            ) {
                Text(
                    text = title,
                    style = MaterialTheme.typography.titleLarge
                )

                Divider()

                // Выбор часов
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    Text(
                        text = "Часы",
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        IconButton(
                            onClick = {
                                selectedHour = (selectedHour - 1).coerceIn(0, 23)
                            },
                            modifier = Modifier.size(48.dp)
                        ) {
                            Icon(
                                imageVector = Icons.Default.Remove,
                                contentDescription = "Уменьшить час"
                            )
                        }
                        OutlinedTextField(
                            value = selectedHour.toString().padStart(2, '0'),
                            onValueChange = { value ->
                                val newHour = value.toIntOrNull()?.coerceIn(0, 23) ?: 0
                                selectedHour = newHour
                            },
                            modifier = Modifier
                                .weight(1f)
                                .width(80.dp),
                            singleLine = true,
                            textStyle = TextStyle(
                                fontSize = 24.sp,
                                fontWeight = FontWeight.Bold
                            )
                        )
                        IconButton(
                            onClick = {
                                selectedHour = (selectedHour + 1).coerceIn(0, 23)
                            },
                            modifier = Modifier.size(48.dp)
                        ) {
                            Icon(
                                imageVector = Icons.Default.Add,
                                contentDescription = "Увеличить час"
                            )
                        }
                    }
                }

                // Выбор минут
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    Text(
                        text = "Минуты",
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        IconButton(
                            onClick = {
                                selectedMinute = ((selectedMinute - 5) + 60) % 60
                            },
                            modifier = Modifier.size(48.dp)
                        ) {
                            Icon(
                                imageVector = Icons.Default.Remove,
                                contentDescription = "Уменьшить минуты"
                            )
                        }
                        OutlinedTextField(
                            value = selectedMinute.toString().padStart(2, '0'),
                            onValueChange = { value ->
                                val newMinute = value.toIntOrNull()?.coerceIn(0, 59) ?: 0
                                selectedMinute = newMinute
                            },
                            modifier = Modifier
                                .weight(1f)
                                .width(80.dp),
                            singleLine = true,
                            textStyle = TextStyle(
                                fontSize = 24.sp,
                                fontWeight = FontWeight.Bold
                            )
                        )
                        IconButton(
                            onClick = {
                                selectedMinute = (selectedMinute + 5) % 60
                            },
                            modifier = Modifier.size(48.dp)
                        ) {
                            Icon(
                                imageVector = Icons.Default.Add,
                                contentDescription = "Увеличить минуты"
                            )
                        }
                    }
                }

                // Предпросмотр времени
                Card(
                    modifier = Modifier.fillMaxWidth(),
                    colors = CardDefaults.cardColors(
                        containerColor = MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.3f)
                    )
                ) {
                    Text(
                        text = formatTime(selectedHour, selectedMinute),
                        style = MaterialTheme.typography.displaySmall,
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(16.dp),
                        color = MaterialTheme.colorScheme.onPrimaryContainer
                    )
                }

                Divider()

                // Кнопки действий
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(8.dp)
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

