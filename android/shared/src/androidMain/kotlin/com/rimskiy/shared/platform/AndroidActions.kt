package com.rimskiy.shared.platform

import android.app.Activity
import android.app.DownloadManager
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.net.Uri
import android.os.Build
import android.os.Environment
import android.provider.MediaStore
import android.provider.Settings
import android.util.Log
import androidx.core.content.FileProvider
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import java.io.File

actual class PlatformActions(private val context: Context) {
    actual fun openPhone(phone: String) {
        try {
            val cleanPhone = phone.trim()
            if (cleanPhone.isEmpty()) {
                Log.e("PlatformActions", "Phone number is empty")
                return
            }
            val intent = Intent(Intent.ACTION_DIAL).apply {
                data = Uri.parse("tel:$cleanPhone")
                flags = Intent.FLAG_ACTIVITY_NEW_TASK
            }
            val chooser = Intent.createChooser(intent, "Выберите приложение для звонка")
            chooser.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            context.startActivity(chooser)
        } catch (e: Exception) {
            Log.e("PlatformActions", "Failed to open phone: ${e.message}", e)
        }
    }

    actual fun openSms(phone: String) {
        try {
            val cleanPhone = phone.trim()
            if (cleanPhone.isEmpty()) {
                Log.e("PlatformActions", "Phone number is empty")
                return
            }
            val intent = Intent(Intent.ACTION_SENDTO).apply {
                data = Uri.parse("sms:$cleanPhone")
                flags = Intent.FLAG_ACTIVITY_NEW_TASK
            }
            val chooser = Intent.createChooser(intent, "Выберите приложение для SMS")
            chooser.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            context.startActivity(chooser)
        } catch (e: Exception) {
            Log.e("PlatformActions", "Failed to open SMS: ${e.message}", e)
        }
    }

    actual fun openTelegram(username: String) {
        try {
            val usernameClean = username.trim().removePrefix("@")
            if (usernameClean.isEmpty()) {
                Log.e("PlatformActions", "Telegram username is empty")
                return
            }
            // Пробуем открыть через приложение Telegram
            val telegramIntent = Intent(Intent.ACTION_VIEW).apply {
                data = Uri.parse("tg://resolve?domain=$usernameClean")
                flags = Intent.FLAG_ACTIVITY_NEW_TASK
            }
            
            // Если приложение Telegram не установлено, открываем через браузер
            val webIntent = Intent(Intent.ACTION_VIEW).apply {
                data = Uri.parse("https://t.me/$usernameClean")
                flags = Intent.FLAG_ACTIVITY_NEW_TASK
            }
            
            val chooser = Intent.createChooser(telegramIntent, "Открыть Telegram")
            chooser.putExtra(Intent.EXTRA_INITIAL_INTENTS, arrayOf(webIntent))
            chooser.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            
            try {
                context.startActivity(chooser)
            } catch (e: Exception) {
                // Если не получилось через chooser, пробуем напрямую через браузер
                context.startActivity(webIntent)
            }
        } catch (e: Exception) {
            Log.e("PlatformActions", "Failed to open Telegram: ${e.message}", e)
        }
    }
    
    actual fun takePhoto(callback: (ByteArray?) -> Unit) {
        try {
            if (context !is Activity) {
                Log.e("PlatformActions", "Context is not an Activity, cannot take photo")
                callback(null)
                return
            }
            
            // Устанавливаем callback
            CameraHandler.setPhotoCallback(callback)
            
            // Создаем Intent для камеры
            val intent = Intent(MediaStore.ACTION_IMAGE_CAPTURE)
            
            // Проверяем, есть ли приложение камеры
            if (intent.resolveActivity(context.packageManager) != null) {
                // Используем рефлексию для доступа к MainActivity.instance и вызова launchCamera
                try {
                    val mainActivityClass = Class.forName("com.rimskiy.app.MainActivity")
                    val instanceField = mainActivityClass.getDeclaredField("instance")
                    instanceField.isAccessible = true
                    val mainActivity = instanceField.get(null)
                    
                    if (mainActivity != null) {
                        val launchMethod = mainActivityClass.getMethod("launchCamera")
                        launchMethod.invoke(mainActivity)
                    } else {
                        Log.e("PlatformActions", "MainActivity instance not available")
                        CameraHandler.clearPhotoCallback()
                        callback(null)
                    }
                } catch (e: Exception) {
                    Log.e("PlatformActions", "Failed to launch camera: ${e.message}", e)
                    CameraHandler.clearPhotoCallback()
                    callback(null)
                }
            } else {
                Log.e("PlatformActions", "No camera app found")
                CameraHandler.clearPhotoCallback()
                callback(null)
            }
        } catch (e: Exception) {
            Log.e("PlatformActions", "Failed to take photo: ${e.message}", e)
            CameraHandler.clearPhotoCallback()
            callback(null)
        }
    }
    
    actual fun downloadAndInstallApk(url: String, onProgress: (Int) -> Unit, onComplete: () -> Unit, onError: (String) -> Unit) {
        try {
            if (context !is Activity) {
                onError("Context is not an Activity")
                return
            }
            
            val activity = context as Activity
            
            // Проверяем разрешение на установку из неизвестных источников (Android 8.0+)
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                if (!activity.packageManager.canRequestPackageInstalls()) {
                    // Запрашиваем разрешение
                    val intent = Intent(Settings.ACTION_MANAGE_UNKNOWN_APP_SOURCES).apply {
                        data = Uri.parse("package:${activity.packageName}")
                        flags = Intent.FLAG_ACTIVITY_NEW_TASK
                    }
                    try {
                        activity.startActivity(intent)
                        onError("Пожалуйста, разрешите установку из неизвестных источников в настройках")
                    } catch (e: Exception) {
                        Log.e("PlatformActions", "Failed to open settings: ${e.message}", e)
                        onError("Не удалось открыть настройки. Разрешите установку из неизвестных источников вручную.")
                    }
                    return
                }
            }
            
            // Создаем директорию для загрузки, если её нет
            val downloadDir = File(context.getExternalFilesDir(Environment.DIRECTORY_DOWNLOADS), "updates")
            if (!downloadDir.exists()) {
                downloadDir.mkdirs()
            }
            
            val apkFile = File(downloadDir, "app-update.apk")
            
            // Если файл уже существует, удаляем его
            if (apkFile.exists()) {
                apkFile.delete()
            }
            
            // Запускаем загрузку через DownloadManager
            val downloadManager = context.getSystemService(Context.DOWNLOAD_SERVICE) as DownloadManager
            val request = DownloadManager.Request(Uri.parse(url)).apply {
                setTitle("Обновление приложения")
                setDescription("Загрузка новой версии приложения...")
                // Для Android 10+ используем getExternalFilesDir, для старых версий - setDestinationUri
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                    setDestinationInExternalFilesDir(context, Environment.DIRECTORY_DOWNLOADS, "updates/app-update.apk")
                } else {
                    @Suppress("DEPRECATION")
                    setDestinationUri(Uri.fromFile(apkFile))
                }
                setNotificationVisibility(DownloadManager.Request.VISIBILITY_VISIBLE_NOTIFY_COMPLETED)
                setAllowedOverMetered(true)
                setAllowedOverRoaming(true)
            }
            
            val downloadId = downloadManager.enqueue(request)
            
            // Отслеживаем прогресс загрузки
            val receiver = object : BroadcastReceiver() {
                override fun onReceive(context: Context?, intent: Intent?) {
                    val id = intent?.getLongExtra(DownloadManager.EXTRA_DOWNLOAD_ID, -1)
                    if (id == downloadId) {
                        val query = DownloadManager.Query().setFilterById(downloadId)
                        val cursor = downloadManager.query(query)
                        
                        if (cursor.moveToFirst()) {
                            val status = cursor.getInt(cursor.getColumnIndexOrThrow(DownloadManager.COLUMN_STATUS))
                            when (status) {
                                DownloadManager.STATUS_SUCCESSFUL -> {
                                    context?.unregisterReceiver(this)
                                    // Получаем путь к загруженному файлу
                                    val downloadedFile = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                                        val externalDir = context?.getExternalFilesDir(Environment.DIRECTORY_DOWNLOADS)
                                        if (externalDir != null) {
                                            File(externalDir, "updates/app-update.apk")
                                        } else {
                                            apkFile
                                        }
                                    } else {
                                        apkFile
                                    }
                                    // Устанавливаем APK
                                    installApk(activity, downloadedFile, onComplete, onError)
                                }
                                DownloadManager.STATUS_FAILED -> {
                                    context?.unregisterReceiver(this)
                                    val reason = cursor.getInt(cursor.getColumnIndexOrThrow(DownloadManager.COLUMN_REASON))
                                    onError("Ошибка загрузки: $reason")
                                }
                                DownloadManager.STATUS_RUNNING -> {
                                    val bytesDownloaded = cursor.getInt(cursor.getColumnIndexOrThrow(DownloadManager.COLUMN_BYTES_DOWNLOADED_SO_FAR))
                                    val totalBytes = cursor.getInt(cursor.getColumnIndexOrThrow(DownloadManager.COLUMN_TOTAL_SIZE_BYTES))
                                    if (totalBytes > 0) {
                                        val progress = (bytesDownloaded * 100 / totalBytes).toInt()
                                        onProgress(progress)
                                    }
                                }
                            }
                        }
                        cursor.close()
                    }
                }
            }
            
            context.registerReceiver(receiver, IntentFilter(DownloadManager.ACTION_DOWNLOAD_COMPLETE))
            
            // Также проверяем прогресс периодически
            CoroutineScope(Dispatchers.IO).launch {
                while (true) {
                    kotlinx.coroutines.delay(500)
                    val query = DownloadManager.Query().setFilterById(downloadId)
                    val cursor = downloadManager.query(query)
                    
                    if (cursor.moveToFirst()) {
                        val status = cursor.getInt(cursor.getColumnIndexOrThrow(DownloadManager.COLUMN_STATUS))
                        if (status == DownloadManager.STATUS_RUNNING) {
                            val bytesDownloaded = cursor.getInt(cursor.getColumnIndexOrThrow(DownloadManager.COLUMN_BYTES_DOWNLOADED_SO_FAR))
                            val totalBytes = cursor.getInt(cursor.getColumnIndexOrThrow(DownloadManager.COLUMN_TOTAL_SIZE_BYTES))
                            if (totalBytes > 0) {
                                val progress = (bytesDownloaded * 100 / totalBytes).toInt()
                                CoroutineScope(Dispatchers.Main).launch {
                                    onProgress(progress)
                                }
                            }
                        } else if (status == DownloadManager.STATUS_SUCCESSFUL || status == DownloadManager.STATUS_FAILED) {
                            cursor.close()
                            break
                        }
                    }
                    cursor.close()
                }
            }
            
        } catch (e: Exception) {
            Log.e("PlatformActions", "Failed to download APK: ${e.message}", e)
            onError("Ошибка загрузки: ${e.message}")
        }
    }
    
    private fun installApk(activity: Activity, apkFile: File, onComplete: () -> Unit, onError: (String) -> Unit) {
        try {
            val intent = Intent(Intent.ACTION_VIEW).apply {
                flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_GRANT_READ_URI_PERMISSION
                
                val uri = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
                    // Используем FileProvider для Android 7.0+
                    FileProvider.getUriForFile(
                        activity,
                        "${activity.packageName}.fileprovider",
                        apkFile
                    )
                } else {
                    // Для старых версий Android
                    Uri.fromFile(apkFile)
                }
                
                setDataAndType(uri, "application/vnd.android.package-archive")
            }
            
            activity.startActivity(intent)
            onComplete()
        } catch (e: Exception) {
            Log.e("PlatformActions", "Failed to install APK: ${e.message}", e)
            onError("Ошибка установки: ${e.message}")
        }
    }
}
