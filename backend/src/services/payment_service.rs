use hmac::{Hmac, Mac};
use sha2::Sha256;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::Config;
use crate::errors::AppError;

type HmacSha256 = Hmac<Sha256>;

/// Ответ от ЮKassa при создании платежа
#[derive(Debug, Deserialize)]
pub struct YookassaPaymentResponse {
    pub id: String,
    pub status: String,
    pub amount: YookassaAmount,
    pub confirmation: YookassaConfirmation,
    pub description: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct YookassaAmount {
    pub value: String,
    pub currency: String,
}

#[derive(Debug, Deserialize)]
pub struct YookassaConfirmation {
    pub r#type: String,
    pub confirmation_url: String,
}

/// FIX: Webhook от ЮKassa с HMAC-верификацией
#[derive(Debug, Deserialize)]
pub struct YookassaWebhook {
    pub event: String,
    pub r#type: String,
    pub object: YookassaWebhookObject,
}

#[derive(Debug, Deserialize)]
pub struct YookassaWebhookObject {
    pub id: String,
    pub status: String,
    pub amount: YookassaAmount,
    pub metadata: Option<serde_json::Value>,
}

/// FIX: HMAC-верификация webhook от ЮKassa
/// ЮKassa отправляет webhook с заголовком X-Yoo-Signature
pub fn verify_yookassa_webhook(
    body: &str,
    signature: &str,
    webhook_secret: &str,
) -> Result<(), AppError> {
    let mut mac = HmacSha256::new_from_slice(webhook_secret.as_bytes())
        .map_err(|e| AppError::InternalError(format!("HMAC error: {}", e)))?;

    mac.update(body.as_bytes());

    let expected_signature = mac.finalize().into_bytes();
    let expected_hex = hex::encode(expected_signature);

    // Сравнение с constant-time для защиты от timing attacks
    if !constant_time_eq(signature.as_bytes(), expected_hex.as_bytes()) {
        log::warn!("Webhook signature mismatch");
        return Err(AppError::AuthError("Invalid webhook signature".to_string()));
    }

    Ok(())
}

/// Constant-time comparison для защиты от timing attacks
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }

    result == 0
}

/// Запрос к ЮKassa для создания платежа
#[derive(Debug, Serialize)]
struct YookassaCreatePayment {
    amount: YookassaAmount,
    confirmation: YookassaConfirmationRequest,
    description: String,
    metadata: serde_json::Value,
    capture: bool,
    save_payment_method: bool,
}

#[derive(Debug, Serialize)]
struct YookassaConfirmationRequest {
    r#type: String,
    return_url: String,
}

/// Создание платежа в ЮKassa
pub async fn create_yookassa_payment(
    config: &Config,
    payment_id: Uuid,
    amount_kopecks: i32,
    return_url: &str,
    save_card: bool,
) -> Result<YookassaPaymentResponse, AppError> {
    let amount_rubles = format!("{:.2}", amount_kopecks as f64 / 100.0);

    let request_body = YookassaCreatePayment {
        amount: YookassaAmount {
            value: amount_rubles,
            currency: "RUB".to_string(),
        },
        confirmation: YookassaConfirmationRequest {
            r#type: "redirect".to_string(),
            return_url: return_url.to_string(),
        },
        description: format!("Оплата подписки на Чистовик"),
        metadata: serde_json::json!({
            "payment_id": payment_id.to_string(),
            "platform": "chistovik",
        }),
        capture: true,
        save_payment_method: save_card,
    };

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.yookassa.ru/v3/payments")
        .header("Idempotence-Key", payment_id.to_string())
        .basic_auth(&config.yookassa_shop_id, Some(&config.yookassa_secret_key))
        .json(&request_body)
        .send()
        .await
        .map_err(|e| AppError::PaymentError(format!("Ошибка запроса к ЮKassa: {}", e)))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(AppError::PaymentError(format!("ЮKassa ошибка: {}", error_text)));
    }

    let payment_response: YookassaPaymentResponse = response
        .json()
        .await
        .map_err(|e| AppError::PaymentError(format!("Ошибка парсинга ответа: {}", e)))?;

    Ok(payment_response)
}

/// Capture платежа (для двухстадийной оплаты)
pub async fn capture_yookassa_payment(
    config: &Config,
    yookassa_payment_id: &str,
) -> Result<(), AppError> {
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("https://api.yookassa.ru/v3/payments/{}/capture", yookassa_payment_id))
        .basic_auth(&config.yookassa_shop_id, Some(&config.yookassa_secret_key))
        .json(&serde_json::json!({
            "amount": {
                "value": "0.00",
                "currency": "RUB"
            }
        }))
        .send()
        .await
        .map_err(|e| AppError::PaymentError(format!("Ошибка capture: {}", e)))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(AppError::PaymentError(format!("Capture ошибка: {}", error_text)));
    }

    Ok(())
}

/// Создание возврата в ЮKassa
pub async fn create_refund(
    config: &Config,
    yookassa_payment_id: &str,
    amount_kopecks: Option<i32>,
) -> Result<String, AppError> {
    let mut body = serde_json::json!({
        "payment_id": yookassa_payment_id,
    });

    if let Some(amount) = amount_kopecks {
        body["amount"] = serde_json::json!({
            "value": format!("{:.2}", amount as f64 / 100.0),
            "currency": "RUB"
        });
    }

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.yookassa.ru/v3/refunds")
        .basic_auth(&config.yookassa_shop_id, Some(&config.yookassa_secret_key))
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::PaymentError(format!("Ошибка возврата: {}", e)))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(AppError::PaymentError(format!("Возврат ошибка: {}", error_text)));
    }

    let refund: serde_json::Value = response
        .json()
        .await
        .map_err(|e| AppError::PaymentError(format!("Ошибка парсинга: {}", e)))?;

    Ok(refund["id"].as_str().unwrap_or("unknown").to_string())
}

/// Обработка автоматического списания (рекуррентный платёж)
pub async fn process_recurring_payment(
    config: &Config,
    recurring_token: &str,
    amount_kopecks: i32,
    description: &str,
) -> Result<YookassaPaymentResponse, AppError> {
    let amount_rubles = format!("{:.2}", amount_kopecks as f64 / 100.0);

    let request_body = serde_json::json!({
        "amount": {
            "value": amount_rubles,
            "currency": "RUB"
        },
        "description": description,
        "payment_method_id": recurring_token,
        "capture": true,
    });

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.yookassa.ru/v3/payments")
        .basic_auth(&config.yookassa_shop_id, Some(&config.yookassa_secret_key))
        .json(&request_body)
        .send()
        .await
        .map_err(|e| AppError::PaymentError(format!("Ошибка рекуррентного платежа: {}", e)))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(AppError::PaymentError(format!("Рекуррент ошибка: {}", error_text)));
    }

    let payment_response: YookassaPaymentResponse = response
        .json()
        .await
        .map_err(|e| AppError::PaymentError(format!("Ошибка парсинга: {}", e)))?;

    Ok(payment_response)
}

/// Обработка grace period (льготный период после неудачного платежа)
pub async fn handle_failed_renewal(
    pool: &sqlx::PgPool,
    subscription_id: Uuid,
) -> Result<(), AppError> {
    // Увеличение счётчика неудачных попыток
    let failures: i32 = sqlx::query_scalar(
        "UPDATE subscriptions SET consecutive_failures = consecutive_failures + 1, status = 'grace_period', grace_period_end = NOW() + INTERVAL '5 days', updated_at = NOW() WHERE id = $1 RETURNING consecutive_failures"
    )
    .bind(subscription_id)
    .fetch_one(pool)
    .await?;

    // Если слишком много неудачных попыток — блокируем подписку
    if failures >= 3 {
        sqlx::query(
            "UPDATE subscriptions SET status = 'expired', updated_at = NOW() WHERE id = $1"
        )
        .bind(subscription_id)
        .execute(pool)
        .await?;
    }

    Ok(())
}
