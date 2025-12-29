package com.rimskiy.shared.domain.usecase

import com.rimskiy.shared.data.model.PublicUserInfo
import com.rimskiy.shared.data.model.UpdateUserRequest
import com.rimskiy.shared.data.model.UserResponse
import com.rimskiy.shared.domain.repository.IUserRepository

class GetProfileUseCase(
    private val userRepository: IUserRepository
) {
    suspend operator fun invoke(): Result<UserResponse> {
        return userRepository.getProfile()
    }
}

class UpdateProfileUseCase(
    private val userRepository: IUserRepository
) {
    suspend operator fun invoke(request: UpdateUserRequest): Result<UserResponse> {
        return userRepository.updateProfile(request)
    }
}

class GetUserByPlateUseCase(
    private val userRepository: IUserRepository
) {
    suspend operator fun invoke(plate: String): Result<PublicUserInfo?> {
        return userRepository.getUserByPlate(plate)
    }
}

