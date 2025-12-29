use crate::error::{AppError, AppResult};
use base64::Engine;

/// Распознаёт номер автомобиля с изображения
pub async fn recognize_plate_from_image(image_data: &[u8]) -> AppResult<String> {
    // Пробуем использовать внешний OCR сервис, если настроен
    if let Ok(ocr_api_url) = std::env::var("OCR_API_URL") {
        return recognize_via_api(&ocr_api_url, image_data).await;
    }

    // Fallback: возвращаем ошибку, чтобы пользователь вводил номер вручную
    // В реальном приложении здесь должен быть полноценный OCR (например, через Tesseract или ML модель)
    Err(AppError::Internal(
        "OCR not configured. Please set OCR_API_URL environment variable or enter plate manually"
            .to_string(),
    ))
}

/// Распознавание через внешний API
async fn recognize_via_api(api_url: &str, image_data: &[u8]) -> AppResult<String> {
    let client = reqwest::Client::new();
    let base64_image = base64::engine::general_purpose::STANDARD.encode(image_data);

    let response = client
        .post(api_url)
        .json(&serde_json::json!({
            "image": base64_image,
            "type": "license_plate"
        }))
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("OCR API request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(AppError::Internal(format!(
            "OCR API returned error: {}",
            response.status()
        )));
    }

    let result: serde_json::Value = response
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to parse OCR response: {}", e)))?;

    let plate = result["plate"]
        .as_str()
        .ok_or_else(|| AppError::Internal("OCR API did not return plate".to_string()))?;

    Ok(plate.to_string())
}
