package com.rimskiy.shared.di

import com.rimskiy.shared.platform.IosActions
import com.rimskiy.shared.platform.PlatformActions

actual fun getPlatformActions(): PlatformActions {
    return IosActions()
}

