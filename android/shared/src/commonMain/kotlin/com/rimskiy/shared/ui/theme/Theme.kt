package com.rimskiy.shared.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp

private val DarkColorScheme = darkColorScheme(
    primary = Color(0xFFBB86FC),
    secondary = Color(0xFF03DAC6),
    tertiary = Color(0xFF3700B3),
    error = Color(0xFFCF6679),
    errorContainer = Color(0xFF8B0000),
    onError = Color.White,
    onErrorContainer = Color(0xFFFFCCCC),
    background = Color(0xFF121212),
    surface = Color(0xFF1E1E1E),
    surfaceVariant = Color(0xFF2C2C2C),
    onPrimary = Color.Black,
    onSecondary = Color.Black,
    onBackground = Color.White,
    onSurface = Color.White,
    onSurfaceVariant = Color(0xFFB0B0B0)
)

// Мягкая цветовая палитра: пастельные тона
private val LightColorScheme = lightColorScheme(
    primary = Color(0xFFB8A082), // Мягкий беж
    secondary = Color(0xFFC8B5A5), // Светлый беж
    tertiary = Color(0xFFA8957A), // Теплый коричневый
    error = Color(0xFFD9776C), // Мягкий красный
    errorContainer = Color(0xFFFFEBE9),
    onError = Color.White,
    onErrorContainer = Color(0xFF9A4D47),
    primaryContainer = Color(0xFFF5EFE8), // Очень светлый беж
    onPrimaryContainer = Color(0xFF4A4238),
    background = Color(0xFFFAF8F5), // Почти белый с теплым оттенком
    surface = Color(0xFFFFFEFC), // Почти белый
    surfaceVariant = Color(0xFFF0EBE3), // Очень светлый беж
    onPrimary = Color.White,
    onSecondary = Color.White,
    onBackground = Color(0xFF2C2C2C), // Мягкий темный
    onSurface = Color(0xFF3A3A3A), // Более мягкий темный
    onSurfaceVariant = Color(0xFF6B6B6B) // Мягкий серый
)

// Мягкие скругленные формы
private val SoftShapes = Shapes(
    extraSmall = RoundedCornerShape(8.dp),
    small = RoundedCornerShape(12.dp),
    medium = RoundedCornerShape(16.dp),
    large = RoundedCornerShape(24.dp),
    extraLarge = RoundedCornerShape(32.dp)
)

@Composable
fun RimskiyTheme(
    darkTheme: Boolean = false, // Всегда светлая тема в римском стиле
    content: @Composable () -> Unit
) {
    MaterialTheme(
        colorScheme = LightColorScheme,
        shapes = SoftShapes,
        content = content
    )
}

