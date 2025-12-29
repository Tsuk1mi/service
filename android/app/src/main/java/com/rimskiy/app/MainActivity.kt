package com.rimskiy.app

import android.Manifest
import android.content.Intent
import android.content.pm.PackageManager
import android.graphics.Bitmap
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.provider.MediaStore
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import com.rimskiy.shared.di.AndroidContextHolder
import com.rimskiy.shared.platform.CameraHandler
import com.rimskiy.shared.ui.RimskiyApp
import java.io.ByteArrayOutputStream

class MainActivity : ComponentActivity() {
    // Activity Result Launcher для камеры
    private val cameraLauncher = registerForActivityResult(ActivityResultContracts.StartActivityForResult()) { result ->
        if (result.resultCode == RESULT_OK) {
            val data = result.data
            // Получаем фото из Intent
            val imageBitmap = data?.extras?.get("data") as? Bitmap
            
            if (imageBitmap != null) {
                // Конвертируем Bitmap в ByteArray
                try {
                    // Улучшенная обработка изображения для OCR:
                    // 1. Поворачиваем изображение, если нужно
                    // 2. Обрезаем и увеличиваем контрастность
                    // 3. Сжимаем для отправки на сервер
                    
                    // Получаем ориентацию из EXIF (если есть)
                    var processedBitmap = imageBitmap
                    
                    // Сжимаем изображение для отправки (макс 1920x1920 для лучшего качества OCR)
                    val maxSize = 1920
                    val scaledBitmap = if (processedBitmap.width > maxSize || processedBitmap.height > maxSize) {
                        val scale = minOf(maxSize.toFloat() / processedBitmap.width, maxSize.toFloat() / processedBitmap.height)
                        val scaled = Bitmap.createScaledBitmap(
                            processedBitmap,
                            (processedBitmap.width * scale).toInt(),
                            (processedBitmap.height * scale).toInt(),
                            true
                        )
                        scaled
                    } else {
                        processedBitmap
                    }
                    
                    // Конвертируем в JPEG с высоким качеством для лучшего распознавания
                    val outputStream = ByteArrayOutputStream()
                    scaledBitmap.compress(Bitmap.CompressFormat.JPEG, 90, outputStream)
                    val imageBytes = outputStream.toByteArray()
                    outputStream.close()
                    
                    // Освобождаем память
                    if (scaledBitmap != processedBitmap && processedBitmap != imageBitmap) {
                        scaledBitmap.recycle()
                    }
                    if (processedBitmap != imageBitmap) {
                        processedBitmap.recycle()
                    }
                    imageBitmap.recycle()
                    
                    // Вызываем callback
                    CameraHandler.handlePhotoResult(this, null, imageBytes)
                } catch (e: Exception) {
                    Log.e("MainActivity", "Failed to process photo: ${e.message}", e)
                    CameraHandler.handlePhotoResult(this, null, null)
                }
            } else {
                // Если фото не в extras, пробуем получить из URI
                val imageUri = data?.data
                if (imageUri != null) {
                    CameraHandler.handlePhotoResult(this, imageUri, null)
                } else {
                    CameraHandler.handlePhotoResult(this, null, null)
                }
            }
        } else {
            // Пользователь отменил съемку
            CameraHandler.handlePhotoResult(this, null, null)
        }
    }
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        // Сохраняем экземпляр для доступа из PlatformActions
        instance = this
        
        // Инициализируем контекст для PlatformActions (используем Activity context для startActivity)
        AndroidContextHolder.context = this
        
        requestRuntimePermissions()
        
        // Получаем URL и DDNS учетные данные из BuildConfig
        val apiUrl = BuildConfig.API_BASE_URL
        val ddnsUsername = try {
            val username = BuildConfig.DDNS_USERNAME
            if (username == "null") null else username
        } catch (e: Exception) {
            null
        }
        val ddnsPassword = try {
            val password = BuildConfig.DDNS_PASSWORD
            if (password == "null") null else password
        } catch (e: Exception) {
            null
        }
        
        setContent {
            RimskiyApp(
                baseUrl = apiUrl,
                ddnsUsername = ddnsUsername,
                ddnsPassword = ddnsPassword
            )
        }
    }
    
    private fun requestRuntimePermissions() {
        val permissions = mutableListOf<String>()
        
        if (ContextCompat.checkSelfPermission(this, Manifest.permission.CAMERA) != PackageManager.PERMISSION_GRANTED) {
            permissions += Manifest.permission.CAMERA
        }
        
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU &&
            ContextCompat.checkSelfPermission(this, Manifest.permission.POST_NOTIFICATIONS) != PackageManager.PERMISSION_GRANTED
        ) {
            permissions += Manifest.permission.POST_NOTIFICATIONS
        }
        
        if (permissions.isNotEmpty()) {
            ActivityCompat.requestPermissions(this, permissions.toTypedArray(), PERMISSION_REQUEST_CODE)
        }
    }
    
    fun launchCamera() {
        val intent = Intent(MediaStore.ACTION_IMAGE_CAPTURE)
        if (intent.resolveActivity(packageManager) != null) {
            cameraLauncher.launch(intent)
        }
    }
    
    companion object {
        private const val PERMISSION_REQUEST_CODE = 1001
        @JvmStatic
        var instance: MainActivity? = null
    }
}

