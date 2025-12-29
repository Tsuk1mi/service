use chrono::{DateTime, Utc};
use chrono::NaiveTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

use crate::utils::normalize_plate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserPlate {
    pub id: Uuid,
    pub user_id: Uuid,
    pub plate: String,
    pub is_primary: bool,
    pub departure_time: Option<NaiveTime>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserPlateRequest {
    #[validate(length(min = 8, max = 9, message = "Номер автомобиля должен быть от 8 до 9 символов"))]
    pub plate: String,
    pub is_primary: Option<bool>,
    pub departure_time: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserPlateRequest {
    pub departure_time: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserPlateResponse {
    pub id: String,
    pub user_id: String,
    pub plate: String,
    pub is_primary: bool,
    pub departure_time: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl UserPlate {
    pub fn to_response(&self) -> UserPlateResponse {
        UserPlateResponse {
            id: self.id.to_string(),
            user_id: self.user_id.to_string(),
            plate: self.plate.clone(),
            is_primary: self.is_primary,
            departure_time: self.departure_time.map(|t| t.format("%H:%M").to_string()),
            created_at: self.created_at.to_rfc3339(),
            updated_at: self.updated_at.to_rfc3339(),
        }
    }
}

impl CreateUserPlateRequest {
    pub fn normalize(&mut self) {
        self.plate = normalize_plate(&self.plate);
    }
}

