package com.rimskiy.shared.platform

expect class PlatformActions {
    fun openPhone(phone: String)
    fun openSms(phone: String)
    fun openTelegram(username: String)
    fun takePhoto(callback: (ByteArray?) -> Unit)
    fun downloadAndInstallApk(url: String, onProgress: (Int) -> Unit, onComplete: () -> Unit, onError: (String) -> Unit)
}

