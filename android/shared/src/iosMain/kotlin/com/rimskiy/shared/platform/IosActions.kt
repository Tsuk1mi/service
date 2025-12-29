package com.rimskiy.shared.platform

import platform.UIKit.UIApplication
import platform.Foundation.NSURL

actual class PlatformActions {
    actual fun openPhone(phone: String) {
        val url = NSURL.URLWithString("tel://$phone")
        url?.let { UIApplication.sharedApplication.openURL(it) }
    }

    actual fun openSms(phone: String) {
        val url = NSURL.URLWithString("sms://$phone")
        url?.let { UIApplication.sharedApplication.openURL(it) }
    }

    actual fun openTelegram(username: String) {
        val usernameClean = username.removePrefix("@")
        val url = NSURL.URLWithString("https://t.me/$usernameClean")
        url?.let { UIApplication.sharedApplication.openURL(it) }
    }
}

