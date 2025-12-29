package com.rimskiy.shared.domain.repository

import com.rimskiy.shared.data.api.ApiClient
import com.rimskiy.shared.data.local.SettingsManager
import com.rimskiy.shared.data.model.*

class AuthRepository(
    private val apiClient: ApiClient,
    private val settingsManager: SettingsManager
) : IAuthRepository {
    
    override suspend fun startAuth(phone: String): Result<AuthStartResponse> {
        return try {
            println("[AuthRepository] startAuth called with phone: $phone")
            // Валидация телефона
            val normalizedPhone = com.rimskiy.shared.utils.PhoneUtils.normalizePhone(phone)
            println("[AuthRepository] Normalized phone: $normalizedPhone")
            
            if (!com.rimskiy.shared.utils.PhoneUtils.validatePhone(normalizedPhone)) {
                println("[AuthRepository] Phone validation failed")
                return Result.failure(Exception("Неверный формат номера телефона"))
            }
            
            println("[AuthRepository] Calling apiClient.authStart...")
            val response = apiClient.authStart(AuthStartRequest(normalizedPhone))
            println("[AuthRepository] Auth start successful")
            Result.success(response)
        } catch (e: Exception) {
            println("[AuthRepository] Error in startAuth: ${e.message}")
            e.printStackTrace()
            Result.failure(Exception(com.rimskiy.shared.data.api.ErrorHandler.getErrorMessage(e)))
        }
    }

    override suspend fun verifyAuth(phone: String, code: String): Result<AuthVerifyResponse> {
        return try {
            if (code.isBlank() || code.length < 4) {
                return Result.failure(Exception("Код должен содержать минимум 4 цифры"))
            }
            val normalizedPhone = com.rimskiy.shared.utils.PhoneUtils.normalizePhone(phone)
            val response = apiClient.authVerify(AuthVerifyRequest(normalizedPhone, code))
            settingsManager.authToken = "Bearer ${response.token}"
            settingsManager.userId = response.user_id
            Result.success(response)
        } catch (e: Exception) {
            Result.failure(Exception(com.rimskiy.shared.data.api.ErrorHandler.getErrorMessage(e)))
        }
    }

    override suspend fun refreshToken(): Result<RefreshTokenResponse> {
        return try {
            val currentToken = settingsManager.authToken
                ?: return Result.failure(Exception("Не авторизован"))
            
            // Убираем префикс "Bearer " если есть
            val tokenValue = if (currentToken.startsWith("Bearer ")) {
                currentToken.substring(7)
            } else {
                currentToken
            }
            
            val response = apiClient.refreshToken(RefreshTokenRequest(tokenValue))
            settingsManager.authToken = "Bearer ${response.token}"
            settingsManager.userId = response.user_id
            Result.success(response)
        } catch (e: Exception) {
            // Если refresh не удался, очищаем токен
            settingsManager.clearAuth()
            Result.failure(Exception(com.rimskiy.shared.data.api.ErrorHandler.getErrorMessage(e)))
        }
    }

    override suspend fun logout() {
        settingsManager.clearAuth()
    }

    override fun isAuthenticated(): Boolean {
        return settingsManager.isAuthenticated
    }
}

