use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{Utc, Duration};

use crate::errors::AppError;
use crate::models::subscription::*;
use crate::models::user::JwtClaims;

/// Конфигурация роутов подписок
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/subscriptions")
            .route("", web::get().to(list_subscriptions))
            .route("", web::post().to(create_subscription))
            .route("/{subscription_id}", web::get().to(get_subscription))
            .route("/{subscription_id}/modify", web::post().to(modify_subscription))
            .route("/{subscription_id}/pause", web::post().to(pause_subscription))
            .route("/{subscription_id}/resume", web::post().to(resume_subscription))
            .route("/{subscription_id}/cancel", web::post().to(cancel_subscription))
            .route("/check-access/{author_id}", web::get().to(check_access))
    );
}

/// GET /api/v1/subscriptions
/// Список подписок пользователя
async fn list_subscriptions(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
) -> Result<HttpResponse, AppError> {
    let subscriptions = sqlx::query_as::<_, SubscriptionPublic>(
        r#"
        SELECT 
            s.id, s.author_id,
            a.display_name as author_name,
            a.avatar_url as author_avatar_url,
            a.slug as author_slug,
            s.plan_type,
            p.title as plan_title,
            s.status,
            s.price_kopecks,
            s.next_billing_date,
            s.auto_renew,
            s.current_period_end
        FROM subscriptions s
        JOIN authors a ON s.author_id = a.id
        JOIN plans p ON s.plan_id = p.id
        WHERE s.user_id = $1
        ORDER BY s.created_at DESC
        "#
    )
    .bind(claims.sub)
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(subscriptions))
}

/// POST /api/v1/subscriptions
/// Оформление подписки
async fn create_subscription(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
    body: web::Json<CreateSubscriptionRequest>,
) -> Result<HttpResponse, AppError> {
    // Проверка, что план существует и активен
    let plan = sqlx::query_as::<_, PlanCheck>(
        "SELECT id, author_id, plan_type, price_monthly_kopecks, price_quarterly_kopecks, price_yearly_kopecks, is_enabled FROM plans WHERE id = $1"
    )
    .bind(body.plan_id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Тариф не найден".to_string()))?;

    if !plan.is_enabled {
        return Err(AppError::ValidationError("Тариф недоступен".to_string()));
    }

    // Проверка, что у пользователя нет активной подписки на этот план
    let existing = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM subscriptions WHERE user_id = $1 AND author_id = $2 AND plan_type = $3 AND status = 'active'"
    )
    .bind(claims.sub)
    .bind(body.author_id)
    .bind(&plan.plan_type)
    .fetch_optional(pool.get_ref())
    .await?;

    if existing.is_some() {
        return Err(AppError::ValidationError(
            "У вас уже есть активная подписка на этот модуль".to_string()
        ));
    }

    // Определение цены по периоду
    let price = match body.period {
        SubscriptionPeriod::Monthly => plan.price_monthly_kopecks,
        SubscriptionPeriod::Quarterly => plan.price_quarterly_kopecks
            .ok_or_else(|| AppError::ValidationError("Квартальный период не доступен".to_string()))?,
        SubscriptionPeriod::Yearly => plan.price_yearly_kopecks
            .ok_or_else(|| AppError::ValidationError("Годовой период не доступен".to_string()))?,
    };

    let now = Utc::now();
    let period_end = match body.period {
        SubscriptionPeriod::Monthly => now + Duration::days(30),
        SubscriptionPeriod::Quarterly => now + Duration::days(90),
        SubscriptionPeriod::Yearly => now + Duration::days(365),
    };

    let subscription_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO subscriptions (
            id, user_id, author_id, plan_id, plan_type, status, period,
            price_kopecks, current_period_start, current_period_end,
            next_billing_date, auto_renew, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, 'active', $6, $7, $8, $9, $9, true, $10, $10)
        "#
    )
    .bind(subscription_id)
    .bind(claims.sub)
    .bind(body.author_id)
    .bind(body.plan_id)
    .bind(&plan.plan_type)
    .bind(&body.period)
    .bind(price)
    .bind(now)
    .bind(period_end)
    .bind(now)
    .execute(pool.get_ref())
    .await?;

    // Обновление счётчика подписчиков
    sqlx::query(
        "UPDATE authors SET total_subscribers = total_subscribers + 1 WHERE id = $1"
    )
    .bind(body.author_id)
    .execute(pool.get_ref())
    .await?;

    sqlx::query(
        "UPDATE plans SET subscribers_count = subscribers_count + 1 WHERE id = $1"
    )
    .bind(body.plan_id)
    .execute(pool.get_ref())
    .await?;

    Ok(HttpResponse::Created().json(serde_json::json!({
        "subscription_id": subscription_id,
        "price_kopecks": price,
        "period_end": period_end,
        "message": "Подписка оформлена. Перейдите к оплате."
    })))
}

/// GET /api/v1/subscriptions/{subscription_id}
/// Получение информации о подписке
async fn get_subscription(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let subscription_id = path.into_inner();

    let subscription = sqlx::query_as::<_, Subscription>(
        "SELECT * FROM subscriptions WHERE id = $1 AND user_id = $2"
    )
    .bind(subscription_id)
    .bind(claims.sub)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Подписка не найдена".to_string()))?;

    Ok(HttpResponse::Ok().json(subscription))
}

/// POST /api/v1/subscriptions/{subscription_id}/modify
/// Изменение подписки (upgrade/downgrade)
async fn modify_subscription(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
    path: web::Path<Uuid>,
    body: web::Json<ModifySubscriptionRequest>,
) -> Result<HttpResponse, AppError> {
    let subscription_id = path.into_inner();

    let subscription = sqlx::query_as::<_, Subscription>(
        "SELECT * FROM subscriptions WHERE id = $1 AND user_id = $2"
    )
    .bind(subscription_id)
    .bind(claims.sub)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Подписка не найдена".to_string()))?;

    match body.action {
        SubscriptionAction::Upgrade => {
            let new_plan_id = body.new_plan_id
                .ok_or_else(|| AppError::ValidationError("Укажите новый план".to_string()))?;
            
            // Расчёт перерасчёта (proration)
            let remaining_days = (subscription.current_period_end - Utc::now()).num_days() as i32;
            let old_daily_price = subscription.price_kopecks / 30;
            
            let new_price: i32 = sqlx::query_scalar(
                "SELECT price_monthly_kopecks FROM plans WHERE id = $1"
            )
            .bind(new_plan_id)
            .fetch_optional(pool.get_ref())
            .await?
            .ok_or_else(|| AppError::NotFound("Новый план не найден".to_string()))?;
            
            let new_daily_price = new_price / 30;
            let credit = old_daily_price * remaining_days;
            let charge = new_daily_price * remaining_days;
            let net = charge - credit;

            // Обновление подписки
            sqlx::query(
                "UPDATE subscriptions SET plan_id = $1, price_kopecks = $2, updated_at = NOW() WHERE id = $3"
            )
            .bind(new_plan_id)
            .bind(new_price)
            .bind(subscription_id)
            .execute(pool.get_ref())
            .await?;

            let proration = ProrationResult {
                remaining_days,
                old_plan_daily_price_kopecks: old_daily_price,
                new_plan_daily_price_kopecks: new_daily_price,
                credit_kopecks: credit,
                charge_kopecks: charge,
                net_amount_kopecks: net,
            };

            Ok(HttpResponse::Ok().json(serde_json::json!({
                "message": "Подписка обновлена",
                "proration": proration,
                "additional_charge_kopecks": net.max(0),
            })))
        }
        SubscriptionAction::Cancel => {
            sqlx::query(
                "UPDATE subscriptions SET status = 'cancelled', cancelled_at = NOW(), auto_renew = false, updated_at = NOW() WHERE id = $1"
            )
            .bind(subscription_id)
            .execute(pool.get_ref())
            .await?;

            Ok(HttpResponse::Ok().json(serde_json::json!({
                "message": "Подписка отменена. Доступ сохранится до конца оплаченного периода.",
                "access_until": subscription.current_period_end,
            })))
        }
        SubscriptionAction::ToggleAutoRenew => {
            let new_auto_renew = !subscription.auto_renew;
            sqlx::query(
                "UPDATE subscriptions SET auto_renew = $1, updated_at = NOW() WHERE id = $2"
            )
            .bind(new_auto_renew)
            .bind(subscription_id)
            .execute(pool.get_ref())
            .await?;

            Ok(HttpResponse::Ok().json(serde_json::json!({
                "auto_renew": new_auto_renew,
                "message": if new_auto_renew { "Автопродление включено" } else { "Автопродление выключено" },
            })))
        }
        _ => {
            Err(AppError::ValidationError("Действие не поддерживается".to_string()))
        }
    }
}

/// POST /api/v1/subscriptions/{subscription_id}/pause
/// Заморозка подписки
async fn pause_subscription(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
    path: web::Path<Uuid>,
    body: web::Json<PauseSubscriptionRequest>,
) -> Result<HttpResponse, AppError> {
    if body.pause_months < 1 || body.pause_months > 3 {
        return Err(AppError::ValidationError(
            "Заморозка возможна на 1-3 месяца".to_string()
        ));
    }

    let subscription_id = path.into_inner();
    let now = Utc::now();
    let pause_end = now + Duration::days(30 * body.pause_months as i64);

    sqlx::query(
        r#"
        UPDATE subscriptions 
        SET status = 'paused', paused_at = $1, pause_end_at = $2, updated_at = NOW()
        WHERE id = $3 AND user_id = $4 AND status = 'active'
        "#
    )
    .bind(now)
    .bind(pause_end)
    .bind(subscription_id)
    .bind(claims.sub)
    .execute(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": format!("Подписка заморожена до {}", pause_end.format("%d.%m.%Y")),
        "pause_end": pause_end,
    })))
}

/// POST /api/v1/subscriptions/{subscription_id}/resume
/// Разморозка подписки
async fn resume_subscription(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let subscription_id = path.into_inner();

    let subscription = sqlx::query_as::<_, Subscription>(
        "SELECT * FROM subscriptions WHERE id = $1 AND user_id = $2 AND status = 'paused'"
    )
    .bind(subscription_id)
    .bind(claims.sub)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Замороженная подписка не найдена".to_string()))?;

    let now = Utc::now();
    let remaining_pause = subscription.pause_end_at.unwrap_or(now) - now;
    let new_period_end = subscription.current_period_end + remaining_pause;

    sqlx::query(
        r#"
        UPDATE subscriptions 
        SET status = 'active', paused_at = NULL, pause_end_at = NULL,
            current_period_end = $1, next_billing_date = $1, updated_at = NOW()
        WHERE id = $2
        "#
    )
    .bind(new_period_end)
    .bind(subscription_id)
    .execute(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Подписка разморожена",
        "new_period_end": new_period_end,
    })))
}

/// POST /api/v1/subscriptions/{subscription_id}/cancel
/// Отмена подписки
async fn cancel_subscription(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let subscription_id = path.into_inner();

    let subscription = sqlx::query_as::<_, Subscription>(
        "SELECT * FROM subscriptions WHERE id = $1 AND user_id = $2 AND status IN ('active', 'grace_period')"
    )
    .bind(subscription_id)
    .bind(claims.sub)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Подписка не найдена".to_string()))?;

    sqlx::query(
        "UPDATE subscriptions SET status = 'cancelled', cancelled_at = NOW(), auto_renew = false, updated_at = NOW() WHERE id = $1"
    )
    .bind(subscription_id)
    .execute(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Подписка отменена",
        "access_until": subscription.current_period_end,
    })))
}

/// GET /api/v1/subscriptions/check-access/{author_id}
/// Проверка доступа к контенту автора
async fn check_access(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let author_id = path.into_inner();

    let active_sub = sqlx::query_as::<_, Subscription>(
        r#"
        SELECT * FROM subscriptions 
        WHERE user_id = $1 AND author_id = $2 
        AND status IN ('active', 'grace_period')
        AND current_period_end > NOW()
        ORDER BY created_at DESC
        LIMIT 1
        "#
    )
    .bind(claims.sub)
    .bind(author_id)
    .fetch_optional(pool.get_ref())
    .await?;

    match active_sub {
        Some(sub) => {
            Ok(HttpResponse::Ok().json(SubscriptionAccessCheck {
                has_access: true,
                subscription_id: Some(sub.id),
                plan_type: Some(sub.plan_type.clone()),
                access_until: Some(sub.current_period_end),
            }))
        }
        None => {
            Ok(HttpResponse::Ok().json(SubscriptionAccessCheck {
                has_access: false,
                subscription_id: None,
                plan_type: None,
                access_until: None,
            }))
        }
    }
}

#[derive(serde::Serialize)]
struct SubscriptionAccessCheck {
    has_access: bool,
    subscription_id: Option<Uuid>,
    plan_type: Option<crate::models::author::PlanType>,
    access_until: Option<chrono::DateTime<Utc>>,
}

#[derive(sqlx::FromRow)]
struct PlanCheck {
    id: Uuid,
    author_id: Uuid,
    plan_type: crate::models::author::PlanType,
    price_monthly_kopecks: i32,
    price_quarterly_kopecks: Option<i32>,
    price_yearly_kopecks: Option<i32>,
    is_enabled: bool,
}
