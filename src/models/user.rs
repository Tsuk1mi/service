use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use serde_json::json;
use sqlx::FromRow;
use uuid::Uuid;
use utoipa::ToSchema;
use validator::Validate;

use crate::utils::{normalize_phone, normalize_plate};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum OwnerType {
    Owner,
    Renter,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub phone_encrypted: Option<String>,
    pub phone_hash: Option<String>,
    pub telegram: Option<String>,
    pub plate: Option<String>, // Может быть NULL, если пользователь еще не добавил номер
    pub name: Option<String>,
    pub show_contacts: bool,
    #[sqlx(default)]
    pub owner_type: Option<String>, // Храним как String для совместимости
    #[sqlx(default)]
    pub owner_info: Option<serde_json::Value>,
    #[sqlx(default)]
    pub departure_time: Option<chrono::NaiveTime>,
    #[sqlx(default)]
    pub push_token: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 1, max = 20, message = "Имя должно быть от 1 до 20 символов"))]
    pub name: Option<String>,
    
    pub phone: Option<String>,
    
    #[validate(length(max = 32, message = "Telegram username должен быть до 32 символов"))]
    pub telegram: Option<String>,
    
    pub plate: String,
    
    #[serde(default)]
    pub show_contacts: bool,
    
    #[serde(default)]
    pub owner_type: Option<String>, // "owner" или "renter"
    
    pub owner_info: Option<serde_json::Value>, // Информация о собственнике
}

#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({
    "name": "Иван Иванов",
    "telegram": "@ivan",
    "plate": "А123БВ777",
    "show_contacts": true,
    "owner_type": "renter",
    "departure_time": "08:00"
}))]
pub struct UpdateUserRequest {
    /// Имя пользователя
    #[schema(example = "Иван Иванов")]
    pub name: Option<String>,
    /// Номер телефона
    #[schema(example = "+79165180900")]
    pub phone: Option<String>,
    /// Telegram username
    #[schema(example = "@ivan")]
    pub telegram: Option<String>,
    /// Номер автомобиля
    #[schema(example = "А123БВ777")]
    pub plate: Option<String>,
    /// Показывать ли контакты другим пользователям
    #[schema(example = true)]
    pub show_contacts: Option<bool>,
    /// Тип владельца: "owner" или "renter"
    #[schema(example = "renter")]
    pub owner_type: Option<String>,
    /// Дополнительная информация о собственнике (для арендаторов)
    pub owner_info: Option<serde_json::Value>,
    /// Время выезда в формате HH:MM
    #[schema(example = "08:00")]
    pub departure_time: Option<String>,
    /// Push token устройства
    pub push_token: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    /// ID пользователя
    #[schema(value_type = String, format = "uuid", example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: Uuid,
    /// Имя пользователя
    #[schema(example = "Иван Иванов")]
    pub name: Option<String>,
    /// Номер телефона
    #[schema(example = "+79165180900")]
    pub phone: Option<String>,
    /// Telegram username
    #[schema(example = "@ivan")]
    pub telegram: Option<String>,
    /// Номер автомобиля
    #[schema(example = "А123БВ777")]
    pub plate: String,
    /// Показывать ли контакты
    #[schema(example = true)]
    pub show_contacts: bool,
    /// Тип владельца
    #[schema(example = "renter")]
    pub owner_type: Option<String>,
    /// Информация о собственнике
    pub owner_info: Option<serde_json::Value>,
    /// Время выезда
    #[schema(example = "08:00")]
    pub departure_time: Option<String>,
    /// Push token устройства
    pub push_token: Option<String>,
    /// Дата создания
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PublicUserInfo {
    /// ID пользователя
    #[schema(value_type = String, format = "uuid", example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: Uuid,
    /// Имя пользователя
    #[schema(example = "Иван Иванов")]
    pub name: Option<String>,
    /// Номер автомобиля
    #[schema(example = "А123БВ777")]
    pub plate: String,
    /// Номер телефона (только если show_contacts = true)
    #[schema(example = "+79165180900")]
    pub phone: Option<String>,
    /// Telegram username (только если show_contacts = true)
    #[schema(example = "@ivan")]
    pub telegram: Option<String>,
    /// Время выезда
    #[schema(example = "08:00")]
    pub departure_time: Option<String>,
}

impl User {
    pub fn to_response(&self, phone_decrypted: Option<String>) -> UserResponse {
        UserResponse {
            id: self.id,
            name: self.name.clone(),
            phone: phone_decrypted,
            telegram: self.telegram.clone(),
            plate: self.plate.clone().unwrap_or_default(),
            show_contacts: self.show_contacts,
            owner_type: self.owner_type.clone(),
            owner_info: self.owner_info.clone(),
            departure_time: self.departure_time.map(|t| t.format("%H:%M").to_string()),
            push_token: self.push_token.clone(),
            created_at: self.created_at,
        }
    }

    pub fn to_public_info(&self, phone_decrypted: Option<String>) -> PublicUserInfo {
        PublicUserInfo {
            id: self.id,
            name: self.name.clone(),
            plate: self.plate.clone().unwrap_or_default(),
            phone: if self.show_contacts { phone_decrypted } else { None },
            telegram: if self.show_contacts { self.telegram.clone() } else { None },
            departure_time: self.departure_time.map(|t| t.format("%H:%M").to_string()),
        }
    }
}

impl CreateUserRequest {
    pub fn normalize(&mut self) {
        if let Some(ref mut phone) = self.phone {
            *phone = normalize_phone(phone);
        }
        if let Some(ref mut telegram) = self.telegram {
            *telegram = telegram.trim().to_lowercase();
        }
        self.plate = normalize_plate(&self.plate);
    }
}

impl UpdateUserRequest {
    pub fn normalize(&mut self) {
        if let Some(ref mut phone) = self.phone {
            *phone = normalize_phone(phone);
        }
        if let Some(ref mut telegram) = self.telegram {
            *telegram = telegram.trim().to_lowercase();
        }
        if let Some(ref mut plate) = self.plate {
            *plate = normalize_plate(plate);
        }
    }
}

