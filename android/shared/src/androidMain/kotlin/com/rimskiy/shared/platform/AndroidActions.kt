package com.rimskiy.shared.platform

import android.app.Activity
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.provider.MediaStore
import android.util.Log

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
}
