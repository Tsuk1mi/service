package com.rimskiy.shared.platform

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.pm.PackageInstaller
import android.os.Build
import android.util.Log

/**
 * Получает результат установки APK через PackageInstaller.Session.commit().
 * Регистрируется в manifest приложения.
 */
class ApkInstallReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        val status = intent.getIntExtra(PackageInstaller.EXTRA_STATUS, PackageInstaller.STATUS_FAILURE)
        val message = intent.getStringExtra(PackageInstaller.EXTRA_STATUS_MESSAGE)

        when (status) {
            PackageInstaller.STATUS_SUCCESS -> {
                Log.i("ApkInstallReceiver", "Install success")
                ApkInstallResultCallbacks.complete?.invoke()
                ApkInstallResultCallbacks.clear()
            }
            PackageInstaller.STATUS_PENDING_USER_ACTION -> {
                // На большинстве устройств (и особенно Samsung) нужно вручную запустить Intent подтверждения установки,
                // который приходит в EXTRA_INTENT.
                val confirmIntent: Intent? = try {
                    if (Build.VERSION.SDK_INT >= 33) {
                        intent.getParcelableExtra(Intent.EXTRA_INTENT, Intent::class.java)
                    } else {
                        @Suppress("DEPRECATION")
                        intent.getParcelableExtra(Intent.EXTRA_INTENT) as? Intent
                    }
                } catch (e: Exception) {
                    Log.e("ApkInstallReceiver", "Failed to read EXTRA_INTENT: ${e.message}", e)
                    null
                }

                if (confirmIntent == null) {
                    Log.e("ApkInstallReceiver", "Pending user action but confirm intent is null")
                    ApkInstallResultCallbacks.error?.invoke("Требуется подтверждение установки, но intent не получен")
                    ApkInstallResultCallbacks.clear()
                    return
                }

                Log.i("ApkInstallReceiver", "Pending user action: starting confirmation UI")
                confirmIntent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                try {
                    context.startActivity(confirmIntent)
                } catch (e: Exception) {
                    Log.e("ApkInstallReceiver", "Failed to start confirmation UI: ${e.message}", e)
                    ApkInstallResultCallbacks.error?.invoke("Не удалось открыть окно подтверждения установки: ${e.message}")
                    ApkInstallResultCallbacks.clear()
                }
            }
            else -> {
                val human = when (status) {
                    PackageInstaller.STATUS_FAILURE_ABORTED -> "Установка отменена пользователем"
                    PackageInstaller.STATUS_FAILURE_BLOCKED -> "Установка заблокирована системой"
                    PackageInstaller.STATUS_FAILURE_CONFLICT -> "Конфликт: уже установлена другая версия/подпись"
                    PackageInstaller.STATUS_FAILURE_INCOMPATIBLE -> "APK несовместим с устройством"
                    PackageInstaller.STATUS_FAILURE_INVALID -> "APK повреждён или неверный формат"
                    PackageInstaller.STATUS_FAILURE_STORAGE -> "Недостаточно памяти для установки"
                    else -> "Ошибка установки"
                }

                val details = message?.takeIf { it.isNotBlank() }?.let { ": $it" }.orEmpty()
                val err = "$human$details"
                Log.e("ApkInstallReceiver", "Install failed: $status $message")
                ApkInstallResultCallbacks.error?.invoke(err)
                ApkInstallResultCallbacks.clear()
            }
        }
    }
}

internal object ApkInstallResultCallbacks {
    var complete: (() -> Unit)? = null
    var error: ((String) -> Unit)? = null

    fun clear() {
        complete = null
        error = null
    }
}


