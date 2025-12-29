package com.rimskiy.shared.data.model

import kotlinx.serialization.Serializable

@Serializable
data class UserResponse(
    val id: String,
    val name: String? = null,
    val phone: String? = null,
    val telegram: String? = null,
    val plate: String,
    val show_contacts: Boolean,
    val owner_type: String? = null,
    val owner_info: kotlinx.serialization.json.JsonObject? = null,
    val departure_time: String? = null,
    val created_at: String
)

@Serializable
data class UpdateUserRequest(
    val name: String? = null,
    val phone: String? = null,
    val telegram: String? = null,
    val plate: String? = null,
    val show_contacts: Boolean? = null,
    val owner_type: String? = null,
    val owner_info: kotlinx.serialization.json.JsonObject? = null,
    val departure_time: String? = null
)

@Serializable
data class PublicUserInfo(
    val id: String,
    val name: String? = null,
    val plate: String,
    val phone: String? = null,
    val telegram: String? = null,
    val departure_time: String? = null
)

