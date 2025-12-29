package com.rimskiy.shared.domain.usecase

import com.rimskiy.shared.data.model.AuthStartResponse
import com.rimskiy.shared.data.model.AuthVerifyResponse
import com.rimskiy.shared.domain.repository.IAuthRepository

class StartAuthUseCase(
    private val authRepository: IAuthRepository
) {
    suspend operator fun invoke(phone: String): Result<AuthStartResponse> {
        return authRepository.startAuth(phone)
    }
}

class VerifyAuthUseCase(
    private val authRepository: IAuthRepository
) {
    suspend operator fun invoke(phone: String, code: String): Result<AuthVerifyResponse> {
        return authRepository.verifyAuth(phone, code)
    }
}

class LogoutUseCase(
    private val authRepository: IAuthRepository
) {
    suspend operator fun invoke() {
        authRepository.logout()
    }
}

class IsAuthenticatedUseCase(
    private val authRepository: IAuthRepository
) {
    operator fun invoke(): Boolean {
        return authRepository.isAuthenticated()
    }
}

