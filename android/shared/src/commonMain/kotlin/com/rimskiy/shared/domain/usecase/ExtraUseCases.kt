package com.rimskiy.shared.domain.usecase

import com.rimskiy.shared.data.api.ApiClient
import com.rimskiy.shared.data.api.RecognizePlateResponse
import com.rimskiy.shared.data.local.SettingsManager
import com.rimskiy.shared.data.model.*

class RecognizePlateUseCase(
    private val apiClient: ApiClient,
    private val settingsManager: SettingsManager
) {
    suspend operator fun invoke(imageBytes: ByteArray): Result<RecognizePlateResponse> {
        return try {
            val token = settingsManager.authToken ?: return Result.failure(Exception("Не авторизован"))
            val response = apiClient.recognizePlateFromImage(token, imageBytes)
            Result.success(response)
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
}

class GetNotificationsUseCase(
    private val apiClient: ApiClient,
    private val settingsManager: SettingsManager
) {
    suspend operator fun invoke(unreadOnly: Boolean = false): Result<List<NotificationResponse>> {
        return try {
            val token = settingsManager.authToken ?: return Result.failure(Exception("Не авторизован"))
            Result.success(apiClient.getNotifications(token, unreadOnly))
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
}

class MarkNotificationReadUseCase(
    private val apiClient: ApiClient,
    private val settingsManager: SettingsManager
) {
    suspend operator fun invoke(notificationId: String): Result<Unit> {
        return try {
            val token = settingsManager.authToken ?: return Result.failure(Exception("Не авторизован"))
            apiClient.markNotificationRead(token, notificationId)
            Result.success(Unit)
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
}

class MarkAllNotificationsReadUseCase(
    private val apiClient: ApiClient,
    private val settingsManager: SettingsManager
) {
    suspend operator fun invoke(): Result<Unit> {
        return try {
            val token = settingsManager.authToken ?: return Result.failure(Exception("Не авторизован"))
            apiClient.markAllNotificationsRead(token)
            Result.success(Unit)
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
}

class GetUserPlatesUseCase(
    private val apiClient: ApiClient,
    private val settingsManager: SettingsManager
) {
    suspend operator fun invoke(): Result<List<UserPlateResponse>> {
        return try {
            val token = settingsManager.authToken ?: return Result.failure(Exception("Не авторизован"))
            Result.success(apiClient.getUserPlates(token))
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
}

class CreateUserPlateUseCase(
    private val apiClient: ApiClient,
    private val settingsManager: SettingsManager
) {
    suspend operator fun invoke(request: CreateUserPlateRequest): Result<UserPlateResponse> {
        return try {
            val token = settingsManager.authToken ?: return Result.failure(Exception("Не авторизован"))
            Result.success(apiClient.createUserPlate(token, request))
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
}

class DeleteUserPlateUseCase(
    private val apiClient: ApiClient,
    private val settingsManager: SettingsManager
) {
    suspend operator fun invoke(plateId: String): Result<Unit> {
        return try {
            val token = settingsManager.authToken ?: return Result.failure(Exception("Не авторизован"))
            apiClient.deleteUserPlate(token, plateId)
            Result.success(Unit)
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
}

class SetPrimaryPlateUseCase(
    private val apiClient: ApiClient,
    private val settingsManager: SettingsManager
) {
    suspend operator fun invoke(plateId: String): Result<Unit> {
        return try {
            val token = settingsManager.authToken ?: return Result.failure(Exception("Не авторизован"))
            apiClient.setPrimaryPlate(token, plateId)
            Result.success(Unit)
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
}

