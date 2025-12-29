package com.rimskiy.shared.domain.repository

import com.rimskiy.shared.data.model.AuthStartResponse
import com.rimskiy.shared.data.model.AuthVerifyResponse
import com.rimskiy.shared.data.model.RefreshTokenResponse

interface IAuthRepository {
    suspend fun startAuth(phone: String): Result<AuthStartResponse>
    suspend fun verifyAuth(phone: String, code: String): Result<AuthVerifyResponse>
    suspend fun refreshToken(): Result<RefreshTokenResponse>
    suspend fun logout()
    fun isAuthenticated(): Boolean
}

