use crate::error::{AppError, AppResult};
use crate::utils::{validate_phone as validate_phone_util, validate_plate as validate_plate_util, normalize_phone, normalize_plate};

/// Сервис валидации (SRP - Single Responsibility Principle)
pub struct ValidationService;

impl ValidationService {
    pub fn validate_phone(phone: &str) -> AppResult<String> {
        let normalized = normalize_phone(phone);
        if !validate_phone_util(&normalized) {
            return Err(AppError::Validation("Неверный формат номера телефона".to_string()));
        }
        Ok(normalized)
    }

    pub fn validate_plate(plate: &str) -> AppResult<String> {
        let normalized = normalize_plate(plate);
        if !validate_plate_util(&normalized) {
            return Err(AppError::Validation("Неверный формат номера автомобиля".to_string()));
        }
        Ok(normalized)
    }
}

