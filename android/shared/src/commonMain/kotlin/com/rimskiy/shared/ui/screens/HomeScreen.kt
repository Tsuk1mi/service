package com.rimskiy.shared.ui.screens

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Home
import androidx.compose.material.icons.filled.List
import androidx.compose.material.icons.filled.Notifications
import androidx.compose.material.icons.filled.Warning
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@Composable
fun HomeScreen(
    onNavigateToProfile: () -> Unit,
    onNavigateToMyBlocks: () -> Unit,
    onNavigateToBlockedBy: () -> Unit,
    onNavigateToNotifications: () -> Unit,
) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Card(
            modifier = Modifier.fillMaxWidth(),
            elevation = CardDefaults.cardElevation(defaultElevation = 2.dp),
            colors = CardDefaults.cardColors(
                containerColor = MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.25f)
            )
        ) {
            Column(
                modifier = Modifier.padding(16.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                Icon(
                    imageVector = Icons.Default.Home,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.primary
                )
                Text("Главное", style = MaterialTheme.typography.titleMedium)
                Text(
                    "Быстрый доступ к основным разделам.",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
        }

        Card(
            modifier = Modifier.fillMaxWidth(),
            elevation = CardDefaults.cardElevation(defaultElevation = 1.dp)
        ) {
            Column(
                modifier = Modifier.padding(16.dp),
                verticalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                Button(
                    onClick = onNavigateToProfile,
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Icon(Icons.Default.Home, contentDescription = null)
                    androidx.compose.foundation.layout.Spacer(modifier = Modifier.height(0.dp))
                    Text("Профиль")
                }
                Button(
                    onClick = onNavigateToMyBlocks,
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Icon(Icons.Default.List, contentDescription = null)
                    androidx.compose.foundation.layout.Spacer(modifier = Modifier.height(0.dp))
                    Text("Мои блокировки")
                }
                Button(
                    onClick = onNavigateToBlockedBy,
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Icon(Icons.Default.Warning, contentDescription = null)
                    androidx.compose.foundation.layout.Spacer(modifier = Modifier.height(0.dp))
                    Text("Меня перекрыли")
                }
                Button(
                    onClick = onNavigateToNotifications,
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Icon(Icons.Default.Notifications, contentDescription = null)
                    androidx.compose.foundation.layout.Spacer(modifier = Modifier.height(0.dp))
                    Text("Уведомления")
                }
            }
        }
    }
}

