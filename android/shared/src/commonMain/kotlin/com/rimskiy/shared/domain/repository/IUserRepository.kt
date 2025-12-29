package com.rimskiy.shared.domain.repository

import com.rimskiy.shared.data.model.PublicUserInfo
import com.rimskiy.shared.data.model.UpdateUserRequest
import com.rimskiy.shared.data.model.UserResponse

interface IUserRepository {
    suspend fun getProfile(): Result<UserResponse>
    suspend fun updateProfile(request: UpdateUserRequest): Result<UserResponse>
    suspend fun getUserByPlate(plate: String): Result<PublicUserInfo?>
}

