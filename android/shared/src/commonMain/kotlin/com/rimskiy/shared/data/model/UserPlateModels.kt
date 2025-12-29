package com.rimskiy.shared.data.model

import kotlinx.serialization.Serializable

@Serializable
data class UserPlateResponse(
    val id: String,
    val user_id: String,
    val plate: String,
    val is_primary: Boolean = false,
    val created_at: String? = null,
    val updated_at: String? = null
)

@Serializable
data class CreateUserPlateRequest(
    val plate: String,
    val is_primary: Boolean = false
)

