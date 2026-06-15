use crate::api::AppState;
use crate::auth::middleware::AuthState;
use crate::error::{AppError, AppResult};
use crate::utils::ocr::recognize_plate_from_image;
use axum::{
    extract::{Extension, Multipart, State},
    response::Json,
    routing::post,
    Router,
};
use serde_json::json;

pub fn ocr_router() -> Router<AppState> {
    Router::new().route("/recognize-plate-auth", post(recognize_plate_auth))
}

const MAX_OCR_IMAGE_BYTES: usize = 5 * 1024 * 1024;

/// Распознать номер по фото (требует авторизации)
#[utoipa::path(
    post,
    path = "/api/ocr/recognize-plate-auth",
    responses(
        (status = 200, description = "Результат распознавания"),
        (status = 401, description = "Не авторизован"),
    ),
    security(("bearer_token" = [])),
    tag = "ocr"
)]
pub async fn recognize_plate_auth(
    State(_state): State<AppState>,
    Extension(_auth_state): Extension<AuthState>,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    let mut image_data: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("Failed to read multipart field: {}", e)))?
    {
        if field.name() == Some("image") {
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::Validation(format!("Failed to read image data: {}", e)))?;
            if data.len() > MAX_OCR_IMAGE_BYTES {
                return Err(AppError::Validation(
                    "Image exceeds maximum size of 5 MB".to_string(),
                ));
            }
            image_data = Some(data.to_vec());
            break;
        }
    }

    let image_data =
        image_data.ok_or_else(|| AppError::Validation("Image field is required".to_string()))?;

    match recognize_plate_from_image(&image_data).await {
        Ok(plate) => Ok(Json(json!({
            "success": true,
            "plate": plate,
        }))),
        Err(e) => Ok(Json(json!({
            "success": false,
            "error": e.to_string(),
        }))),
    }
}
