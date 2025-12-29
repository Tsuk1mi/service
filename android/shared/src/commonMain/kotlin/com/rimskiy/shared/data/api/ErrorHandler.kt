package com.rimskiy.shared.data.api

import io.ktor.client.call.*
import io.ktor.client.statement.*
import io.ktor.http.*

/**
 * Обработчик ошибок API
 * Применяет SRP: отвечает только за обработку и форматирование ошибок
 */
object ErrorHandler {
    /**
     * Извлекает понятное сообщение об ошибке из HTTP ответа
     */
    suspend fun getErrorMessage(response: HttpResponse): String {
        return when (response.status) {
            HttpStatusCode.Unauthorized -> "Необходима авторизация"
            HttpStatusCode.Forbidden -> "Доступ запрещен"
            HttpStatusCode.NotFound -> "Ресурс не найден"
            HttpStatusCode.BadRequest -> {
                try {
                    val body = response.body<String>()
                    println("[ErrorHandler] BadRequest body: $body")
                    // Попытка извлечь сообщение из JSON ответа
                    if (body.contains("\"error\"") || body.contains("\"message\"")) {
                        val errorMsg = extractMessageFromJson(body)
                        if (errorMsg.isNotBlank()) {
                            errorMsg
                        } else if (body.contains("\"details\"")) {
                            extractDetailsFromJson(body) ?: "Неверный запрос"
                        } else {
                            "Неверный запрос"
                        }
                    } else {
                        "Неверный запрос"
                    }
                } catch (e: Exception) {
                    println("[ErrorHandler] Error parsing BadRequest response: ${e.message}")
                    "Неверный запрос"
                }
            }
            HttpStatusCode.InternalServerError -> "Ошибка сервера. Попробуйте позже"
            HttpStatusCode.ServiceUnavailable -> "Сервис временно недоступен"
            else -> {
                try {
                    val body = response.body<String>()
                    if (body.isNotBlank()) {
                        body
                    } else {
                        "Ошибка: ${response.status}"
                    }
                } catch (e: Exception) {
                    "Ошибка соединения"
                }
            }
        }
    }
    
    private fun extractMessageFromJson(json: String): String {
        // Простое извлечение сообщения из JSON
        val messagePattern = "\"(message|error)\"\\s*:\\s*\"([^\"]+)\"".toRegex()
        val match = messagePattern.find(json)
        return match?.groupValues?.get(2) ?: "Ошибка запроса"
    }
    
    private fun extractDetailsFromJson(json: String): String? {
        // Попытка извлечь details из JSON
        val detailsPattern = "\"details\"\\s*:\\s*\"([^\"]+)\"".toRegex()
        return detailsPattern.find(json)?.groupValues?.get(1)
    }
    
    /**
     * Обрабатывает исключение и возвращает понятное сообщение
     */
    fun getErrorMessage(exception: Throwable): String {
        val exceptionClass = exception::class.simpleName ?: ""
        val message = exception.message ?: ""
        
        println("[ErrorHandler] Exception class: $exceptionClass")
        println("[ErrorHandler] Exception message: $message")
        
        return when {
            exceptionClass.contains("Timeout", ignoreCase = true) || 
            message.contains("timeout", ignoreCase = true) -> 
                "Превышено время ожидания. Проверьте интернет-соединение"
            exceptionClass.contains("UnknownHost", ignoreCase = true) || 
            message.contains("Unable to resolve host", ignoreCase = true) ||
            message.contains("Failed to connect", ignoreCase = true) ||
            message.contains("Connection refused", ignoreCase = true) -> 
                "Не удалось подключиться к серверу. Проверьте интернет-соединение и адрес сервера"
            exceptionClass.contains("Connect", ignoreCase = true) || 
            message.contains("connect", ignoreCase = true) -> 
                "Ошибка подключения. Убедитесь, что сервер запущен и доступен"
            exceptionClass.contains("Serialization", ignoreCase = true) -> 
                "Ошибка обработки данных"
            message.isNotBlank() -> message
            else -> "Неизвестная ошибка: $exceptionClass"
        }
    }
}

