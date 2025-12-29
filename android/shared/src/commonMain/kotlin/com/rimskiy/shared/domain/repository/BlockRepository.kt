package com.rimskiy.shared.domain.repository

import com.rimskiy.shared.data.api.ApiClient
import com.rimskiy.shared.data.local.SettingsManager
import com.rimskiy.shared.data.local.TokenManager
import com.rimskiy.shared.data.model.Block
import com.rimskiy.shared.data.model.BlockWithBlockerInfo
import com.rimskiy.shared.data.model.CheckBlockResponse
import com.rimskiy.shared.data.model.CreateBlockRequest

class BlockRepository(
    private val apiClient: ApiClient,
    private val settingsManager: SettingsManager,
    private val tokenManager: TokenManager
) : IBlockRepository {
    
    override suspend fun createBlock(blockedPlate: String, notifyOwner: Boolean): Result<Block> {
        return try {
            val token = tokenManager.getValidToken()
                ?: return Result.failure(Exception("Не авторизован"))
            
            // Валидация номера
            val normalizedPlate = com.rimskiy.shared.utils.PlateUtils.normalizePlate(blockedPlate)
            if (!com.rimskiy.shared.utils.PlateUtils.validatePlate(normalizedPlate)) {
                return Result.failure(Exception("Неверный формат номера автомобиля"))
            }
            
            val response = apiClient.createBlock(token, CreateBlockRequest(normalizedPlate, notifyOwner))
            Result.success(response)
        } catch (e: Exception) {
            Result.failure(Exception(com.rimskiy.shared.data.api.ErrorHandler.getErrorMessage(e)))
        }
    }

    override suspend fun getMyBlocks(): Result<List<Block>> {
        return try {
            val token = tokenManager.getValidToken()
                ?: return Result.failure(Exception("Не авторизован"))
            val response = apiClient.getMyBlocks(token)
            Result.success(response)
        } catch (e: Exception) {
            Result.failure(Exception(com.rimskiy.shared.data.api.ErrorHandler.getErrorMessage(e)))
        }
    }

    override suspend fun getBlocksForMyPlate(myPlate: String?): Result<List<BlockWithBlockerInfo>> {
        return try {
            val token = tokenManager.getValidToken()
                ?: return Result.failure(Exception("Не авторизован"))
            val response = apiClient.getBlocksForMyPlate(token, myPlate)
            Result.success(response)
        } catch (e: Exception) {
            Result.failure(Exception(com.rimskiy.shared.data.api.ErrorHandler.getErrorMessage(e)))
        }
    }

    override suspend fun deleteBlock(blockId: String): Result<Unit> {
        return try {
            val token = tokenManager.getValidToken()
                ?: return Result.failure(Exception("Не авторизован"))
            apiClient.deleteBlock(token, blockId)
            Result.success(Unit)
        } catch (e: Exception) {
            Result.failure(Exception(com.rimskiy.shared.data.api.ErrorHandler.getErrorMessage(e)))
        }
    }

    override suspend fun checkBlock(plate: String): Result<CheckBlockResponse> {
        return try {
            val token = tokenManager.getValidToken()
                ?: return Result.failure(Exception("Не авторизован"))
            val normalizedPlate = com.rimskiy.shared.utils.PlateUtils.normalizePlate(plate)
            val response = apiClient.checkBlock(token, normalizedPlate)
            Result.success(response)
        } catch (e: Exception) {
            Result.failure(Exception(com.rimskiy.shared.data.api.ErrorHandler.getErrorMessage(e)))
        }
    }

    override suspend fun warnOwner(blockId: String): Result<Unit> {
        return try {
            val token = tokenManager.getValidToken()
                ?: return Result.failure(Exception("Не авторизован"))
            apiClient.warnOwner(token, blockId)
            Result.success(Unit)
        } catch (e: Exception) {
            Result.failure(Exception(com.rimskiy.shared.data.api.ErrorHandler.getErrorMessage(e)))
        }
    }
}

