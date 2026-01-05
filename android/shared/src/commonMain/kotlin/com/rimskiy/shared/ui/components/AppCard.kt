package com.rimskiy.shared.ui.components

import androidx.compose.foundation.clickable
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ElevatedCard
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

/**
 * Единая карточка приложения: одинаковые скругления/подъём/цвета на всех экранах.
 */
@Composable
fun AppCard(
    modifier: Modifier = Modifier,
    onClick: (() -> Unit)? = null,
    containerColor: Color = MaterialTheme.colorScheme.surface,
    elevation: androidx.compose.ui.unit.Dp = AppCardDefaults.Elevation,
    content: @Composable () -> Unit,
) {
    val clickableModifier = if (onClick != null) modifier.clickable(onClick = onClick) else modifier

    ElevatedCard(
        modifier = clickableModifier,
        elevation = CardDefaults.elevatedCardElevation(defaultElevation = elevation),
        shape = MaterialTheme.shapes.large,
        colors = CardDefaults.cardColors(containerColor = containerColor),
    ) {
        content()
    }
}

object AppCardDefaults {
    val Elevation = 2.dp
}


