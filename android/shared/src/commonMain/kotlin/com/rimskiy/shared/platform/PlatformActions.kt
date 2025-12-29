package com.rimskiy.shared.platform

expect class PlatformActions {
    fun openPhone(phone: String)
    fun openSms(phone: String)
    fun openTelegram(username: String)
    fun takePhoto(callback: (ByteArray?) -> Unit)
}

