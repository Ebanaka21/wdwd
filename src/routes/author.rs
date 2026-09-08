use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::author::*;
use crate::models::user::JwtClaims;

/// Конфигурация роутов автора (защищённые)
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/author")
            .route("/profile", web::get().to(get_author_profile))
            .route("/profile", web::put().to(update_author_profile))
            .route("/plans", web::get().to(get_plans))
            .route("/plans", web::post().to(create_plan))
            .route("/plans/{plan_id}", web::put().to(update_plan))
            .route("/plans/{plan_id}", web::delete().to(delete_plan))
            .route("/stats", web::get().to(get_stats))
            .route("/payouts", web::get().to(get_payouts))
            .route("/payouts", web::post().to(request_payout))
    );
}

/// GET /api/v1/author/profile
/// Получение профиля автора
async fn get_author_profile(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
) -> Result<HttpResponse, AppError> {
    let author = sqlx::query_as::<_, Author>(
        "SELECT * FROM authors WHERE user_id = $1"
    )
    .bind(claims.sub)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Профиль автора не найден".to_string()))?;

    Ok(HttpResponse::Ok().json(author))
}

/// PUT /api/v1/author/profile
/// Обновление профиля автора
async fn update_author_profile(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
    body: web::Json<UpdateAuthorRequest>,
) -> Result<HttpResponse, AppError> {
    let author = sqlx::query_as::<_, Author>(
        "SELECT * FROM authors WHERE user_id = $1"
    )
    .bind(claims.sub)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Профиль автора не найден".to_string()))?;

    sqlx::query(
        r#"
        UPDATE authors SET
            display_name = COALESCE($1, display_name),
            description = COALESCE($2, description),
            avatar_url = COALESCE($3, avatar_url),
            cover_url = COALESCE($4, cover_url),
            telegram_url = COALESCE($5, telegram_url),
            vk_url = COALESCE($6, vk_url),
            youtube_url = COALESCE($7, youtube_url),
            updated_at = NOW()
        WHERE id = $8
        "#
    )
    .bind(&body.display_name)
    .bind(&body.description)
    .bind(&body.avatar_url)
    .bind(&body.cover_url)
    .bind(&body.telegram_url)
    .bind(&body.vk_url)
    .bind(&body.youtube_url)
    .bind(author.id)
    .execute(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Профиль обновлён" })))
}

/// GET /api/v1/author/plans
/// Получение тарифных планов автора
async fn get_plans(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
) -> Result<HttpResponse, AppError> {
    let author = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM authors WHERE user_id = $1"
    )
    .bind(claims.sub)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Автор не найден".to_string()))?;

    let plans = sqlx::query_as::<_, Plan>(
        "SELECT * FROM plans WHERE author_id = $1 ORDER BY created_at ASC"
    )
    .bind(author)
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(plans))
}

/// POST /api/v1/author/plans
/// Создание нового тарифного плана
async fn create_plan(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
    body: web::Json<CreatePlanRequest>,
) -> Result<HttpResponse, AppError> {
    let author_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM authors WHERE user_id = $1"
    )
    .bind(claims.sub)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Автор не найден".to_string()))?;

    let plan_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO plans (id, author_id, plan_type, title, description, 
                          price_monthly_kopecks, price_quarterly_kopecks, 
                          price_yearly_kopecks, is_enabled, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true, NOW(), NOW())
        "#
    )
    .bind(plan_id)
    .bind(author_id)
    .bind(&body.plan_type)
    .bind(&body.title)
    .bind(&body.description)
    .bind(body.price_monthly_kopecks)
    .bind(body.price_quarterly_kopecks)
    .bind(body.price_yearly_kopecks)
    .execute(pool.get_ref())
    .await?;

    Ok(HttpResponse::Created().json(serde_json::json!({
        "id": plan_id,
        "message": "Тариф создан"
    })))
}

/// PUT /api/v1/author/plans/{plan_id}
/// Обновление тарифного плана
async fn update_plan(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
    path: web::Path<Uuid>,
    body: web::Json<UpdatePlanRequest>,
) -> Result<HttpResponse, AppError> {
    let plan_id = path.into_inner();

    // Проверка, что план принадлежит этому автору
    let author_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM authors WHERE user_id = $1"
    )
    .bind(claims.sub)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Автор не найден".to_string()))?;

    let plan_author = sqlx::query_scalar::<_, Uuid>(
        "SELECT author_id FROM plans WHERE id = $1"
    )
    .bind(plan_id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Тариф не найден".to_string()))?;

    if plan_author != author_id {
        return Err(AppError::ForbiddenError("Нет доступа к этому тарифу".to_string()));
    }

    sqlx::query(
        r#"
        UPDATE plans SET
            title = COALESCE($1, title),
            description = COALESCE($2, description),
            price_monthly_kopecks = COALESCE($3, price_monthly_kopecks),
            price_quarterly_kopecks = COALESCE($4, price_quarterly_kopecks),
            price_yearly_kopecks = COALESCE($5, price_yearly_kopecks),
            is_enabled = COALESCE($6, is_enabled),
            updated_at = NOW()
        WHERE id = $7
        "#
    )
    .bind(&body.title)
    .bind(&body.description)
    .bind(body.price_monthly_kopecks)
    .bind(body.price_quarterly_kopecks)
    .bind(body.price_yearly_kopecks)
    .bind(body.is_enabled)
    .bind(plan_id)
    .execute(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Тариф обновлён" })))
}

/// DELETE /api/v1/author/plans/{plan_id}
/// Удаление тарифного плана
async fn delete_plan(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let plan_id = path.into_inner();

    let author_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM authors WHERE user_id = $1"
    )
    .bind(claims.sub)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Автор не найден".to_string()))?;

    let result = sqlx::query(
        "DELETE FROM plans WHERE id = $1 AND author_id = $2"
    )
    .bind(plan_id)
    .bind(author_id)
    .execute(pool.get_ref())
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Тариф не найден".to_string()));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Тариф удалён" })))
}

/// GET /api/v1/author/stats
/// Получение статистики автора
async fn get_stats(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
) -> Result<HttpResponse, AppError> {
    let author_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM authors WHERE user_id = $1"
    )
    .bind(claims.sub)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Автор не найден".to_string()))?;

    // Общая статистика
    let total_subscribers: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT user_id) FROM subscriptions WHERE author_id = $1 AND status = 'active'"
    )
    .bind(author_id)
    .fetch_one(pool.get_ref())
    .await?;

    let monthly_revenue: i64 = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(author_amount_kopecks), 0)
        FROM payments
        WHERE author_id = $1 AND status = 'succeeded' 
        AND created_at >= date_trunc('month', NOW())
        "#
    )
    .bind(author_id)
    .fetch_one(pool.get_ref())
    .await?;

    // Топ треки
    let top_tracks = sqlx::query_as::<_, TrackStat>(
        r#"
        SELECT title, plays_count
        FROM contents
        WHERE author_id = $1 AND status = 'published'
        ORDER BY plays_count DESC
        LIMIT 5
        "#
    )
    .bind(author_id)
    .fetch_all(pool.get_ref())
    .await?;

    // Подписчики по планам
    let subscribers_by_plan = sqlx::query_as::<_, PlanSubscribers>(
        r#"
        SELECT plan_type, COUNT(*) as count
        FROM subscriptions
        WHERE author_id = $1 AND status = 'active'
        GROUP BY plan_type
        "#
    )
    .bind(author_id)
    .fetch_all(pool.get_ref())
    .await?;

    let stats = AuthorStats {
        total_subscribers,
        active_subscribers: total_subscribers,
        monthly_revenue_kopecks: monthly_revenue,
        total_revenue_kopecks: 0, // Загружается отдельно
        conversion_rate: 0.0, // Рассчитывается
        top_tracks,
        subscribers_by_plan,
        revenue_by_month: vec![],
    };

    Ok(HttpResponse::Ok().json(stats))
}

/// GET /api/v1/author/payouts
/// История выплат автора
async fn get_payouts(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
) -> Result<HttpResponse, AppError> {
    let author_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM authors WHERE user_id = $1"
    )
    .bind(claims.sub)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Автор не найден".to_string()))?;

    let payouts = sqlx::query_as::<_, Payout>(
        "SELECT * FROM payouts WHERE author_id = $1 ORDER BY created_at DESC LIMIT 50"
    )
    .bind(author_id)
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(payouts))
}

/// POST /api/v1/author/payouts
/// Запрос на выплату
async fn request_payout(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
    body: web::Json<PayoutRequest>,
) -> Result<HttpResponse, AppError> {
    let author_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM authors WHERE user_id = $1"
    )
    .bind(claims.sub)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Автор не найден".to_string()))?;

    // Проверка доступного баланса
    let available_balance: i64 = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(author_amount_kopecks), 0) - COALESCE(
            (SELECT SUM(amount_kopecks) FROM payouts WHERE author_id = $1 AND status IN ('pending', 'processing', 'completed')),
            0
        )
        FROM payments
        WHERE author_id = $1 AND status = 'succeeded'
        "#
    )
    .bind(author_id)
    .fetch_one(pool.get_ref())
    .await?;

    if body.amount_kopecks > available_balance {
        return Err(AppError::ValidationError(
            format!("Недостаточно средств. Доступно: {} ₽", available_balance / 100)
        ));
    }

    let payout_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO payouts (id, author_id, amount_kopecks, method, details, status, created_at)
        VALUES ($1, $2, $3, $4, $5, 'pending', NOW())
        "#
    )
    .bind(payout_id)
    .bind(author_id)
    .bind(body.amount_kopecks)
    .bind(&body.method)
    .bind(&body.details)
    .execute(pool.get_ref())
    .await?;

    Ok(HttpResponse::Created().json(serde_json::json!({
        "id": payout_id,
        "message": "Заявка на выплату создана"
    })))
}

#[derive(serde::Deserialize)]
struct CreatePlanRequest {
    plan_type: PlanType,
    title: String,
    description: Option<String>,
    price_monthly_kopecks: i32,
    price_quarterly_kopecks: Option<i32>,
    price_yearly_kopecks: Option<i32>,
}
