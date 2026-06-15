use validator::Validate;

use crate::error::{AppError, AppResult};

/// Валидирует DTO с derive(Validate)
pub fn validate_payload<T: Validate>(payload: &T) -> AppResult<()> {
    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))
}
