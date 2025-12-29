package com.rimskiy.shared.data.api

import com.rimskiy.shared.data.model.*
import com.rimskiy.shared.utils.Base64Utils
import com.rimskiy.shared.utils.createHttpClient
import io.ktor.client.*
import io.ktor.client.call.*
import io.ktor.client.plugins.contentnegotiation.*
import io.ktor.client.plugins.logging.*
import io.ktor.client.plugins.HttpTimeout
import io.ktor.client.request.*
import io.ktor.client.request.forms.*
import io.ktor.client.statement.*
import io.ktor.http.*
import io.ktor.serialization.kotlinx.json.*
import kotlinx.serialization.json.Json

/**
 * API клиент для взаимодействия с backend сервером Rimskiy Service
 * 
 * Предоставляет методы для:
 * - Аутентификации (SMS код)
 * - Управления профилем пользователя
 * - Создания и управления блокировками автомобилей
 * - Работы с уведомлениями
 * 
 * Применяет SOLID принципы:
 * - Single Responsibility: только HTTP запросы
 * - Dependency Inversion: использует HttpClient (интерфейс)
 * 
 * @property baseUrl Базовый URL API сервера
 * @property httpClient HTTP клиент (Ktor)
 * @property ddnsAuthHeader Опциональный заголовок Basic Auth для DDNS
 */
class ApiClient(
    private val baseUrl: String, 
    private val httpClient: HttpClient,
    private val ddnsAuthHeader: String? = null
) {
    
    companion object {
        /**
         * Фабричный метод для создания ApiClient
         * Настраивает HTTP клиент с JSON сериализацией
         */
        fun create(baseUrl: String, ddnsUsername: String? = null, ddnsPassword: String? = null): ApiClient {
            println("[ApiClient] Creating client with baseUrl: $baseUrl")
            println("[ApiClient] DDNS credentials provided: ${ddnsUsername != null && ddnsPassword != null}")
            
            // ВАЖНО: Ktor на Android может некорректно обрабатывать URL с встроенными учетными данными.
            // Используем чистый URL без учетных данных в userinfo части.
            // Аутентификацию DDNS будем выполнять только через заголовки HTTP.
            val finalBaseUrl = try {
                // Нормализуем URL: добавляем протокол, если его нет
                val normalizedBaseUrl = if (!baseUrl.contains("://")) {
                    // Если нет протокола, добавляем http://
                    "http://$baseUrl".also {
                        println("[ApiClient] Added missing protocol (http://) to URL: $it")
                    }
                } else {
                    baseUrl
                }
                
                // Очищаем URL от возможных учетных данных в userinfo
                val url = java.net.URL(normalizedBaseUrl)
                val protocol = url.protocol
                val host = url.host
                val port = if (url.port != -1 && url.port != url.defaultPort) ":${url.port}" else ""
                val path = url.path ?: ""
                val query = url.query?.let { "?$it" } ?: ""
                val fragment = url.ref?.let { "#$it" } ?: ""
                
                val cleanUrl = "$protocol://$host$port$path$query$fragment"
                println("[ApiClient] Clean base URL: $cleanUrl")
                cleanUrl
            } catch (e: Exception) {
                println("[ApiClient] Error parsing URL: ${e.message}")
                println("[ApiClient] Original URL: $baseUrl")
                e.printStackTrace()
                
                // Попытка исправить URL, добавив протокол
                val fallbackUrl = if (!baseUrl.contains("://")) {
                    "http://$baseUrl"
                } else {
                    baseUrl
                }
                
                println("[ApiClient] Using fallback URL: $fallbackUrl")
                fallbackUrl
            }
            
            // Используем фабрику HttpClient, которая создает клиент с отключенной проверкой SSL на Android
            val client = createHttpClient()
            println("[ApiClient] HttpClient created with SSL verification disabled")
            
            // Формируем заголовок авторизации для DDNS (Basic Auth)
            // Используем только заголовки, не встраиваем в URL
            val ddnsAuthHeader = if (ddnsUsername != null && ddnsPassword != null) {
                val credentials = "${ddnsUsername}:${ddnsPassword}"
                val encoded = Base64Utils.encode(credentials.toByteArray(Charsets.UTF_8))
                val header = "Basic $encoded"
                println("[ApiClient] DDNS Basic Auth header prepared")
                println("[ApiClient] DDNS username: $ddnsUsername")
                header
            } else {
                null
            }
            
            return ApiClient(finalBaseUrl, client, ddnsAuthHeader)
        }
    }

    // Helper method для добавления DDNS заголовка авторизации
    // Стратегия:
    // 1. Для публичных эндпоинтов (без Bearer токена) используем Authorization с Basic Auth для DDNS
    // 2. Для защищенных эндпоинтов (с Bearer токеном) добавляем Proxy-Authorization с Basic Auth
    //    для DDNS прокси, чтобы не конфликтовать с Bearer токеном в Authorization
    private fun HttpRequestBuilder.addDdnsAuthHeader() {
        ddnsAuthHeader?.let {
            val hasBearerToken = headers.contains("Authorization") && 
                                headers["Authorization"]?.startsWith("Bearer") == true
            
            if (hasBearerToken) {
                // Если есть Bearer токен, используем Proxy-Authorization для DDNS
                headers.append("Proxy-Authorization", it)
                println("[ApiClient] Added DDNS Basic Auth header (Proxy-Authorization) for proxy authentication")
            } else {
                // Для публичных эндпоинтов используем Authorization с Basic Auth
                headers.append("Authorization", it)
                println("[ApiClient] Added DDNS Basic Auth header (Authorization) to request")
            }
        }
    }

    // Helper method для retry логики с экспоненциальной задержкой
    private suspend fun <T> retryWithBackoff(
        maxRetries: Int = 3,
        initialDelayMs: Long = 500,
        block: suspend () -> T
    ): T {
        var lastException: Exception? = null
        var delay = initialDelayMs
        
        repeat(maxRetries) { attempt ->
            try {
                return block()
            } catch (e: Exception) {
                val exception = e as? Exception ?: Exception(e.message ?: "Unknown error")
                lastException = exception
                
                // Не повторяем для ошибок клиента (4xx), кроме временных
                if (e.message?.contains("401", ignoreCase = true) == true ||
                    e.message?.contains("403", ignoreCase = true) == true ||
                    e.message?.contains("404", ignoreCase = true) == true ||
                    e.message?.contains("400", ignoreCase = true) == true) {
                    throw exception
                }
                
                // Если это последняя попытка, выбрасываем исключение
                if (attempt == maxRetries - 1) {
                    throw exception
                }
                
                // Для сетевых ошибок делаем retry
                val shouldRetry = e is java.net.ConnectException ||
                                 e is java.net.SocketTimeoutException ||
                                 e.message?.contains("timeout", ignoreCase = true) == true ||
                                 e.message?.contains("connect", ignoreCase = true) == true ||
                                 e.message?.contains("network", ignoreCase = true) == true
                
                if (shouldRetry) {
                    println("[ApiClient] Retry attempt ${attempt + 1}/$maxRetries after ${delay}ms delay")
                    kotlinx.coroutines.delay(delay)
                    delay *= 2 // Экспоненциальная задержка
                } else {
                    throw exception
                }
            }
        }
        
        throw lastException ?: Exception("Retry failed")
    }

    // ==================== Auth Endpoints ====================
    
    /**
     * Начало процесса авторизации - отправка SMS кода на телефон
     * 
     * @param request Запрос с номером телефона
     * @return Ответ с SMS кодом и временем жизни
     * @throws Exception При ошибках сети или валидации
     */
    suspend fun authStart(request: AuthStartRequest): AuthStartResponse {
        val url = "$baseUrl/api/auth/start"
        println("[API] ========================================")
        println("[API] POST $url")
        println("[API] Base URL: $baseUrl")
        println("[API] Request body: phone=${request.phone}")
        println("[API] DDNS auth header present: ${ddnsAuthHeader != null}")
        
        return try {
            val response = httpClient.post(url) {
                contentType(ContentType.Application.Json)
                setBody(request)
                addDdnsAuthHeader()
                // Добавляем дополнительные заголовки для диагностики
                header("User-Agent", "Rimskiy-Android-Client/1.0")
                header("Accept", "application/json")
            }
            println("[API] Response status: ${response.status.value} ${response.status.description}")
            println("[API] Response headers: ${response.headers}")
            
            if (!response.status.isSuccess()) {
                val errorMessage = ErrorHandler.getErrorMessage(response)
                println("[API] Error response: $errorMessage")
                throw Exception(errorMessage)
            }
            
            val body = response.body<AuthStartResponse>()
            println("[API] Response body: $body")
            println("[API] ========================================")
            body
        } catch (e: java.net.ConnectException) {
            println("[API] ========================================")
            println("[API] CONNECTION ERROR - Failed to connect to server")
            println("[API] URL: $url")
            println("[API] Base URL: $baseUrl")
            println("[API] Error: ${e.message}")
            println("[API] This might indicate:")
            println("[API]   1. Server is not running or not accessible")
            println("[API]   2. Incorrect URL or port")
            println("[API]   3. Firewall or network restrictions")
            println("[API]   4. DDNS service is not properly configured")
            println("[API] ========================================")
            e.printStackTrace()
            throw Exception("Не удалось подключиться к серверу. Проверьте подключение к интернету и правильность адреса сервера. ${e.message}", e)
        } catch (e: java.net.SocketTimeoutException) {
            println("[API] ========================================")
            println("[API] TIMEOUT ERROR - Request timed out")
            println("[API] URL: $url")
            println("[API] Error: ${e.message}")
            println("[API] ========================================")
            e.printStackTrace()
            throw Exception("Превышено время ожидания подключения. Сервер не отвечает. ${e.message}", e)
        } catch (e: Exception) {
            println("[API] ========================================")
            println("[API] Exception during request: ${e.message}")
            println("[API] Exception type: ${e::class.simpleName}")
            println("[API] URL: $url")
            println("[API] ========================================")
            e.printStackTrace()
            throw e
        }
    }

    /**
     * Подтверждение авторизации - проверка SMS кода
     * 
     * @param request Запрос с номером телефона и SMS кодом
     * @return Ответ с JWT токеном и ID пользователя
     * @throws Exception При неверном коде или ошибках сети
     */
    suspend fun authVerify(request: AuthVerifyRequest): AuthVerifyResponse {
        val url = "$baseUrl/api/auth/verify"
        println("[API] POST $url")
        val response = httpClient.post(url) {
            contentType(ContentType.Application.Json)
            setBody(request)
            addDdnsAuthHeader()
        }
        println("[API] Response: ${response.status} for POST $url")
        if (!response.status.isSuccess()) {
            throw Exception(ErrorHandler.getErrorMessage(response))
        }
        return response.body()
    }

    /**
     * Обновление JWT токена
     * 
     * @param request Запрос с текущим токеном
     * @return Ответ с новым токеном
     * @throws Exception При неверном или истекшем токене
     */
    suspend fun refreshToken(request: RefreshTokenRequest): RefreshTokenResponse {
        val url = "$baseUrl/api/auth/refresh"
        println("[API] POST $url")
        val response = httpClient.post(url) {
            contentType(ContentType.Application.Json)
            setBody(request)
            addDdnsAuthHeader()
        }
        println("[API] Response: ${response.status} for POST $url")
        if (!response.status.isSuccess()) {
            throw Exception(ErrorHandler.getErrorMessage(response))
        }
        return response.body()
    }

    // ==================== User Endpoints ====================
    
    /**
     * Получить профиль текущего пользователя
     * 
     * @param token JWT токен (Bearer token)
     * @return Профиль пользователя
     * @throws Exception При ошибках авторизации или сети
     */
    suspend fun getProfile(token: String): UserResponse {
        return retryWithBackoff {
            val response = httpClient.get("$baseUrl/api/users/me") {
                header("Authorization", token)
                header("Cache-Control", "no-cache")
                addDdnsAuthHeader()
            }
            if (!response.status.isSuccess()) {
                val errorMsg = ErrorHandler.getErrorMessage(response)
                println("[API] getProfile failed: ${response.status} - $errorMsg")
                throw Exception(errorMsg)
            }
            response.body()
        }
    }

    suspend fun getUserByPlate(token: String, plate: String): PublicUserInfo? {
        val response = httpClient.get("$baseUrl/api/users/by-plate") {
            header("Authorization", token)
            parameter("plate", plate)
            addDdnsAuthHeader()
        }
        if (!response.status.isSuccess()) {
            // Если пользователь не найден (404), возвращаем null
            if (response.status.value == 404) {
                return null
            }
            throw Exception(ErrorHandler.getErrorMessage(response))
        }
        return response.body()
    }

    /**
     * Обновить профиль текущего пользователя
     * 
     * @param token JWT токен (Bearer token)
     * @param request Данные для обновления
     * @return Обновленный профиль пользователя
     * @throws Exception При ошибках валидации, авторизации или сети
     */
    suspend fun updateProfile(token: String, request: UpdateUserRequest): UserResponse {
        return retryWithBackoff {
            val response = httpClient.put("$baseUrl/api/users/me") {
                contentType(ContentType.Application.Json)
                header("Authorization", token)
                setBody(request)
                addDdnsAuthHeader()
            }
            if (!response.status.isSuccess()) {
                val errorMsg = ErrorHandler.getErrorMessage(response)
                println("[API] updateProfile failed: ${response.status} - $errorMsg")
                throw Exception(errorMsg)
            }
            response.body()
        }
    }

    // ==================== Block Endpoints ====================
    
    /**
     * Создать блокировку автомобиля
     * 
     * @param token JWT токен (Bearer token)
     * @param request Данные блокировки (номер авто, уведомить ли владельца)
     * @return Созданная блокировка
     * @throws Exception При ошибках валидации, авторизации или сети
     */
    suspend fun createBlock(token: String, request: CreateBlockRequest): Block {
        val response = httpClient.post("$baseUrl/api/blocks") {
            contentType(ContentType.Application.Json)
            header("Authorization", token)
            setBody(request)
            addDdnsAuthHeader()
        }
        if (!response.status.isSuccess()) {
            throw Exception(ErrorHandler.getErrorMessage(response))
        }
        return response.body()
    }

    suspend fun getMyBlocks(token: String): List<Block> {
        val response = httpClient.get("$baseUrl/api/blocks") {
            header("Authorization", token)
            header("Cache-Control", "no-cache")
            addDdnsAuthHeader()
        }
        if (!response.status.isSuccess()) {
            throw Exception(ErrorHandler.getErrorMessage(response))
        }
        return response.body()
    }

    suspend fun getBlocksForMyPlate(token: String, myPlate: String? = null): List<BlockWithBlockerInfo> {
        val response = httpClient.get("$baseUrl/api/blocks/my") {
            header("Authorization", token)
            header("Cache-Control", "no-cache")
            myPlate?.let { parameter("my_plate", it) }
            addDdnsAuthHeader()
        }
        if (!response.status.isSuccess()) {
            throw Exception(ErrorHandler.getErrorMessage(response))
        }
        return response.body()
    }

    suspend fun deleteBlock(token: String, blockId: String) {
        val response = httpClient.delete("$baseUrl/api/blocks/$blockId") {
            header("Authorization", token)
            addDdnsAuthHeader()
        }
        if (!response.status.isSuccess()) {
            throw Exception(ErrorHandler.getErrorMessage(response))
        }
    }

    suspend fun warnOwner(token: String, blockId: String) {
        val response = httpClient.post("$baseUrl/api/blocks/$blockId/warn-owner") {
            header("Authorization", token)
            addDdnsAuthHeader()
        }
        if (!response.status.isSuccess()) {
            throw Exception(ErrorHandler.getErrorMessage(response))
        }
    }

    suspend fun checkBlock(token: String, plate: String): CheckBlockResponse {
        val response = httpClient.get("$baseUrl/api/blocks/check") {
            header("Authorization", token)
            parameter("plate", plate)
            addDdnsAuthHeader()
        }
        if (!response.status.isSuccess()) {
            throw Exception(ErrorHandler.getErrorMessage(response))
        }
        return response.body()
    }

    // OCR endpoint для распознавания номера с фото
    suspend fun recognizePlateFromImage(token: String, imageData: ByteArray): RecognizePlateResponse {
        val response = httpClient.post("$baseUrl/api/ocr/recognize-plate-auth") {
            header("Authorization", token)
            addDdnsAuthHeader()
            setBody(
                MultiPartFormDataContent(
                    formData {
                        append("image", imageData, Headers.build {
                            append(HttpHeaders.ContentType, "image/jpeg")
                            append(HttpHeaders.ContentDisposition, "form-data; name=\"image\"; filename=\"plate.jpg\"")
                        })
                    }
                )
            )
        }
        if (!response.status.isSuccess()) {
            throw Exception(ErrorHandler.getErrorMessage(response))
        }
        return response.body()
    }

    // Notification endpoints
    suspend fun getNotifications(token: String, unreadOnly: Boolean = false): List<NotificationResponse> {
        val response = httpClient.get("$baseUrl/api/notifications") {
            header("Authorization", token)
            header("Cache-Control", "no-cache")
            parameter("unread_only", unreadOnly)
            addDdnsAuthHeader()
        }
        if (!response.status.isSuccess()) {
            throw Exception(ErrorHandler.getErrorMessage(response))
        }
        return response.body()
    }

    suspend fun markNotificationRead(token: String, notificationId: String) {
        val response = httpClient.patch("$baseUrl/api/notifications/$notificationId/read") {
            header("Authorization", token)
            addDdnsAuthHeader()
        }
        if (!response.status.isSuccess()) {
            throw Exception(ErrorHandler.getErrorMessage(response))
        }
    }

    suspend fun markAllNotificationsRead(token: String) {
        val response = httpClient.patch("$baseUrl/api/notifications/read-all") {
            header("Authorization", token)
            addDdnsAuthHeader()
        }
        if (!response.status.isSuccess()) {
            throw Exception(ErrorHandler.getErrorMessage(response))
        }
    }

    // User Plate endpoints
    suspend fun getUserPlates(token: String): List<com.rimskiy.shared.data.model.UserPlateResponse> {
        val response = httpClient.get("$baseUrl/api/user/plates") {
            header("Authorization", token)
            header("Cache-Control", "no-cache")
            addDdnsAuthHeader()
        }
        if (!response.status.isSuccess()) {
            throw Exception(ErrorHandler.getErrorMessage(response))
        }
        return response.body()
    }

    suspend fun createUserPlate(token: String, request: com.rimskiy.shared.data.model.CreateUserPlateRequest): com.rimskiy.shared.data.model.UserPlateResponse {
        val response = httpClient.post("$baseUrl/api/user/plates") {
            contentType(ContentType.Application.Json)
            header("Authorization", token)
            setBody(request)
            addDdnsAuthHeader()
        }
        if (!response.status.isSuccess()) {
            throw Exception(ErrorHandler.getErrorMessage(response))
        }
        return response.body()
    }

    suspend fun deleteUserPlate(token: String, plateId: String) {
        val response = httpClient.delete("$baseUrl/api/user/plates/$plateId") {
            header("Authorization", token)
            addDdnsAuthHeader()
        }
        if (!response.status.isSuccess()) {
            throw Exception(ErrorHandler.getErrorMessage(response))
        }
    }

    suspend fun setPrimaryPlate(token: String, plateId: String) {
        val response = httpClient.post("$baseUrl/api/user/plates/$plateId/primary") {
            header("Authorization", token)
            addDdnsAuthHeader()
        }
        if (!response.status.isSuccess()) {
            throw Exception(ErrorHandler.getErrorMessage(response))
        }
    }
}

data class RecognizePlateResponse(
    val success: Boolean,
    val plate: String? = null,
    val error: String? = null
)
