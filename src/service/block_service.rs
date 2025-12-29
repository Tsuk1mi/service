use crate::error::{AppError, AppResult};
use crate::models::block::{Block, BlockWithBlockerInfo, CheckBlockResponse, CreateBlockRequest};
use crate::repository::{
    BlockRepository, CreateNotificationData, NotificationRepository, UserPlateRepository,
    UserRepository,
};
use crate::service::{
    telegram_service::TelegramService, telephony_service::TelephonyService,
    validation_service::ValidationService,
};
use crate::utils::encryption::Encryption;
use uuid::Uuid;

/// Сервис работы с блокировками (SRP)
#[derive(Clone)]
pub struct BlockService {
    encryption: Encryption,
    push_service: crate::service::push_service::PushService,
}

impl BlockService {
    pub fn new(
        encryption: Encryption,
        push_service: crate::service::push_service::PushService,
    ) -> Self {
        Self {
            encryption,
            push_service,
        }
    }

    /// Создаёт новую блокировку
    #[allow(clippy::too_many_arguments)]
    pub async fn create_block<
        BR: BlockRepository,
        NR: NotificationRepository,
        UR: UserRepository,
        UPR: UserPlateRepository,
    >(
        &self,
        blocker_id: Uuid,
        mut request: CreateBlockRequest,
        block_repository: &BR,
        notification_repository: &NR,
        user_repository: &UR,
        user_plate_repository: &UPR,
        telephony_service: &TelephonyService,
        telegram_service: &TelegramService,
    ) -> AppResult<Block> {
        // Нормализация и валидация
        request.normalize();
        let normalized_plate =
            ValidationService::validate_plate(&request.blocked_plate).map_err(|e| {
                tracing::error!(
                    "Plate validation failed for '{}' (normalized: '{}'): {:?}",
                    request.blocked_plate,
                    crate::utils::normalize_plate(&request.blocked_plate),
                    e
                );
                e
            })?;

        // Определяем основной номер блокирующего (для совместных владельцев и запрета самоблокировки)
        let blocker_primary_plate = if let Ok(Some(primary_plate)) = user_plate_repository
            .find_primary_by_user_id(blocker_id)
            .await
        {
            primary_plate.plate
        } else {
            let plates = user_plate_repository.find_by_user_id(blocker_id).await?;
            plates.first().map(|p| p.plate.clone()).ok_or_else(|| {
                AppError::Validation("Сначала добавьте свой автомобиль".to_string())
            })?
        };

        // Запрещаем взаимные блокировки (нельзя перекрыть собственный авто)
        if blocker_primary_plate.eq_ignore_ascii_case(&normalized_plate) {
            tracing::warn!(
                "User {} attempted to block their own plate {}, blocking denied",
                blocker_id,
                normalized_plate
            );
            return Err(AppError::Validation(
                "Нельзя перекрыть свой же автомобиль".to_string(),
            ));
        }

        tracing::info!(
            "Creating block for user {} and plate {}",
            blocker_id,
            normalized_plate
        );

        // Оптимизированная проверка на дубликаты - используем EXISTS вместо загрузки всех блокировок
        let exists = block_repository
            .exists(blocker_id, &normalized_plate)
            .await
            .map_err(|e| {
                tracing::error!("Failed to check existing blocks: {:?}", e);
                e
            })?;

        if exists {
            tracing::warn!(
                "Block already exists for user {} and plate {}",
                blocker_id,
                normalized_plate
            );
            return Err(crate::error::AppError::Validation(
                "Эта блокировка уже существует".to_string(),
            ));
        }

        // Создание блокировки
        let block = block_repository
            .create(blocker_id, &blocker_primary_plate, &normalized_plate)
            .await
            .map_err(|e| {
                tracing::error!("Failed to create block: {:?}", e);
                e
            })?;

        // Если передано время выезда — привязываем к основному номеру блокирующего
        if let Some(time_str) = request.departure_time.as_ref() {
            if !time_str.is_empty() {
                match chrono::NaiveTime::parse_from_str(time_str, "%H:%M") {
                    Ok(dt) => {
                        if let Ok(Some(primary_plate)) = user_plate_repository
                            .find_primary_by_user_id(blocker_id)
                            .await
                        {
                            let _ = user_plate_repository
                                .update_departure_time(primary_plate.id, blocker_id, Some(dt))
                                .await;
                        }
                    }
                    Err(_) => {
                        tracing::warn!(
                            "Invalid departure_time format, expected HH:MM: {}",
                            time_str
                        );
                    }
                }
            }
        }

        tracing::info!("Block created successfully: {}", block.id);

        // Создаём уведомления для владельцев заблокированного автомобиля
        // Получаем информацию о блокирующем
        if let Ok(Some(blocker_user)) = user_repository.find_by_id(blocker_id).await {
            let blocker_name = blocker_user.name.as_deref().unwrap_or("Неизвестно");
            let mut notified_users = std::collections::HashSet::new();

            // Находим пользователей, у которых этот номер в user_plates
            if let Ok(user_plates) = user_plate_repository.find_by_plate(&normalized_plate).await {
                for user_plate in user_plates {
                    let user_id = user_plate.user_id;

                    // Не отправляем уведомление самому блокирующему и избегаем дубликатов
                    if user_id == blocker_id || notified_users.contains(&user_id) {
                        continue;
                    }

                    notified_users.insert(user_id);

                    // Получаем владельца
                    let owner_user = user_repository.find_by_id(user_id).await.ok().flatten();

                    // Создаём уведомление
                    let _ = notification_repository
                        .create(&CreateNotificationData {
                            user_id,
                            r#type: "block".to_string(),
                            title: "Ваш автомобиль заблокирован".to_string(),
                            message: format!(
                                "Автомобиль {} заблокирован пользователем {}",
                                normalized_plate, blocker_name
                            ),
                            data: Some(serde_json::json!({
                                "block_id": block.id,
                                "blocked_plate": normalized_plate,
                                "blocker_id": blocker_id,
                                "blocker_name": blocker_name,
                            })),
                        })
                        .await
                        .map_err(|e| {
                            tracing::error!("Failed to create notification: {:?}", e);
                        });

                    // Отправка уведомлений в зависимости от выбранного способа
                    let notification_method = request
                        .notification_method
                        .as_deref()
                        .unwrap_or("android_push");

                    if notification_method == "telegram" {
                        // Отправка через Telegram
                        if let Some(owner_user) = owner_user.as_ref() {
                            if let Some(telegram_username) = owner_user.telegram.as_ref() {
                                let telegram_service_clone = telegram_service.clone();
                                let telegram_username_clone = telegram_username.clone();
                                let normalized_plate_clone = normalized_plate.clone();
                                let blocker_name_clone = blocker_name.to_string();

                                tokio::spawn(async move {
                                    if let Err(e) = telegram_service_clone
                                        .send_block_notification(
                                            &telegram_username_clone,
                                            &normalized_plate_clone,
                                            &blocker_name_clone,
                                        )
                                        .await
                                    {
                                        tracing::warn!(
                                            "Failed to send Telegram notification: {}",
                                            e
                                        );
                                    }
                                });
                            } else {
                                tracing::warn!(
                                    "User {} has no Telegram username for notification",
                                    user_id
                                );
                            }
                        }
                    } else {
                        // Отправка через Android Push (по умолчанию)
                        if let Some(owner_user) = owner_user.as_ref() {
                            if let Some(push_token) = owner_user.push_token.clone() {
                                let title = "Ваш авто заблокирован";
                                let body =
                                    format!("{} перекрыл {}.", blocker_name, normalized_plate);
                                let data = serde_json::json!({
                                    "block_id": block.id.to_string(),
                                    "blocked_plate": normalized_plate,
                                    "blocker_name": blocker_name,
                                });
                                let push = self.push_service.clone();
                                tokio::spawn(async move {
                                    if let Err(e) =
                                        push.send_fcm(&push_token, title, &body, data).await
                                    {
                                        tracing::warn!("Failed to send FCM push: {}", e);
                                    }
                                });
                            }
                        }
                    }

                    // Если запрошено уведомление владельца, звоним ему
                    if request.notify_owner {
                        if let Some(owner_user) = owner_user {
                            if let Some(phone_encrypted) = owner_user.phone_encrypted {
                                if let Ok(phone) = self.encryption.decrypt(&phone_encrypted) {
                                    let message = telephony_service
                                        .format_block_notification_message(
                                            &normalized_plate,
                                            blocker_name,
                                        );

                                    // Совершаем звонок в фоновом режиме (не блокируем ответ)
                                    let telephony_service_clone = telephony_service.clone();
                                    let phone_clone = phone.clone();
                                    let message_clone = message.clone();

                                    tokio::spawn(async move {
                                        if let Err(e) = telephony_service_clone
                                            .call_owner(&phone_clone, &message_clone)
                                            .await
                                        {
                                            tracing::error!(
                                                "Failed to call owner {}: {}",
                                                phone_clone,
                                                e
                                            );
                                        }
                                    });

                                    tracing::info!(
                                        "Calling owner {} about block on {}",
                                        phone,
                                        normalized_plate
                                    );
                                } else {
                                    tracing::warn!("Failed to decrypt phone for user {}", user_id);
                                }
                            } else {
                                tracing::warn!(
                                    "User {} has no phone number for notification call",
                                    user_id
                                );
                            }
                        }
                    }
                }
            }
        }

        Ok(block)
    }

    /// Получает блокировки пользователя
    pub async fn get_my_blocks<BR: BlockRepository, UPR: UserPlateRepository>(
        &self,
        blocker_id: Uuid,
        block_repository: &BR,
        user_plate_repository: &UPR,
    ) -> AppResult<Vec<Block>> {
        // Блокировки, созданные этим пользователем
        let mut result = block_repository.find_by_blocker_id(blocker_id).await?;

        // Блокировки, созданные владельцами того же автомобиля (по номеру)
        let plates = user_plate_repository.find_by_user_id(blocker_id).await?;
        let plate_strings: Vec<String> = plates.iter().map(|p| p.plate.clone()).collect();
        let mut shared = block_repository
            .find_by_blocker_plates(&plate_strings)
            .await?;

        result.append(&mut shared);
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        result.dedup_by(|a, b| a.id == b.id);

        Ok(result)
    }

    /// Получает блокировки для номера автомобиля пользователя
    pub async fn get_blocks_for_my_plate<
        BR: BlockRepository,
        UR: UserRepository,
        UPR: UserPlateRepository,
    >(
        &self,
        user_id: Uuid,
        my_plate: Option<String>,
        block_repository: &BR,
        user_repository: &UR,
        user_plate_repository: &UPR,
    ) -> AppResult<Vec<BlockWithBlockerInfo>> {
        // Если указан конкретный номер, проверяем только его
        if let Some(plate) = my_plate {
            let normalized_plate = ValidationService::validate_plate(&plate)?;
            return self
                .get_blocks_for_plate(&normalized_plate, block_repository, user_repository)
                .await;
        }

        // Иначе проверяем все номера пользователя из user_plates
        let user_plates = user_plate_repository.find_by_user_id(user_id).await?;

        // Собираем все блокировки для всех номеров пользователя
        let mut all_blocks = Vec::new();
        let mut seen_block_ids = std::collections::HashSet::new();

        for user_plate in user_plates {
            let blocks = self
                .get_blocks_for_plate(&user_plate.plate, block_repository, user_repository)
                .await?;
            for block in blocks {
                // Избегаем дубликатов блокировок (если один номер заблокирован несколько раз)
                if !seen_block_ids.contains(&block.id) {
                    seen_block_ids.insert(block.id);
                    all_blocks.push(block);
                }
            }
        }

        // Сортируем по дате создания (новые сначала)
        all_blocks.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(all_blocks)
    }

    /// Вспомогательный метод для получения блокировок по номеру
    async fn get_blocks_for_plate<BR: BlockRepository, UR: UserRepository>(
        &self,
        plate: &str,
        block_repository: &BR,
        user_repository: &UR,
    ) -> AppResult<Vec<BlockWithBlockerInfo>> {
        // Получаем блокировки
        let blocks = block_repository.find_by_blocked_plate(plate).await?;

        // Для каждой блокировки получаем информацию о блокирующем
        let mut result = Vec::new();
        for block in blocks {
            if let Some(blocker_user) = user_repository.find_by_id(block.blocker_id).await? {
                let phone_decrypted = blocker_user
                    .phone_encrypted
                    .as_ref()
                    .and_then(|enc| self.encryption.decrypt(enc).ok());

                result.push(BlockWithBlockerInfo {
                    id: block.id,
                    blocked_plate: block.blocked_plate,
                    created_at: block.created_at,
                    blocker: blocker_user.to_public_info(phone_decrypted),
                    blocker_owner_type: blocker_user.owner_type.clone(),
                    blocker_owner_info: blocker_user.owner_info.clone(),
                });
            }
        }

        Ok(result)
    }

    /// Удаляет блокировку (только если пользователь является её создателем)
    pub async fn delete_block<
        BR: BlockRepository,
        NR: NotificationRepository,
        UR: UserRepository,
        UPR: UserPlateRepository,
    >(
        &self,
        block_id: Uuid,
        blocker_id: Uuid,
        block_repository: &BR,
        notification_repository: &NR,
        user_repository: &UR,
        user_plate_repository: &UPR,
    ) -> AppResult<()> {
        // Проверяем, что блокировка существует и принадлежит пользователю
        let block = block_repository
            .find_by_id(block_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Block not found".to_string()))?;

        if block.blocker_id != blocker_id {
            return Err(AppError::Auth(
                "You don't have permission to delete this block".to_string(),
            ));
        }

        // Выполняем удаление
        block_repository.delete(block_id, blocker_id).await?;

        // Рассылаем уведомления и пуш владельцам, чьи машины были разблокированы
        let blocked_plate = block.blocked_plate.clone();

        // Имя блокировщика для сообщения
        let blocker_name = user_repository
            .find_by_id(blocker_id)
            .await?
            .and_then(|u| u.name)
            .unwrap_or_else(|| "Неизвестно".to_string());

        // Находим всех владельцев номера
        if let Ok(user_plates) = user_plate_repository.find_by_plate(&blocked_plate).await {
            let mut notified_users = std::collections::HashSet::new();

            for user_plate in user_plates {
                let user_id = user_plate.user_id;

                // Не уведомляем самого блокировщика и избегаем дубликатов
                if user_id == blocker_id || notified_users.contains(&user_id) {
                    continue;
                }
                notified_users.insert(user_id);

                if let Some(owner_user) = user_repository.find_by_id(user_id).await? {
                    // Сохраняем уведомление в БД
                    let _ = notification_repository
                        .create(&CreateNotificationData {
                            user_id,
                            r#type: "unblock".to_string(),
                            title: "Автомобиль разблокирован".to_string(),
                            message: format!(
                                "Автомобиль {} разблокирован пользователем {}",
                                blocked_plate, blocker_name
                            ),
                            data: Some(serde_json::json!({
                                "block_id": block_id,
                                "blocked_plate": blocked_plate,
                                "blocker_id": blocker_id,
                                "blocker_name": blocker_name,
                                "status": "unblocked"
                            })),
                        })
                        .await
                        .map_err(|e| {
                            tracing::error!("Failed to create unblock notification: {:?}", e)
                        });

                    // Пуш-уведомление через FCM, если есть токен
                    if let Some(push_token) = owner_user.push_token.clone() {
                        let title = "Ваш авто разблокирован";
                        let body =
                            format!("{} больше не перекрывает {}.", blocker_name, blocked_plate);
                        let data = serde_json::json!({
                            "block_id": block_id.to_string(),
                            "blocked_plate": blocked_plate,
                            "blocker_name": blocker_name,
                            "status": "unblocked"
                        });
                        let push = self.push_service.clone();
                        tokio::spawn(async move {
                            if let Err(e) = push.send_fcm(&push_token, title, &body, data).await {
                                tracing::warn!("Failed to send FCM push (unblock): {}", e);
                            }
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// Проверяет, заблокирована ли машина
    pub async fn check_block<BR: BlockRepository, UR: UserRepository>(
        &self,
        plate: &str,
        block_repository: &BR,
        user_repository: &UR,
    ) -> AppResult<CheckBlockResponse> {
        let normalized_plate = ValidationService::validate_plate(plate)?;

        let blocks = block_repository
            .find_by_blocked_plate(&normalized_plate)
            .await?;

        if blocks.is_empty() {
            return Ok(CheckBlockResponse {
                is_blocked: false,
                block: None,
            });
        }

        // Берём самую свежую блокировку
        let latest_block = blocks
            .into_iter()
            .max_by_key(|b| b.created_at)
            .ok_or_else(|| AppError::Internal("Failed to find latest block".to_string()))?;

        // Получаем информацию о блокирующем
        if let Some(blocker_user) = user_repository.find_by_id(latest_block.blocker_id).await? {
            let phone_decrypted = blocker_user
                .phone_encrypted
                .as_ref()
                .and_then(|enc| self.encryption.decrypt(enc).ok());

            Ok(CheckBlockResponse {
                is_blocked: true,
                block: Some(BlockWithBlockerInfo {
                    id: latest_block.id,
                    blocked_plate: latest_block.blocked_plate,
                    created_at: latest_block.created_at,
                    blocker: blocker_user.to_public_info(phone_decrypted),
                    blocker_owner_type: blocker_user.owner_type.clone(),
                    blocker_owner_info: blocker_user.owner_info.clone(),
                }),
            })
        } else {
            Ok(CheckBlockResponse {
                is_blocked: true,
                block: None,
            })
        }
    }

    /// Предупреждает владельца заблокированного автомобиля (звонок)
    pub async fn warn_owner<BR: BlockRepository, UR: UserRepository, UPR: UserPlateRepository>(
        &self,
        block_id: Uuid,
        blocker_id: Uuid,
        block_repository: &BR,
        user_repository: &UR,
        user_plate_repository: &UPR,
        telephony_service: &TelephonyService,
    ) -> AppResult<()> {
        // Проверяем, что блокировка существует и принадлежит пользователю
        let block = block_repository
            .find_by_id(block_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Block not found".to_string()))?;

        if block.blocker_id != blocker_id {
            return Err(AppError::Auth(
                "You don't have permission to warn owner for this block".to_string(),
            ));
        }

        // Получаем информацию о блокирующем
        let blocker_user = user_repository
            .find_by_id(blocker_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Blocker user not found".to_string()))?;
        let blocker_name = blocker_user.name.as_deref().unwrap_or("Неизвестно");

        // Находим пользователей, у которых этот номер в user_plates
        let user_plates = user_plate_repository
            .find_by_plate(&block.blocked_plate)
            .await?;

        let mut called = false;
        for user_plate in user_plates {
            let user_id = user_plate.user_id;

            // Не звоним самому блокирующему
            if user_id == blocker_id {
                continue;
            }

            // Находим пользователя и звоним ему
            if let Some(owner_user) = user_repository.find_by_id(user_id).await? {
                if let Some(phone_encrypted) = owner_user.phone_encrypted {
                    if let Ok(phone) = self.encryption.decrypt(&phone_encrypted) {
                        let message = telephony_service
                            .format_block_notification_message(&block.blocked_plate, blocker_name);

                        // Совершаем звонок в фоновом режиме (не блокируем ответ)
                        let telephony_service_clone = telephony_service.clone();
                        let phone_clone = phone.clone();
                        let message_clone = message.clone();

                        tokio::spawn(async move {
                            if let Err(e) = telephony_service_clone
                                .call_owner(&phone_clone, &message_clone)
                                .await
                            {
                                tracing::error!("Failed to call owner {}: {}", phone_clone, e);
                            }
                        });

                        tracing::info!(
                            "Calling owner {} about block on {}",
                            phone,
                            block.blocked_plate
                        );
                        called = true;
                        break; // Звоним только первому найденному владельцу
                    }
                }
            }
        }

        if !called {
            tracing::warn!(
                "No owner found to call for block {} on plate {}",
                block_id,
                block.blocked_plate
            );
        }

        Ok(())
    }
}
