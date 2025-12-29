package com.rimskiy.shared.domain.repository

import com.rimskiy.shared.data.api.ApiClient
import com.rimskiy.shared.data.local.SettingsManager
import com.rimskiy.shared.data.local.TokenManager
import com.rimskiy.shared.data.model.PublicUserInfo
import com.rimskiy.shared.data.model.UpdateUserRequest
import com.rimskiy.shared.data.model.UserResponse

class UserRepository(
    private val apiClient: ApiClient,
    private val settingsManager: SettingsManager,
    private val tokenManager: TokenManager
) : IUserRepository {
    
    override suspend fun getProfile(): Result<UserResponse> {
        return try {
            val token = tokenManager.getValidToken()
                ?: return Result.failure(Exception("Не авторизован"))
            val response = apiClient.getProfile(token)
            Result.success(response)
        } catch (e: Exception) {
            Result.failure(Exception(com.rimskiy.shared.data.api.ErrorHandler.getErrorMessage(e)))
        }
    }

    override suspend fun updateProfile(request: UpdateUserRequest): Result<UserResponse> {
        return try {
            val token = tokenManager.getValidToken()
                ?: return Result.failure(Exception("Не авторизован"))
            
            // Валидация данных
            request.phone?.let { phone ->
                val normalized = com.rimskiy.shared.utils.PhoneUtils.normalizePhone(phone)
                if (!com.rimskiy.shared.utils.PhoneUtils.validatePhone(normalized)) {
                    return Result.failure(Exception("Неверный формат номера телефона"))
                }
            }
            
            request.plate?.let { plate ->
                val normalized = com.rimskiy.shared.utils.PlateUtils.normalizePlate(plate)
                if (!com.rimskiy.shared.utils.PlateUtils.validatePlate(normalized)) {
                    return Result.failure(Exception("Неверный формат номера автомобиля"))
                }
            }
            
            val response = apiClient.updateProfile(token, request)
            Result.success(response)
        } catch (e: Exception) {
            Result.failure(Exception(com.rimskiy.shared.data.api.ErrorHandler.getErrorMessage(e)))
        }
    }

    override suspend fun getUserByPlate(plate: String): Result<PublicUserInfo?> {
        return try {
            val token = tokenManager.getValidToken()
                ?: return Result.failure(Exception("Не авторизован"))
            
            // Валидация и нормализация номера
            val normalizedPlate = com.rimskiy.shared.utils.PlateUtils.normalizePlate(plate)
            if (!com.rimskiy.shared.utils.PlateUtils.validatePlate(normalizedPlate)) {
                return Result.failure(Exception("Неверный формат номера автомобиля"))
            }
            
            val response = apiClient.getUserByPlate(token, normalizedPlate)
            Result.success(response)
        } catch (e: Exception) {
            Result.failure(Exception(com.rimskiy.shared.data.api.ErrorHandler.getErrorMessage(e)))
        }
    }
}

