package com.rimskiy.shared.platform

import com.russhwolf.settings.Settings
import com.russhwolf.settings.NSUserDefaultsSettings
import platform.Foundation.NSUserDefaults

actual fun createSettings(): Settings {
    return NSUserDefaultsSettings(NSUserDefaults.standardUserDefaults)
}

actual fun getPlatformName(): String = "iOS"

