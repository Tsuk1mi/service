package com.rimskiy.shared.platform

import android.app.Activity
import android.app.DownloadManager
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.Environment
import android.provider.MediaStore
import android.provider.Settings
import android.util.Log
import android.util.Base64
import androidx.core.content.FileProvider
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import com.rimskiy.shared.di.AndroidContextHolder
import java.io.File
import java.io.FileOutputStream
import java.net.HttpURLConnection
import java.net.URL

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
                setMimeType("application/vnd.android.package-archive")
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

            // Если сервер/прокси защищён DDNS Basic Auth — добавляем заголовки,
            // иначе DownloadManager получит 401/403 и загрузка “тихо” упадёт.
            val ddnsUser = AndroidContextHolder.ddnsUsername
            val ddnsPass = AndroidContextHolder.ddnsPassword
            if (!ddnsUser.isNullOrBlank() && !ddnsPass.isNullOrBlank()) {
                val credentials = "$ddnsUser:$ddnsPass"
                val encoded = Base64.encodeToString(credentials.toByteArray(Charsets.UTF_8), Base64.NO_WRAP)
                val basic = "Basic $encoded"
                request.addRequestHeader("Authorization", basic)
                // Некоторые прокси требуют именно Proxy-Authorization
                request.addRequestHeader("Proxy-Authorization", basic)
                Log.i("PlatformActions", "DDNS Basic Auth headers added to DownloadManager request")
            }
            
            val downloadId = downloadManager.enqueue(request)

            // ВАЖНО: не используем BroadcastReceiver, т.к. на Android 13+ динамическая регистрация
            // требует флага exported/notExported и часто ломает обновление. Вместо этого — polling.
            CoroutineScope(Dispatchers.IO).launch {
                try {
                    while (true) {
                        kotlinx.coroutines.delay(500)
                        val query = DownloadManager.Query().setFilterById(downloadId)
                        val cursor = downloadManager.query(query)
                        if (!cursor.moveToFirst()) {
                            cursor.close()
                            continue
                        }

                        val status = cursor.getInt(cursor.getColumnIndexOrThrow(DownloadManager.COLUMN_STATUS))
                        when (status) {
                            DownloadManager.STATUS_RUNNING -> {
                                val bytesDownloaded = cursor.getLong(cursor.getColumnIndexOrThrow(DownloadManager.COLUMN_BYTES_DOWNLOADED_SO_FAR))
                                val totalBytes = cursor.getLong(cursor.getColumnIndexOrThrow(DownloadManager.COLUMN_TOTAL_SIZE_BYTES))
                                if (totalBytes > 0) {
                                    val progress = ((bytesDownloaded * 100) / totalBytes).toInt().coerceIn(0, 100)
                                    withContext(Dispatchers.Main) { onProgress(progress) }
                                }
                            }
                            DownloadManager.STATUS_SUCCESSFUL -> {
                                val localUriStr = cursor.getString(cursor.getColumnIndexOrThrow(DownloadManager.COLUMN_LOCAL_URI))
                                cursor.close()
                                val uri = localUriStr?.let { Uri.parse(it) }

                                withContext(Dispatchers.Main) {
                                    installDownloadedApk(activity, uri, apkFile, onComplete, onError)
                                }
                                break
                            }
                            DownloadManager.STATUS_FAILED -> {
                                val reason = cursor.getInt(cursor.getColumnIndexOrThrow(DownloadManager.COLUMN_REASON))
                                cursor.close()
                                // Fallback: чтобы получить нормальный HTTP-код/текст и не зависеть от DownloadManager,
                                // пробуем скачать сами.
                                val fallbackOk = tryManualHttpDownload(url, apkFile) { p ->
                                    withContext(Dispatchers.Main) { onProgress(p) }
                                }
                                withContext(Dispatchers.Main) {
                                    if (fallbackOk) {
                                        installDownloadedApk(activity, Uri.fromFile(apkFile), apkFile, onComplete, onError)
                                    } else {
                                        onError("Ошибка загрузки (DownloadManager reason=$reason). Частая причина: сервер не отдаёт APK (404). Проверьте APP_APK_PATH на сервере.")
                                    }
                                }
                                break
                            }
                            else -> {
                                // PENDING/PAUSED — просто ждём
                            }
                        }
                        cursor.close()
                    }
                } catch (e: Exception) {
                    Log.e("PlatformActions", "Download polling failed: ${e.message}", e)
                    withContext(Dispatchers.Main) { onError("Ошибка загрузки: ${e.message}") }
                }
            }
            
        } catch (e: Exception) {
            Log.e("PlatformActions", "Failed to download APK: ${e.message}", e)
            onError("Ошибка загрузки: ${e.message}")
        }
    }

    private suspend fun tryManualHttpDownload(
        url: String,
        destFile: File,
        onProgress: suspend (Int) -> Unit
    ): Boolean = withContext(Dispatchers.IO) {
        try {
            destFile.parentFile?.mkdirs()
            if (destFile.exists()) destFile.delete()

            val conn = (URL(url).openConnection() as HttpURLConnection).apply {
                instanceFollowRedirects = true
                connectTimeout = 15_000
                readTimeout = 60_000
                requestMethod = "GET"
                setRequestProperty("Accept", "application/vnd.android.package-archive")
                // Если когда-то включите DDNS basic-auth — оно автоматически добавится через holder
                val ddnsUser = AndroidContextHolder.ddnsUsername
                val ddnsPass = AndroidContextHolder.ddnsPassword
                if (!ddnsUser.isNullOrBlank() && !ddnsPass.isNullOrBlank()) {
                    val credentials = "$ddnsUser:$ddnsPass"
                    val encoded = Base64.encodeToString(credentials.toByteArray(Charsets.UTF_8), Base64.NO_WRAP)
                    setRequestProperty("Authorization", "Basic $encoded")
                    setRequestProperty("Proxy-Authorization", "Basic $encoded")
                }
            }

            val code = conn.responseCode
            if (code !in 200..299) {
                val msg = try {
                    conn.errorStream?.bufferedReader()?.readText()
                } catch (_: Exception) {
                    null
                }
                Log.e("PlatformActions", "Manual download failed: HTTP $code, body=${msg?.take(300)}")
                return@withContext false
            }

            val total = conn.contentLengthLong.takeIf { it > 0 } ?: -1L
            conn.inputStream.use { input ->
                FileOutputStream(destFile).use { output ->
                    val buf = ByteArray(8 * 1024)
                    var read: Int
                    var downloaded = 0L
                    var lastProgress = -1
                    while (input.read(buf).also { read = it } != -1) {
                        output.write(buf, 0, read)
                        downloaded += read.toLong()
                        if (total > 0) {
                            val p = ((downloaded * 100) / total).toInt().coerceIn(0, 100)
                            if (p != lastProgress) {
                                lastProgress = p
                                onProgress(p)
                            }
                        }
                    }
                }
            }
            true
        } catch (e: Exception) {
            Log.e("PlatformActions", "Manual download exception: ${e.message}", e)
            false
        }
    }
    
    private fun installDownloadedApk(
        activity: Activity,
        downloadedLocalUri: Uri?,
        fallbackApkFile: File,
        onComplete: () -> Unit,
        onError: (String) -> Unit
    ) {
        try {
            val installUri: Uri = when {
                downloadedLocalUri == null -> {
                    // Fallback: используем ожидаемый путь назначения
                    FileProvider.getUriForFile(activity, "${activity.packageName}.fileprovider", fallbackApkFile)
                }
                downloadedLocalUri.scheme.equals("content", ignoreCase = true) -> {
                    // DownloadManager может вернуть content:// — используем как есть
                    downloadedLocalUri
                }
                downloadedLocalUri.scheme.equals("file", ignoreCase = true) -> {
                    // file:// нельзя шарить на Android N+ — конвертируем в FileProvider
                    val file = File(downloadedLocalUri.path ?: fallbackApkFile.absolutePath)
                    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
                        FileProvider.getUriForFile(activity, "${activity.packageName}.fileprovider", file)
                    } else {
                        Uri.fromFile(file)
                    }
                }
                else -> {
                    downloadedLocalUri
                }
            }

            val intent = Intent(Intent.ACTION_VIEW).apply {
                flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_GRANT_READ_URI_PERMISSION
                setDataAndType(installUri, "application/vnd.android.package-archive")
            }
            
            activity.startActivity(intent)
            onComplete()
        } catch (e: Exception) {
            Log.e("PlatformActions", "Failed to install APK: ${e.message}", e)
            onError("Ошибка установки: ${e.message}")
        }
    }
}
