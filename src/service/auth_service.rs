use crate::auth::jwt::create_token;
use crate::auth::sms::SmsService;
use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::models::auth::{AuthStartResponse, AuthVerifyResponse, RefreshTokenResponse};
use crate::repository::{CreateUserData, UserPlateRepository, UserRepository};
use crate::service::validation_service::ValidationService;
use crate::utils::encryption::Encryption;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(serde::Deserialize)]
struct TelegramBotSendCodeResponse {
    success: bool,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Clone, Debug)]
struct GatewayRequest {
    request_id: String,
    expires_at: DateTime<Utc>,
}

/// Сервис аутентификации и авторизации пользователей
#[derive(Clone)]
pub struct AuthService {
    sms_service: SmsService,
    encryption: Encryption,
    config: Config,
    http_client: Client,
    gateway_requests: Arc<RwLock<HashMap<String, GatewayRequest>>>,
}

impl AuthService {
    pub fn new(sms_service: SmsService, encryption: Encryption, config: Config) -> Self {
        Self {
            sms_service,
            encryption,
            config: config.clone(),
            http_client: Client::new(),
            gateway_requests: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn telegram_gateway_base_url(&self) -> String {
        self.config
            .telegram_gateway_base_url
            .clone()
            .unwrap_or_else(|| "https://gatewayapi.telegram.org/".to_string())
    }

    async fn send_code_via_telegram_gateway(&self, phone: &str) -> Result<String, String> {
        let token = self
            .config
            .telegram_gateway_token
            .as_ref()
            .ok_or_else(|| "TELEGRAM_GATEWAY_TOKEN not configured".to_string())?;

        let url = format!(
            "{}sendVerificationMessage",
            self.telegram_gateway_base_url()
        );
        let code_length = self.config.sms_code_length.clamp(4, 8);

        let resp = self
            .http_client
            .post(url)
            .bearer_auth(token)
            .json(&json!({
                "phone_number": phone,
                "code_length": code_length
            }))
            .send()
            .await
            .map_err(|e| format!("Telegram Gateway request failed: {}", e))?;

        let status = resp.status();
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Telegram Gateway response parse failed: {}", e))?;

        if !status.is_success() {
            return Err(format!("Telegram Gateway HTTP error {}: {}", status, body));
        }

        if body.get("ok").and_then(|v| v.as_bool()) != Some(true) {
            return Err(body
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Telegram Gateway returned ok=false")
                .to_string());
        }

        let request_id = body
            .get("result")
            .and_then(|r| r.get("request_id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("Telegram Gateway missing request_id: {}", body))?;

        Ok(request_id.to_string())
    }

    async fn verify_code_via_telegram_gateway(
        &self,
        request_id: &str,
        code: &str,
    ) -> Result<bool, String> {
        let token = self
            .config
            .telegram_gateway_token
            .as_ref()
            .ok_or_else(|| "TELEGRAM_GATEWAY_TOKEN not configured".to_string())?;

        let url = format!(
            "{}checkVerificationStatus",
            self.telegram_gateway_base_url()
        );

        let resp = self
            .http_client
            .post(url)
            .bearer_auth(token)
            .json(&json!({
                "request_id": request_id,
                "code": code
            }))
            .send()
            .await
            .map_err(|e| format!("Telegram Gateway request failed: {}", e))?;

        let status = resp.status();
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Telegram Gateway response parse failed: {}", e))?;

        if !status.is_success() {
            return Err(format!("Telegram Gateway HTTP error {}: {}", status, body));
        }

        if body.get("ok").and_then(|v| v.as_bool()) != Some(true) {
            return Err(body
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Telegram Gateway returned ok=false")
                .to_string());
        }

        let status_str = body
            .get("result")
            .and_then(|r| r.get("verification_status"))
            .and_then(|vs| vs.get("status"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        Ok(status_str == "CODE_VALID")
    }

    /// Отправляет код авторизации в Telegram бот
    async fn send_code_to_telegram(&self, phone: &str, code: &str) -> Result<(), String> {
        // Получаем порт бота (основной порт + 1)
        let bot_port = self
            .config
            .server_port
            .checked_add(1)
            .ok_or_else(|| "Port overflow".to_string())?;

        let bot_url = format!("http://localhost:{}/send_code", bot_port);

        let payload = json!({
            "phone": phone,
            "code": code
        });

        match self.http_client.post(&bot_url).json(&payload).send().await {
            Ok(response) => {
                if !response.status().is_success() {
                    return Err(format!(
                        "Telegram bot returned error: {}",
                        response.status()
                    ));
                }

                let body: TelegramBotSendCodeResponse = response
                    .json()
                    .await
                    .map_err(|e| format!("Failed to parse Telegram bot response: {}", e))?;

                if body.success {
                    tracing::info!("Код отправлен в Telegram бот для {}", phone);
                    Ok(())
                } else {
                    Err(body
                        .error
                        .unwrap_or_else(|| "Telegram bot отказался отправить код".to_string()))
                }
            }
            Err(e) => {
                // Если бот не запущен или недоступен, это не критично
                tracing::debug!("Telegram bot недоступен: {}", e);
                Err(format!("Telegram bot недоступен: {}", e))
            }
        }
    }

    fn phone_hash(phone: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(phone.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Начинает процесс авторизации
    pub async fn start_auth(&self, phone: &str) -> AppResult<AuthStartResponse> {
        let normalized_phone = ValidationService::validate_phone(phone)?;

        let expires_in = (self.config.sms_code_expiration_minutes * 60) as u64;

        // Основной канал: Telegram Gateway API (если настроен и не dev-режим)
        if self.config.telegram_gateway_token.is_some() && !self.config.return_sms_code_in_response
        {
            let request_id = self
                .send_code_via_telegram_gateway(&normalized_phone)
                .await
                .map_err(AppError::Internal)?;

            let expires_at =
                Utc::now() + chrono::Duration::minutes(self.config.sms_code_expiration_minutes);
            self.gateway_requests.write().await.insert(
                normalized_phone.clone(),
                GatewayRequest {
                    request_id,
                    expires_at,
                },
            );

            return Ok(AuthStartResponse {
                code: String::new(),
                expires_in,
            });
        }

        // Генерируем код
        let code = self.sms_service.generate_code(&normalized_phone).await
            .map_err(|e| {
                tracing::error!("Failed to generate/send SMS code for {}: {}", normalized_phone, e);
                AppError::Internal(format!(
                    "Не удалось отправить SMS код. {}. Для разработки установите RETURN_SMS_CODE_IN_RESPONSE=true", e
                ))
            })?;

        // Отправляем код в Telegram бот (основной канал доставки)
        if let Err(e) = self.send_code_to_telegram(&normalized_phone, &code).await {
            tracing::warn!("Не удалось отправить код в Telegram бот: {}", e);
            if !self.config.return_sms_code_in_response {
                let bot_username = std::env::var("TELEGRAM_BOT_USERNAME").ok();
                let hint = match bot_username {
                    Some(u) if !u.trim().is_empty() => format!(
                        "Откройте Telegram бота @{} и нажмите Start, чтобы получить код.",
                        u.trim().trim_start_matches('@')
                    ),
                    _ => "Откройте Telegram бота и нажмите Start, чтобы получить код.".to_string(),
                };
                return Err(AppError::Auth(format!("{}. {}", e, hint)));
            }
        }

        // Возвращаем код в ответе только если return_sms_code_in_response = true (dev режим)
        // Иначе возвращаем пустую строку (код отправлен по SMS)
        let response_code = if self.config.return_sms_code_in_response {
            tracing::info!(
                "[DEV] Returning SMS code in response for {}: {}",
                normalized_phone,
                code
            );
            code
        } else {
            tracing::info!(
                "SMS code generated and sent (not returned in response) for {}",
                normalized_phone
            );
            String::new()
        };

        Ok(AuthStartResponse {
            code: response_code,
            expires_in,
        })
    }

    /// Проверяет код и создаёт/находит пользователя
    pub async fn verify_auth<R: UserRepository, RP: UserPlateRepository>(
        &self,
        phone: &str,
        code: &str,
        user_repository: &R,
        user_plate_repository: &RP,
    ) -> AppResult<AuthVerifyResponse> {
        let normalized_phone = ValidationService::validate_phone(phone)?;

        // Проверяем код (Telegram Gateway, если настроен и не dev-режим)
        if self.config.telegram_gateway_token.is_some() && !self.config.return_sms_code_in_response
        {
            let entry = self
                .gateway_requests
                .read()
                .await
                .get(&normalized_phone)
                .cloned();
            let entry =
                entry.ok_or_else(|| AppError::Auth("Код не запрошен или истёк".to_string()))?;
            if entry.expires_at <= Utc::now() {
                self.gateway_requests
                    .write()
                    .await
                    .remove(&normalized_phone);
                return Err(AppError::Auth("Код не запрошен или истёк".to_string()));
            }

            let is_valid = self
                .verify_code_via_telegram_gateway(&entry.request_id, code)
                .await
                .map_err(AppError::Auth)?;

            if !is_valid {
                return Err(AppError::Auth("Неверный код подтверждения".to_string()));
            }

            self.gateway_requests
                .write()
                .await
                .remove(&normalized_phone);
        } else {
            // Fallback: локальная проверка кода (dev/старый режим)
            if !self.sms_service.verify_code(&normalized_phone, code).await {
                return Err(AppError::Auth("Неверный код подтверждения".to_string()));
            }
        }

        // Хэш и шифруем телефон
        let phone_hash = Self::phone_hash(&normalized_phone);
        let phone_encrypted = self
            .encryption
            .encrypt(&normalized_phone)
            .map_err(|e| AppError::Encryption(e.to_string()))?;

        // Ищем или создаём пользователя
        let user = match user_repository.find_by_phone_hash(&phone_hash).await? {
            Some(user) => {
                // Пользователь существует - синхронизируем данные с user_plates
                tracing::info!("Existing user found: {}", user.id);

                // Проверяем наличие основного автомобиля
                let primary_plate = user_plate_repository
                    .find_primary_by_user_id(user.id)
                    .await?;

                if let Some(primary) = primary_plate {
                    // Основной автомобиль найден - синхронизируем номер в users.plate
                    if user.plate.as_deref() != Some(primary.plate.as_str()) {
                        tracing::info!(
                            "Syncing user {} plate from primary: {:?} -> {}",
                            user.id,
                            user.plate,
                            primary.plate
                        );
                        // Обновляем номер в users для обратной совместимости
                        let update_data = crate::repository::UpdateUserData {
                            name: None,
                            phone_encrypted: None,
                            phone_hash: None,
                            telegram: None,
                            plate: Some(primary.plate.clone()),
                            show_contacts: None,
                            owner_type: None,
                            owner_info: None,
                            departure_time: None,
                            push_token: None,
                        };
                        if let Ok(updated_user) =
                            user_repository.update(user.id, &update_data).await
                        {
                            tracing::info!("User plate synchronized successfully");
                            updated_user
                        } else {
                            tracing::warn!("Failed to sync user plate, using existing user");
                            user
                        }
                    } else {
                        user
                    }
                } else if let Some(ref plate) = user.plate {
                    // Нет основного автомобиля, но есть номер в users.plate - создаем его
                    tracing::info!(
                        "Creating primary plate for existing user {}: {}",
                        user.id,
                        plate
                    );
                    let normalized_plate = crate::utils::normalize_plate(plate);

                    // Проверяем валидность номера перед созданием
                    if !normalized_plate.is_empty() {
                        match user_plate_repository
                            .create(user.id, &normalized_plate, true, None)
                            .await
                        {
                            Ok(_) => {
                                tracing::info!("Primary plate created successfully");
                                user
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "Failed to create primary plate: {:?}, continuing with user",
                                    e
                                );
                                user
                            }
                        }
                    } else {
                        tracing::warn!(
                            "User {} has invalid plate, skipping primary plate creation",
                            user.id
                        );
                        user
                    }
                } else {
                    // У пользователя нет номера - это нормально, он добавит его позже
                    tracing::info!(
                        "User {} has no plate yet - user should add it later",
                        user.id
                    );
                    user
                }
            }
            None => {
                let new_user_id = Uuid::new_v4();
                let user = user_repository
                    .create(&CreateUserData {
                        id: new_user_id,
                        phone_encrypted,
                        phone_hash,
                        plate: String::new(), // Будет сохранено как NULL в БД
                    })
                    .await?;
                tracing::info!(
                    "Created new user {} without plate - user should add it later",
                    new_user_id
                );
                user
            }
        };

        // Удаляем использованный код
        self.sms_service.remove_code(&normalized_phone).await;

        // Создаём токен
        let token = create_token(user.id, &self.config)?;

        Ok(AuthVerifyResponse {
            token,
            user_id: user.id,
        })
    }

    /// Обновляет токен, если он еще действителен или истек недавно (в течение 30 минут)
    pub async fn refresh_token(&self, token: &str) -> AppResult<RefreshTokenResponse> {
        use jsonwebtoken::{decode, DecodingKey, Validation};
        use serde_json::Value;

        // Декодируем токен без проверки времени истечения
        let key = DecodingKey::from_secret(self.config.jwt_secret.as_ref());
        let mut validation = Validation::default();
        validation.validate_exp = false; // Отключаем проверку времени истечения

        let token_data = decode::<Value>(token, &key, &validation)
            .map_err(|e| AppError::Auth(format!("Invalid token format: {}", e)))?;

        // Извлекаем user_id и время создания
        let user_id_str = token_data
            .claims
            .get("sub")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Auth("Invalid token: missing user_id".to_string()))?;

        let user_id = uuid::Uuid::parse_str(user_id_str)
            .map_err(|_| AppError::Auth("Invalid token: invalid user_id format".to_string()))?;

        let iat = token_data
            .claims
            .get("iat")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| AppError::Auth("Invalid token: missing iat".to_string()))?;

        // Проверяем, не слишком ли давно истек токен (максимум 30 минут после истечения)
        // Это позволяет обновлять токен, если пользователь был неактивен недолго
        let now = chrono::Utc::now().timestamp();
        let token_age = now - iat;
        let max_age_after_expiry = 30 * 60; // 30 минут в секундах

        // Если токен слишком старый (больше времени жизни + окно обновления), требуем повторного входа
        let max_total_age = (self.config.jwt_expiration_minutes * 60) + max_age_after_expiry;
        if token_age > max_total_age {
            return Err(AppError::Auth(
                "Token expired too long ago. Please login again".to_string(),
            ));
        }

        // Создаём новый токен
        let new_token = create_token(user_id, &self.config)?;

        Ok(RefreshTokenResponse {
            token: new_token,
            user_id,
        })
    }
}
