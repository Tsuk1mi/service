package com.rimskiy.shared.platform

import android.net.Uri

object CameraHandler {
    private var photoCallback: ((ByteArray?) -> Unit)? = null

    fun setPhotoCallback(callback: (ByteArray?) -> Unit) {
        photoCallback = callback
    }

    fun clearPhotoCallback() {
        photoCallback = null
    }

    // Совместимость с вызовами из MainActivity
    fun handlePhotoResult(@Suppress("UNUSED_PARAMETER") activity: Any?, imageUri: Uri?, data: ByteArray?) {
        // Если байты переданы напрямую, используем их, иначе заглушка
        val bytes = data
        photoCallback?.invoke(bytes)
        clearPhotoCallback()
    }
}

