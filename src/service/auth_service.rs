use crate::auth::jwt::{create_token_pair, decode_token_ignore_exp, verify_refresh_token, TokenType};
use crate::auth::sms::SmsService;
use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::metrics;
use crate::models::auth::{AuthStartResponse, AuthVerifyResponse, RefreshTokenResponse};
use crate::queue::EventPublisher;
use crate::redis::JwtBlacklist;
use crate::repository::{CreateUserData, UserPlateRepository, UserRepository};
use crate::service::validation_service::ValidationService;
use crate::utils::encryption::Encryption;
use crate::utils::phone::phone_hash;
use reqwest::Client;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

/// Сервис авторизации (SRP - Single Responsibility Principle)
#[derive(Clone)]
pub struct AuthService {
    sms_service: SmsService,
    encryption: Encryption,
    config: Config,
    http_client: Client,
    event_publisher: Arc<dyn EventPublisher>,
    jwt_blacklist: Option<JwtBlacklist>,
}

impl AuthService {
    pub fn new(
        sms_service: SmsService,
        encryption: Encryption,
        config: Config,
        event_publisher: Arc<dyn EventPublisher>,
        jwt_blacklist: Option<JwtBlacklist>,
    ) -> Self {
        Self {
            sms_service,
            encryption,
            config: config.clone(),
            http_client: Client::new(),
            event_publisher,
            jwt_blacklist,
        }
    }

    /// Отправляет код авторизации в Telegram бот
    async fn send_code_to_telegram(&self, phone: &str, code: &str) -> Result<(), String> {
        let bot_url = self.config.telegram_bot_http_url.clone().unwrap_or_else(|| {
            let bot_port = self
                .config
                .server_port
                .checked_add(1)
                .unwrap_or(self.config.server_port);
            format!("http://localhost:{}/send_code", bot_port)
        });

        let payload = json!({
            "phone": phone,
            "code": code
        });

        let mut request = self.http_client.post(&bot_url).json(&payload);
        if let Some(ref token) = self.config.internal_api_token {
            request = request.header("X-Internal-Token", token);
        }

        match request.send().await {
            Ok(response) => {
                if response.status().is_success() {
                    tracing::info!("Код отправлен в Telegram бот для {}", phone);
                    Ok(())
                } else {
                    Err(format!(
                        "Telegram bot returned error: {}",
                        response.status()
                    ))
                }
            }
            Err(e) => {
                // Если бот не запущен или недоступен, это не критично
                tracing::debug!("Telegram bot недоступен: {}", e);
                Err(format!("Telegram bot недоступен: {}", e))
            }
        }
    }


    /// Начинает процесс авторизации
    pub async fn start_auth(&self, phone: &str) -> AppResult<AuthStartResponse> {
        let normalized_phone = ValidationService::validate_phone(phone)?;

        self.sms_service.check_rate_limit(&normalized_phone).await?;

        let code = self
            .sms_service
            .generate_code(&normalized_phone)
            .await
            .map_err(|e| {
                tracing::error!(
                    "Failed to generate OTP for {}: {:?}",
                    normalized_phone,
                    e
                );
                e
            })?;

        metrics::record_otp_sent();

        // Отправляем код в Telegram бот (если настроен)
        if let Err(e) = self.send_code_to_telegram(&normalized_phone, &code).await {
            tracing::warn!("Не удалось отправить код в Telegram бот: {}", e);
            // Не прерываем процесс, если не удалось отправить в Telegram
        }

        // Отправляем SMS через внешний провайдер или очередь
        if let Some(sms_url) = &self.config.sms_api_url {
            if let Err(e) = Self::send_sms_via_provider(
                sms_url,
                self.config.sms_api_key.as_deref(),
                &normalized_phone,
                &code,
            )
            .await
            {
                tracing::warn!("Не удалось отправить SMS через провайдера: {}", e);
            }
        } else if let Err(e) = self
            .event_publisher
            .publish_sms(&normalized_phone, &code)
            .await
        {
            tracing::warn!("Не удалось поставить SMS в очередь: {:?}", e);
        }

        let expires_in = (self.config.sms_code_expiration_minutes * 60) as u64;

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

        let telegram_bot_username = std::env::var("TELEGRAM_BOT_USERNAME").ok();
        let telegram_deeplink = telegram_bot_username.as_deref().map(|u| {
            // start payload: p<digits> (no '+'), e.g. p79001234567
            let digits: String = normalized_phone
                .chars()
                .filter(|c| c.is_ascii_digit())
                .collect();
            format!(
                "https://t.me/{}?start=p{}",
                u.trim().trim_start_matches('@'),
                digits
            )
        });

        Ok(AuthStartResponse {
            code: response_code,
            expires_in,
            telegram_bot_username,
            telegram_deeplink,
        })
    }

    /// Отправка SMS через внешний провайдер (простой POST)
    async fn send_sms_via_provider(
        url: &str,
        api_key: Option<&str>,
        phone: &str,
        code: &str,
    ) -> Result<(), String> {
        let client = reqwest::Client::new();
        let mut request = client.post(url).json(&serde_json::json!({
            "phone": phone,
            "code": code,
            "message": format!("Ваш код подтверждения: {}", code),
        }));

        if let Some(key) = api_key {
            request = request.header("Authorization", format!("Bearer {}", key));
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("SMS API request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("SMS API error: {} - {}", status, text));
        }

        tracing::info!("SMS отправлено через провайдера на {}", phone);
        Ok(())
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

        if !self.sms_service.verify_code(&normalized_phone, code).await? {
            return Err(AppError::Auth("Неверный код подтверждения".to_string()));
        }

        // Хэш и шифруем телефон
        let phone_hash = phone_hash(&normalized_phone);
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

        self.sms_service.remove_code(&normalized_phone).await?;

        let tokens = create_token_pair(user.id, &self.config)?;

        Ok(AuthVerifyResponse {
            token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            user_id: user.id,
        })
    }

    /// Обновляет access token по refresh token
    pub async fn refresh_token(&self, token: &str) -> AppResult<RefreshTokenResponse> {
        let claims = verify_refresh_token(token, &self.config)?;

        if let Some(ref blacklist) = self.jwt_blacklist {
            if blacklist.is_blacklisted(&claims.jti).await? {
                return Err(AppError::Auth("Refresh token revoked".to_string()));
            }
        }

        let now = chrono::Utc::now().timestamp();
        let max_age_after_expiry = 30 * 60;
        let max_total_age = (self.config.jwt_refresh_expiration_minutes * 60) + max_age_after_expiry;
        let token_age = now - claims.iat;

        if token_age > max_total_age {
            return Err(AppError::Auth(
                "Refresh token expired too long ago. Please login again".to_string(),
            ));
        }

        if let Some(ref blacklist) = self.jwt_blacklist {
            let ttl = (claims.exp - now).max(0) as u64;
            if ttl > 0 {
                blacklist.blacklist_jti(&claims.jti, ttl).await?;
            }
        }

        let tokens = create_token_pair(claims.sub, &self.config)?;

        Ok(RefreshTokenResponse {
            token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            user_id: claims.sub,
        })
    }

    /// Отзывает refresh token (logout)
    pub async fn logout(&self, token: &str) -> AppResult<()> {
        let claims = decode_token_ignore_exp(token, &self.config)?;
        if claims.typ != TokenType::Refresh {
            return Err(AppError::Auth("Expected refresh token".to_string()));
        }

        if let Some(ref blacklist) = self.jwt_blacklist {
            let now = chrono::Utc::now().timestamp();
            let ttl = (claims.exp - now).max(0) as u64;
            if ttl > 0 {
                blacklist.blacklist_jti(&claims.jti, ttl).await?;
            }
        }

        Ok(())
    }
}
