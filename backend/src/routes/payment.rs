use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

use crate::config::Config;
use crate::errors::AppError;
use crate::models::payment::*;
use crate::models::user::JwtClaims;
use crate::services::payment_service;

/// Конфигурация роутов платежей
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/payments")
            .route("", web::get().to(list_payments))
            .route("/create", web::post().to(create_payment))
            .route("/methods", web::get().to(list_payment_methods))
            .route("/methods", web::post().to(add_payment_method))
            .route("/methods/{method_id}", web::delete().to(remove_payment_method))
            .route("/webhook/yookassa", web::post().to(yookassa_webhook))
    );
}

/// GET /api/v1/payments
/// История платежей пользователя
async fn list_payments(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
) -> Result<HttpResponse, AppError> {
    let payments = sqlx::query_as::<_, PaymentPublic>(
        r#"
        SELECT 
            p.id,
            a.display_name as author_name,
            p.payment_type,
            p.amount_kopecks,
            p.status,
            p.payment_method,
            pm.card_last_four,
            p.created_at,
            p.completed_at
        FROM payments p
        JOIN authors a ON p.author_id = a.id
        LEFT JOIN payment_methods pm ON p.id = pm.id
        WHERE p.user_id = $1
        ORDER BY p.created_at DESC
        LIMIT 50
        "#
    )
    .bind(claims.sub)
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(payments))
}

/// POST /api/v1/payments/create
/// Создание платежа (инициация оплаты через ЮKassa)
async fn create_payment(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    claims: web::ReqData<JwtClaims>,
    body: web::Json<CreatePaymentRequest>,
) -> Result<HttpResponse, AppError> {
    // Получение информации о плане
    let plan = sqlx::query_as::<_, PlanForPayment>(
        "SELECT id, author_id, plan_type, title, price_monthly_kopecks FROM plans WHERE id = $1 AND is_enabled = true"
    )
    .bind(body.plan_id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Тариф не найден".to_string()))?;

    // Расчёт комиссии платформы
    let amount = plan.price_monthly_kopecks;
    let commission = (amount as f64 * config.platform_commission_percent / 100.0) as i32;
    let author_amount = amount - commission;

    let payment_id = Uuid::new_v4();
    let idempotency_key = Uuid::new_v4().to_string();

    // Создание записи платежа
    sqlx::query(
        r#"
        INSERT INTO payments (
            id, user_id, author_id, subscription_id, payment_type,
            amount_kopecks, commission_kopecks, author_amount_kopecks,
            currency, status, payment_method, description, idempotency_key,
            created_at
        )
        VALUES ($1, $2, $3, $4, 'subscription', $5, $6, $7, 'RUB', 'pending', $8, $9, $10, NOW())
        "#
    )
    .bind(payment_id)
    .bind(claims.sub)
    .bind(plan.author_id)
    .bind(body.subscription_id)
    .bind(amount)
    .bind(commission)
    .bind(author_amount)
    .bind(&body.payment_method_type)
    .bind(format!("Подписка: {} — {}", plan.title, plan.author_id))
    .bind(&idempotency_key)
    .execute(pool.get_ref())
    .await?;

    // Создание платежа в ЮKassa
    let yookassa_response = payment_service::create_yookassa_payment(
        &config,
        payment_id,
        amount,
        &body.return_url,
        body.save_card,
    ).await?;

    // Если нужно сохранить карту — создаём запись payment_method
    if body.save_card {
        // В реальном коде — сохранение данных карты из ответа ЮKassa
    }

    Ok(HttpResponse::Created().json(PaymentResponse {
        payment_id,
        confirmation_url: yookassa_response.confirmation_url,
        amount_kopecks: amount,
        status: PaymentStatus::Pending,
    }))
}

/// GET /api/v1/payments/methods
/// Список сохранённых способов оплаты
async fn list_payment_methods(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
) -> Result<HttpResponse, AppError> {
    let methods = sqlx::query_as::<_, PaymentMethod>(
        "SELECT * FROM payment_methods WHERE user_id = $1 AND is_active = true ORDER BY is_default DESC, created_at DESC"
    )
    .bind(claims.sub)
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(methods))
}

/// POST /api/v1/payments/methods
/// Добавление способа оплаты
async fn add_payment_method(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
    body: web::Json<AddPaymentMethodRequest>,
) -> Result<HttpResponse, AppError> {
    let method_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO payment_methods (id, user_id, method_type, card_last_four, card_expiry, is_default, is_active, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, true, NOW())
        "#
    )
    .bind(method_id)
    .bind(claims.sub)
    .bind(&body.method_type)
    .bind(&body.card_last_four)
    .bind(&body.card_expiry)
    .bind(body.is_default.unwrap_or(false))
    .execute(pool.get_ref())
    .await?;

    Ok(HttpResponse::Created().json(serde_json::json!({
        "id": method_id,
        "message": "Способ оплаты добавлен"
    })))
}

/// DELETE /api/v1/payments/methods/{method_id}
/// Удаление способа оплаты
async fn remove_payment_method(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let method_id = path.into_inner();

    let result = sqlx::query(
        "UPDATE payment_methods SET is_active = false WHERE id = $1 AND user_id = $2"
    )
    .bind(method_id)
    .bind(claims.sub)
    .execute(pool.get_ref())
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Способ оплаты не найден".to_string()));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Способ оплаты удалён" })))
}

/// POST /api/v1/payments/webhook/yookassa
/// Webhook от ЮKassa о статусе платежа
async fn yookassa_webhook(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    req: actix_web::HttpRequest,
    body: web::Bytes,
) -> Result<HttpResponse, AppError> {
    // FIX: HMAC-верификация webhook от ЮKassa
    let signature = req.headers()
        .get("X-Yoo-Signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::AuthError("Missing webhook signature".to_string()))?;

    let body_str = std::str::from_utf8(&body)
        .map_err(|e| AppError::ValidationError(format!("Invalid UTF-8: {}", e)))?;

    payment_service::verify_yookassa_webhook(
        body_str,
        signature,
        &config.yookassa_webhook_secret,
    )?;

    let webhook: payment_service::YookassaWebhook = serde_json::from_str(body_str)
        .map_err(|e| AppError::ValidationError(format!("Invalid webhook payload: {}", e)))?;

    let body = webhook;

    match body.event.as_str() {
        "payment.succeeded" => {
            // Обновление статуса платежа
            let payment_id: Uuid = sqlx::query_scalar(
                "SELECT id FROM payments WHERE external_id = $1"
            )
            .bind(&body.payment_id)
            .fetch_optional(pool.get_ref())
            .await?
            .ok_or_else(|| AppError::NotFound("Платёж не найден".to_string()))?;

            sqlx::query(
                "UPDATE payments SET status = 'succeeded', completed_at = NOW() WHERE id = $1"
            )
            .bind(payment_id)
            .execute(pool.get_ref())
            .await?;

            // Активация подписки (если это оплата подписки)
            let subscription_id: Option<Uuid> = sqlx::query_scalar(
                "SELECT subscription_id FROM payments WHERE id = $1"
            )
            .bind(payment_id)
            .fetch_optional(pool.get_ref())
            .await?;

            if let Some(sub_id) = subscription_id {
                sqlx::query(
                    "UPDATE subscriptions SET status = 'active' WHERE id = $1 AND status = 'pending'"
                )
                .bind(sub_id)
                .execute(pool.get_ref())
                .await?;
            }
        }
        "payment.canceled" | "payment.cancelled" => {
            let payment_id: Uuid = sqlx::query_scalar(
                "SELECT id FROM payments WHERE external_id = $1"
            )
            .bind(&body.payment_id)
            .fetch_optional(pool.get_ref())
            .await?
            .ok_or_else(|| AppError::NotFound("Платёж не найден".to_string()))?;

            sqlx::query(
                "UPDATE payments SET status = 'cancelled' WHERE id = $1"
            )
            .bind(payment_id)
            .execute(pool.get_ref())
            .await?;
        }
        "payment.waiting_for_capture" => {
            // Для двухстадийной оплаты — автоматический capture
            payment_service::capture_yookassa_payment(&config, &body.payment_id).await?;
        }
        _ => {
            log::warn!("Unknown webhook event: {}", body.event);
        }
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "ok" })))
}

#[derive(serde::Deserialize)]
struct AddPaymentMethodRequest {
    method_type: PaymentMethodType,
    card_last_four: String,
    card_expiry: String,
    is_default: Option<bool>,
}

#[derive(sqlx::FromRow)]
struct PlanForPayment {
    id: Uuid,
    author_id: Uuid,
    plan_type: crate::models::author::PlanType,
    title: String,
    price_monthly_kopecks: i32,
}
