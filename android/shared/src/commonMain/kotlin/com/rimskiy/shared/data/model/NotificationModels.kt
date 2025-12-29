package com.rimskiy.shared.data.model

import kotlinx.serialization.Serializable

@Serializable
data class NotificationResponse(
    val id: String,
    val type: String? = null,
    val title: String? = null,
    val message: String? = null,
    val data: kotlinx.serialization.json.JsonObject? = null,
    val read: Boolean = false,
    val created_at: String? = null
)

