use uuid::Uuid;
use crate::error::{AppError, AppResult};
use crate::models::user::{UpdateUserRequest, UserResponse};
use crate::repository::{UserRepository, UserPlateRepository, UpdateUserData};
use crate::service::validation_service::ValidationService;
use crate::utils::encryption::Encryption;
use sha2::{Sha256, Digest};

/// Сервис работы с пользователями (SRP)
#[derive(Clone)]
pub struct UserService {
    encryption: Encryption,
}

impl UserService {
    pub fn new(encryption: Encryption) -> Self {
        Self { encryption }
    }

    fn phone_hash(phone: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(phone.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Получает профиль пользователя
    pub async fn get_profile<R: UserRepository, RP: UserPlateRepository>(
        &self,
        user_id: Uuid,
        repository: &R,
        user_plate_repository: &RP,
    ) -> AppResult<UserResponse> {
        tracing::info!("get_profile called for user_id: {}", user_id);
        let mut user = repository.find_by_id(user_id).await?
            .ok_or_else(|| {
                tracing::error!("User not found: {}", user_id);
                AppError::NotFound("User not found".to_string())
            })?;
        tracing::info!("User found: {} (plate: {:?})", user_id, user.plate);

        // Получаем актуальный основной номер из user_plates (источник истины)
        if let Ok(Some(primary_plate)) = user_plate_repository.find_primary_by_user_id(user_id).await {
            // Синхронизируем номер пользователя из основного автомобиля
            if user.plate.as_ref().map(|p| p.as_str()) != Some(primary_plate.plate.as_str()) {
                tracing::info!("Syncing user {} plate from user_plates: {:?} -> {}", user_id, user.plate, primary_plate.plate);
                // Обновляем номер в users для обратной совместимости
                let update_data = UpdateUserData {
                    name: None,
                    phone_encrypted: None,
                    phone_hash: None,
                    telegram: None,
                    plate: Some(primary_plate.plate.clone()),
                    show_contacts: None,
                    owner_type: None,
                    owner_info: None,
                    departure_time: None, // Не изменяем время выезда при синхронизации
                    push_token: None,
                };
                if let Ok(updated) = repository.update(user_id, &update_data).await {
                    user = updated;
                }
            }
        } else {
            // Если нет основного автомобиля, создаем его из users.plate (если номер есть и валиден)
            if let Some(ref plate) = user.plate {
                tracing::info!("No primary plate found for user {}, creating from users.plate: {}", user_id, plate);
                let normalized_plate = crate::utils::normalize_plate(plate);
                
                // Проверяем валидность номера перед созданием
                if !normalized_plate.is_empty() {
                        match user_plate_repository.create(user_id, &normalized_plate, true, None).await {
                        Ok(_) => {
                            tracing::info!("Primary plate created successfully from users.plate");
                        },
                        Err(e) => {
                            tracing::warn!("Failed to create primary plate from users.plate: {:?}", e);
                            // Продолжаем выполнение, даже если не удалось создать primary_plate
                        }
                    }
                } else {
                    tracing::warn!("User {} has invalid plate in users.plate, skipping primary plate creation", user_id);
                }
            } else {
                tracing::info!("No primary plate found for user {} and users.plate is empty - user should add plate later", user_id);
            }
        }

        let phone_decrypted = user.phone_encrypted.as_ref()
            .and_then(|enc| self.encryption.decrypt(enc).ok());

        Ok(user.to_response(phone_decrypted))
    }

    /// Обновляет профиль пользователя
    pub async fn update_profile<R: UserRepository, RP: UserPlateRepository>(
        &self,
        user_id: Uuid,
        request: UpdateUserRequest,
        repository: &R,
        user_plate_repository: &RP,
    ) -> AppResult<UserResponse> {
        tracing::info!("update_profile called for user {}: name={:?}, phone={:?}, telegram={:?}, plate={:?}, owner_type={:?}, owner_info={:?}", 
            user_id, 
            request.name.as_ref().map(|s| if s.is_empty() { "<empty>" } else { s.as_str() }),
            request.phone.as_ref().map(|_| "<present>"),
            request.telegram.as_ref().map(|s| if s.is_empty() { "<empty>" } else { s.as_str() }),
            request.plate.as_ref().map(|s| s.as_str()),
            request.owner_type.as_ref().map(|s| s.as_str()),
            request.owner_info.is_some()
        );
        
        // Валидация полей вручную (проверяем только непустые значения)
        if let Some(ref name) = request.name {
            if !name.is_empty() && (name.len() < 1 || name.len() > 20) {
                return Err(AppError::Validation("Имя должно быть от 1 до 20 символов".to_string()));
            }
        }
        
        if let Some(ref phone) = request.phone {
            if !phone.is_empty() {
                ValidationService::validate_phone(phone)?;
            }
        }
        
        if let Some(ref telegram) = request.telegram {
            if !telegram.is_empty() && telegram.len() > 32 {
                return Err(AppError::Validation("Telegram username должен быть до 32 символов".to_string()));
            }
        }
        
        if let Some(ref plate) = request.plate {
            if !plate.is_empty() {
                ValidationService::validate_plate(plate)?;
            }
        }

        // Нормализация данных
        let mut normalized_request = request;
        normalized_request.normalize();
        
        // Преобразуем пустые строки в None для корректной обработки
        if let Some(ref name) = normalized_request.name {
            if name.is_empty() {
                normalized_request.name = None;
            }
        }
        if let Some(ref phone) = normalized_request.phone {
            if phone.is_empty() {
                normalized_request.phone = None;
            }
        }
        if let Some(ref telegram) = normalized_request.telegram {
            if telegram.is_empty() {
                normalized_request.telegram = None;
            }
        }

        // Шифрование телефона если он обновляется
        let (phone_encrypted, phone_hash) = if let Some(phone) = normalized_request.phone {
            let enc = self.encryption.encrypt(&phone)
                .map_err(|e| AppError::Encryption(e.to_string()))?;
            let hash = Self::phone_hash(&phone);
            (Some(enc), Some(hash))
        } else {
            (None, None)
        };

        // ВСЕГДА синхронизируем номер автомобиля с user_plates
        // Если plate передан - обновляем/создаем основной автомобиль
        if let Some(new_plate) = &normalized_request.plate {
            let normalized_plate = crate::utils::normalize_plate(new_plate);
            
            // Находим текущий основной автомобиль
            if let Ok(Some(primary_plate)) = user_plate_repository.find_primary_by_user_id(user_id).await {
                if primary_plate.plate != normalized_plate {
                    // Номер изменился - обновляем основной автомобиль
                    tracing::info!("Syncing primary plate for user {}: {} -> {}", user_id, primary_plate.plate, normalized_plate);
                    
                    // Проверяем, есть ли уже такой номер у пользователя
                    let existing_plates = user_plate_repository.find_by_user_id(user_id).await?;
                    let existing_plate = existing_plates.iter().find(|p| p.plate == normalized_plate);
                    
                    if let Some(existing) = existing_plate {
                        // Номер уже существует - делаем его основным
                        user_plate_repository.set_primary(existing.id, user_id).await?;
                    } else {
                        // Создаем новый номер как основной
                        user_plate_repository.create(user_id, &normalized_plate, true, None).await?;
                    }
                }
            } else {
                // Нет основного автомобиля - создаем из переданного номера
                tracing::info!("Creating primary plate for user {}: {}", user_id, normalized_plate);
                user_plate_repository.create(user_id, &normalized_plate, true, None).await?;
            }
        } else {
            // Если plate не передан, используем номер из основного автомобиля
            if let Ok(Some(primary_plate)) = user_plate_repository.find_primary_by_user_id(user_id).await {
                tracing::info!("No plate in request, using primary plate for user {}: {}", user_id, primary_plate.plate);
                normalized_request.plate = Some(primary_plate.plate.clone());
            }
        }

        // Парсинг времени выезда
        let departure_time = if let Some(ref time_str) = normalized_request.departure_time {
            if time_str.is_empty() {
                None
            } else {
                chrono::NaiveTime::parse_from_str(time_str, "%H:%M")
                    .map_err(|_| AppError::Validation("Неверный формат времени выезда. Используйте формат HH:MM (например, 18:30)".to_string()))?
                    .into()
            }
        } else {
            None
        };

        // Обновление в БД
        let update_data = UpdateUserData {
            name: normalized_request.name,
            phone_encrypted,
            phone_hash,
            telegram: normalized_request.telegram,
            plate: normalized_request.plate.clone(),
            show_contacts: normalized_request.show_contacts,
            owner_type: normalized_request.owner_type,
            owner_info: normalized_request.owner_info,
            departure_time,
            push_token: None,
        };

        let updated_user = repository.update(user_id, &update_data).await?;

        // Расшифровка телефона для ответа
        let phone_decrypted = updated_user.phone_encrypted.as_ref()
            .and_then(|enc| self.encryption.decrypt(enc).ok());

        Ok(updated_user.to_response(phone_decrypted))
    }

    /// Получает публичную информацию о пользователе по номеру автомобиля
    pub async fn get_user_by_plate<R: UserRepository, RP: UserPlateRepository>(
        &self,
        plate: &str,
        repository: &R,
        user_plate_repository: &RP,
    ) -> AppResult<Option<crate::models::user::PublicUserInfo>> {
        let normalized_plate = ValidationService::validate_plate(plate)?;
        
        // Находим пользователей, у которых этот номер в user_plates
        let user_plates = user_plate_repository.find_by_plate(&normalized_plate).await?;
        
        // Берем первого пользователя с этим номером (обычно он один)
        if let Some(user_plate) = user_plates.first() {
            if let Some(user) = repository.find_by_id(user_plate.user_id).await? {
                let phone_decrypted = user.phone_encrypted.as_ref()
                    .and_then(|enc| self.encryption.decrypt(enc).ok());
                
                return Ok(Some(user.to_public_info(phone_decrypted)));
            }
        }
        
        Ok(None)
    }
}

