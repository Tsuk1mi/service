package com.rimskiy.shared.data.local

/**
 * Минимальный менеджер токена: берет Bearer-токен из настроек.
 * При необходимости можно расширить логикой refresh.
 */
class TokenManager(
    private val settingsManager: SettingsManager
) {
    fun getValidToken(): String? = settingsManager.authToken
    fun clear() = settingsManager.clearAuth()
}

