package com.rimskiy.shared.utils

import io.ktor.client.HttpClient
import io.ktor.client.engine.android.Android
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.plugins.logging.LogLevel
import io.ktor.client.plugins.logging.Logger
import io.ktor.client.plugins.logging.Logging
import io.ktor.client.plugins.HttpTimeout
import io.ktor.serialization.kotlinx.json.json
import kotlinx.serialization.json.Json
import java.util.Base64

object Base64Utils {
    fun encode(data: ByteArray): String = Base64.getEncoder().encodeToString(data)
    fun decode(str: String): ByteArray = Base64.getDecoder().decode(str)
}

/**
 * Создает HttpClient для Android со стандартной JSON-serialization, логированием и таймаутами.
 */
fun createHttpClient(): HttpClient = HttpClient(Android) {
    install(ContentNegotiation) {
        json(
            Json {
                ignoreUnknownKeys = true
                isLenient = true
            }
        )
    }
    install(Logging) {
        level = LogLevel.INFO
        logger = object : Logger {
            override fun log(message: String) {
                println(message)
            }
        }
    }
    install(HttpTimeout) {
        requestTimeoutMillis = 30_000
        connectTimeoutMillis = 30_000
        socketTimeoutMillis = 30_000
    }
}

