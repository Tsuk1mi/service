package com.rimskiy.shared.platform

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.pm.PackageInstaller
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
                // Обычно сюда приходит Intent, который нужно запустить, но на большинстве устройств
                // commit() сам инициирует UI. Оставляем лог для диагностики.
                Log.i("ApkInstallReceiver", "Pending user action")
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


