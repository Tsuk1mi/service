package com.rimskiy.shared.ui.screens

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.Info
import androidx.compose.material.icons.filled.Link
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Divider
import androidx.compose.material3.ElevatedCard
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.Alignment

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AboutScreen(
    appVersion: String,
    latestVersion: String?,
    downloadUrl: String?,
    isUpdateAvailable: Boolean,
    isDownloading: Boolean,
    downloadProgress: Int,
    downloadError: String?,
    onInstallUpdate: (String) -> Unit,
    onBack: () -> Unit
) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(bottom = 16.dp)
    ) {
        TopAppBar(
            title = { Text("О приложении") },
            navigationIcon = {
                IconButton(onClick = onBack) {
                    Icon(Icons.Default.ArrowBack, contentDescription = "Назад")
                }
            },
            colors = TopAppBarDefaults.topAppBarColors(
                containerColor = MaterialTheme.colorScheme.surface
            )
        )

        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(horizontal = 16.dp)
                .padding(top = 8.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            InfoCard(
                title = "Версия приложения",
                subtitle = "Текущая версия клиента",
                value = appVersion
            )

            InfoCard(
                title = "Последняя версия",
                subtitle = "Версия, доступная на сервере",
                value = latestVersion ?: "Не указана"
            )

            if (isUpdateAvailable && !downloadUrl.isNullOrBlank()) {
                ElevatedCard(
                    modifier = Modifier.fillMaxWidth(),
                    elevation = CardDefaults.elevatedCardElevation(defaultElevation = 3.dp),
                    colors = CardDefaults.cardColors(
                        containerColor = MaterialTheme.colorScheme.primaryContainer
                    ),
                    onClick = { onInstallUpdate(downloadUrl) }
                ) {
                    Column(
                        modifier = Modifier.padding(20.dp),
                        verticalArrangement = Arrangement.spacedBy(12.dp)
                    ) {
                        RowIconTitle(
                            icon = Icons.Default.Link,
                            title = "Установить обновление"
                        )
                        Divider(
                            color = MaterialTheme.colorScheme.onPrimaryContainer.copy(alpha = 0.2f),
                            thickness = 1.dp
                        )
                        if (isDownloading) {
                            androidx.compose.foundation.layout.Row(
                                modifier = Modifier.fillMaxWidth(),
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(12.dp)
                            ) {
                                CircularProgressIndicator()
                                Text(
                                    text = "Загрузка: $downloadProgress%",
                                    style = MaterialTheme.typography.bodyMedium,
                                    color = MaterialTheme.colorScheme.onPrimaryContainer
                                )
                            }
                        } else {
                            Text(
                                text = "Нажмите, чтобы скачать APK с сервера и запустить установку.",
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onPrimaryContainer
                            )
                        }

                        if (!downloadError.isNullOrBlank()) {
                            Text(
                                text = downloadError,
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.error
                            )
                        }
                    }
                }
            }

            Spacer(modifier = Modifier.height(8.dp))

            Text(
                text = "Здесь вы можете проверить текущую версию клиента, минимальную версию сервера и ссылку для обновления.",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
    }
}

@Composable
private fun InfoCard(title: String, subtitle: String, value: String) {
    ElevatedCard(
        modifier = Modifier.fillMaxWidth(),
        elevation = CardDefaults.elevatedCardElevation(defaultElevation = 2.dp),
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.surface
        )
    ) {
        Column(
            modifier = Modifier.padding(20.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            RowIconTitle(icon = Icons.Default.Info, title = title)
            Divider(
                color = MaterialTheme.colorScheme.outline.copy(alpha = 0.2f),
                thickness = 1.dp
            )
            Text(
                text = subtitle,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
            Text(
                text = value,
                style = MaterialTheme.typography.titleLarge,
                color = MaterialTheme.colorScheme.onSurface
            )
        }
    }
}

@Composable
private fun RowIconTitle(
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    title: String
) {
    androidx.compose.foundation.layout.Row(
        verticalAlignment = androidx.compose.ui.Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.primary
        )
        Text(title, style = MaterialTheme.typography.titleMedium)
    }
}

