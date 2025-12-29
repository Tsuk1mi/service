use uuid::Uuid;
use crate::auth::jwt::create_token;
use crate::auth::sms::SmsService;
use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::models::auth::{AuthStartResponse, AuthVerifyResponse, RefreshTokenResponse};
use crate::repository::{UserRepository, UserPlateRepository, CreateUserData};
use crate::service::validation_service::ValidationService;
use crate::utils::encryption::Encryption;
use sha2::{Sha256, Digest};

/// Сервис авторизации (SRP - Single Responsibility Principle)
#[derive(Clone)]
pub struct AuthService {
    sms_service: SmsService,
    encryption: Encryption,
    config: Config,
}

impl AuthService {
    pub fn new(sms_service: SmsService, encryption: Encryption, config: Config) -> Self {
        Self {
            sms_service,
            encryption,
            config,
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
        
        // Генерируем код
        let code = self.sms_service.generate_code(&normalized_phone).await
            .map_err(|e| {
                tracing::error!("Failed to generate/send SMS code for {}: {}", normalized_phone, e);
                AppError::Internal(format!(
                    "Не удалось отправить SMS код. {}. Для разработки установите RETURN_SMS_CODE_IN_RESPONSE=true", e
                ))
            })?;
        
        let expires_in = (self.config.sms_code_expiration_minutes * 60) as u64;

        // Возвращаем код в ответе только если return_sms_code_in_response = true (dev режим)
        // Иначе возвращаем пустую строку (код отправлен по SMS)
        let response_code = if self.config.return_sms_code_in_response {
            tracing::info!("[DEV] Returning SMS code in response for {}: {}", normalized_phone, code);
            code
        } else {
            tracing::info!("SMS code generated and sent (not returned in response) for {}", normalized_phone);
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

        // Проверяем код
        if !self.sms_service.verify_code(&normalized_phone, code).await {
            return Err(AppError::Auth("Неверный код подтверждения".to_string()));
        }

        // Хэш и шифруем телефон
        let phone_hash = Self::phone_hash(&normalized_phone);
        let phone_encrypted = self.encryption.encrypt(&normalized_phone)
            .map_err(|e| AppError::Encryption(e.to_string()))?;

        // Ищем или создаём пользователя
        let user = match user_repository.find_by_phone_hash(&phone_hash).await? {
            Some(user) => {
                // Пользователь существует - синхронизируем данные с user_plates
                tracing::info!("Existing user found: {}", user.id);
                
                // Проверяем наличие основного автомобиля
                let primary_plate = user_plate_repository.find_primary_by_user_id(user.id).await?;
                
                if let Some(primary) = primary_plate {
                    // Основной автомобиль найден - синхронизируем номер в users.plate
                    if user.plate.as_ref().map(|p| p.as_str()) != Some(primary.plate.as_str()) {
                        tracing::info!("Syncing user {} plate from primary: {:?} -> {}", user.id, user.plate, primary.plate);
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
                        if let Ok(updated_user) = user_repository.update(user.id, &update_data).await {
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
                    tracing::info!("Creating primary plate for existing user {}: {}", user.id, plate);
                    let normalized_plate = crate::utils::normalize_plate(plate);
                    
                    // Проверяем валидность номера перед созданием
                    if !normalized_plate.is_empty() {
                        match user_plate_repository.create(user.id, &normalized_plate, true, None).await {
                            Ok(_) => {
                                tracing::info!("Primary plate created successfully");
                                user
                            },
                            Err(e) => {
                                tracing::warn!("Failed to create primary plate: {:?}, continuing with user", e);
                                user
                            }
                        }
                    } else {
                        tracing::warn!("User {} has invalid plate, skipping primary plate creation", user.id);
                        user
                    }
                } else {
                    // У пользователя нет номера - это нормально, он добавит его позже
                    tracing::info!("User {} has no plate yet - user should add it later", user.id);
                    user
                }
            },
            None => {
                let new_user_id = Uuid::new_v4();
                let user = user_repository.create(&CreateUserData {
                    id: new_user_id,
                    phone_encrypted,
                    phone_hash,
                    plate: String::new(), // Будет сохранено как NULL в БД
                }).await?;
                tracing::info!("Created new user {} without plate - user should add it later", new_user_id);
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
        let user_id_str = token_data.claims.get("sub")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Auth("Invalid token: missing user_id".to_string()))?;
        
        let user_id = uuid::Uuid::parse_str(user_id_str)
            .map_err(|_| AppError::Auth("Invalid token: invalid user_id format".to_string()))?;
        
        let iat = token_data.claims.get("iat")
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
            return Err(AppError::Auth("Token expired too long ago. Please login again".to_string()));
        }

        // Создаём новый токен
        let new_token = create_token(user_id, &self.config)?;

        Ok(RefreshTokenResponse {
            token: new_token,
            user_id,
        })
    }
}

