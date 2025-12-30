package com.rimskiy.shared.data.model

import kotlinx.serialization.Serializable

@Serializable
data class ServerInfoResponse(
    val server_url: String? = null,
    val port: Int? = null,
    val server_version: String? = null,
    val min_client_version: String? = null,
    val release_client_version: String? = null,
    val app_download_url: String? = null
    val telegram_bot_username: String? = null
)

