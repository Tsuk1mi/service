package com.rimskiy.shared.platform

import com.russhwolf.settings.Settings

expect fun createSettings(): Settings
expect fun getPlatformName(): String

