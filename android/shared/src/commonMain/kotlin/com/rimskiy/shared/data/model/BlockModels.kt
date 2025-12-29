package com.rimskiy.shared.data.model

import kotlinx.serialization.Serializable

@Serializable
data class Block(
    val id: String,
    val blocker_id: String,
    val blocked_plate: String,
    val created_at: String
)

@Serializable
data class CreateBlockRequest(
    val blocked_plate: String,
    val notify_owner: Boolean = false
)

@Serializable
data class BlockWithBlockerInfo(
    val id: String,
    val blocked_plate: String,
    val created_at: String,
    val blocker: PublicUserInfo,
    val blocker_owner_type: String? = null,
    val blocker_owner_info: kotlinx.serialization.json.JsonObject? = null
)

@Serializable
data class CheckBlockResponse(
    val is_blocked: Boolean,
    val block: BlockWithBlockerInfo? = null
)

