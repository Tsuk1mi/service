package com.rimskiy.shared.data.model

import kotlinx.serialization.Serializable

@Serializable
data class AuthStartRequest(
    val phone: String
)

@Serializable
data class AuthStartResponse(
    val code: String,
    val expires_in: Long
)

@Serializable
data class AuthVerifyRequest(
    val phone: String,
    val code: String
)

@Serializable
data class AuthVerifyResponse(
    val token: String,
    val user_id: String
)

@Serializable
data class RefreshTokenRequest(
    val token: String
)

@Serializable
data class RefreshTokenResponse(
    val token: String,
    val user_id: String
)

