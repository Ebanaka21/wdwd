use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::user::{JwtClaims, UserRole};

/// Конфигурация роутов администратора
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin")
            .route("/stats", web::get().to(get_platform_stats))
            .route("/users", web::get().to(list_users))
            .route("/users/{user_id}/ban", web::post().to(ban_user))
            .route("/users/{user_id}/unban", web::post().to(unban_user))
            .route("/authors", web::get().to(list_authors))
            .route("/authors/{author_id}/verify", web::post().to(verify_author))
            .route("/authors/{author_id}/reject", web::post().to(reject_author))
            .route("/transactions", web::get().to(list_transactions))
            .route("/transactions/{payment_id}/refund", web::post().to(refund_payment))
            .route("/moderation/reports", web::get().to(list_reports))
            .route("/moderation/reports/{report_id}/resolve", web::post().to(resolve_report))
            .route("/settings/commission", web::put().to(update_commission))
            .route("/payouts/pending", web::get().to(list_pending_payouts))
            .route("/payouts/{payout_id}/approve", web::post().to(approve_payout))
            .route("/payouts/{payout_id}/reject", web::post().to(reject_payout))
    );
}

/// GET /api/v1/admin/stats
/// Статистика платформы
async fn get_platform_stats(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
) -> Result<HttpResponse, AppError> {
    // Проверка роли админа
    if claims.role != UserRole::Admin {
        return Err(AppError::ForbiddenError("Требуются права администратора".to_string()));
    }

    let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE role = 'fan'")
        .fetch_one(pool.get_ref()).await?;

    let total_authors: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM authors")
        .fetch_one(pool.get_ref()).await?;

    let monthly_revenue: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount_kopecks), 0) FROM payments WHERE status = 'succeeded' AND created_at >= date_trunc('month', NOW())"
    ).fetch_one(pool.get_ref()).await?;

    let monthly_commission: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(commission_kopecks), 0) FROM payments WHERE status = 'succeeded' AND created_at >= date_trunc('month', NOW())"
    ).fetch_one(pool.get_ref()).await?;

    let pending_moderation: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM content_reports WHERE status = 'pending'"
    ).fetch_one(pool.get_ref()).await.unwrap_or(0);

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "total_users": total_users,
        "total_authors": total_authors,
        "monthly_revenue_kopecks": monthly_revenue,
        "monthly_commission_kopecks": monthly_commission,
        "pending_moderation": pending_moderation,
    })))
}

/// GET /api/v1/admin/users
/// Список пользователей (с пагинацией и поиском)
async fn list_users(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
    query: web::Query<UserQuery>,
) -> Result<HttpResponse, AppError> {
    if claims.role != UserRole::Admin {
        return Err(AppError::ForbiddenError("Требуются права администратора".to_string()));
    }

    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);

    let users = if let Some(ref search) = query.search {
        sqlx::query_as::<_, AdminUser>(
            r#"
            SELECT id, email, display_name, role, is_active, created_at, last_login_at
            FROM users
            WHERE email ILIKE $1 OR display_name ILIKE $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(format!("%{}%", search))
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(pool.get_ref())
        .await?
    } else {
        sqlx::query_as::<_, AdminUser>(
            "SELECT id, email, display_name, role, is_active, created_at, last_login_at FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(pool.get_ref())
        .await?
    };

    Ok(HttpResponse::Ok().json(users))
}

/// POST /api/v1/admin/users/{user_id}/ban
/// Блокировка пользователя
async fn ban_user(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    if claims.role != UserRole::Admin {
        return Err(AppError::ForbiddenError("Требуются права администратора".to_string()));
    }

    let user_id = path.into_inner();
    sqlx::query("UPDATE users SET is_active = false WHERE id = $1")
        .bind(user_id)
        .execute(pool.get_ref())
        .await?;

    // Деактивация всех сессий
    sqlx::query("UPDATE user_sessions SET is_active = false WHERE user_id = $1")
        .bind(user_id)
        .execute(pool.get_ref())
        .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Пользователь заблокирован" })))
}

/// POST /api/v1/admin/users/{user_id}/unban
/// Разблокировка пользователя
async fn unban_user(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    if claims.role != UserRole::Admin {
        return Err(AppError::ForbiddenError("Требуются права администратора".to_string()));
    }

    let user_id = path.into_inner();
    sqlx::query("UPDATE users SET is_active = true WHERE id = $1")
        .bind(user_id)
        .execute(pool.get_ref())
        .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Пользователь разблокирован" })))
}

/// GET /api/v1/admin/authors
/// Список авторов
async fn list_authors(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
) -> Result<HttpResponse, AppError> {
    if claims.role != UserRole::Admin {
        return Err(AppError::ForbiddenError("Требуются права администратора".to_string()));
    }

    let authors = sqlx::query_as::<_, AdminAuthor>(
        r#"
        SELECT a.id, a.display_name, a.slug, a.is_verified, a.total_subscribers,
               a.total_revenue_kopecks, a.created_at
        FROM authors a
        ORDER BY a.created_at DESC
        LIMIT 100
        "#
    )
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(authors))
}

/// POST /api/v1/admin/authors/{author_id}/verify
/// Верификация автора
async fn verify_author(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    if claims.role != UserRole::Admin {
        return Err(AppError::ForbiddenError("Требуются права администратора".to_string()));
    }

    let author_id = path.into_inner();
    sqlx::query("UPDATE authors SET is_verified = true, updated_at = NOW() WHERE id = $1")
        .bind(author_id)
        .execute(pool.get_ref())
        .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Автор верифицирован" })))
}

/// POST /api/v1/admin/authors/{author_id}/reject
/// Отклонение автора
async fn reject_author(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    if claims.role != UserRole::Admin {
        return Err(AppError::ForbiddenError("Требуются права администратора".to_string()));
    }

    let author_id = path.into_inner();
    sqlx::query("UPDATE authors SET is_active = false, updated_at = NOW() WHERE id = $1")
        .bind(author_id)
        .execute(pool.get_ref())
        .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Автор отклонён" })))
}

/// GET /api/v1/admin/transactions
/// Список транзакций
async fn list_transactions(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
) -> Result<HttpResponse, AppError> {
    if claims.role != UserRole::Admin {
        return Err(AppError::ForbiddenError("Требуются права администратора".to_string()));
    }

    let transactions = sqlx::query_as::<_, AdminTransaction>(
        r#"
        SELECT p.id, p.amount_kopecks, p.commission_kopecks, p.status,
               p.payment_method, p.created_at,
               u.display_name as user_name,
               a.display_name as author_name
        FROM payments p
        JOIN users u ON p.user_id = u.id
        JOIN authors a ON p.author_id = a.id
        ORDER BY p.created_at DESC
        LIMIT 100
        "#
    )
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(transactions))
}

/// POST /api/v1/admin/transactions/{payment_id}/refund
/// Возврат платежа
async fn refund_payment(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
    path: web::Path<Uuid>,
    body: web::Json<RefundBody>,
) -> Result<HttpResponse, AppError> {
    if claims.role != UserRole::Admin {
        return Err(AppError::ForbiddenError("Требуются права администратора".to_string()));
    }

    let payment_id = path.into_inner();
    sqlx::query(
        "UPDATE payments SET status = 'refunded', refunded_at = NOW() WHERE id = $1 AND status = 'succeeded'"
    )
    .bind(payment_id)
    .execute(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Возврат оформлен",
        "reason": body.reason,
    })))
}

/// GET /api/v1/admin/moderation/reports
/// Список жалоб
async fn list_reports(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
) -> Result<HttpResponse, AppError> {
    if claims.role != UserRole::Admin {
        return Err(AppError::ForbiddenError("Требуются права администратора".to_string()));
    }

    let reports = sqlx::query_as::<_, ContentReport>(
        r#"
        SELECT id, content_id, reporter_id, reason, description, status, created_at
        FROM content_reports
        WHERE status = 'pending'
        ORDER BY created_at DESC
        LIMIT 50
        "#
    )
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(reports))
}

/// POST /api/v1/admin/moderation/reports/{report_id}/resolve
/// Обработка жалобы
async fn resolve_report(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
    path: web::Path<Uuid>,
    body: web::Json<ResolveReportBody>,
) -> Result<HttpResponse, AppError> {
    if claims.role != UserRole::Admin {
        return Err(AppError::ForbiddenError("Требуются права администратора".to_string()));
    }

    let report_id = path.into_inner();

    sqlx::query(
        "UPDATE content_reports SET status = $1, resolved_at = NOW() WHERE id = $2"
    )
    .bind(if body.action == "confirm" { "confirmed" } else { "dismissed" })
    .bind(report_id)
    .execute(pool.get_ref())
    .await?;

    // Если жалоба подтверждена — архивируем контент
    if body.action == "confirm" {
        let content_id: Uuid = sqlx::query_scalar(
            "SELECT content_id FROM content_reports WHERE id = $1"
        )
        .bind(report_id)
        .fetch_optional(pool.get_ref())
        .await?
        .ok_or_else(|| AppError::NotFound("Жалоба не найдена".to_string()))?;

        sqlx::query(
            "UPDATE contents SET status = 'archived', updated_at = NOW() WHERE id = $1"
        )
        .bind(content_id)
        .execute(pool.get_ref())
        .await?;
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Жалоба обработана" })))
}

/// PUT /api/v1/admin/settings/commission
/// Обновление комиссии платформы
async fn update_commission(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
    body: web::Json<CommissionBody>,
) -> Result<HttpResponse, AppError> {
    if claims.role != UserRole::Admin {
        return Err(AppError::ForbiddenError("Требуются права администратора".to_string()));
    }

    if body.percent < 0.0 || body.percent > 50.0 {
        return Err(AppError::ValidationError("Комиссия должна быть от 0 до 50%".to_string()));
    }

    // В реальном коде — сохранение в таблицу platform_settings
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Комиссия обновлена",
        "new_percent": body.percent,
    })))
}

/// GET /api/v1/admin/payouts/pending
/// Ожидающие выплаты
async fn list_pending_payouts(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
) -> Result<HttpResponse, AppError> {
    if claims.role != UserRole::Admin {
        return Err(AppError::ForbiddenError("Требуются права администратора".to_string()));
    }

    let payouts = sqlx::query_as::<_, AdminPayout>(
        r#"
        SELECT p.id, p.amount_kopecks, p.method, p.details, p.created_at,
               a.display_name as author_name
        FROM payouts p
        JOIN authors a ON p.author_id = a.id
        WHERE p.status = 'pending'
        ORDER BY p.created_at ASC
        "#
    )
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(payouts))
}

/// POST /api/v1/admin/payouts/{payout_id}/approve
/// Одобрение выплаты
async fn approve_payout(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    if claims.role != UserRole::Admin {
        return Err(AppError::ForbiddenError("Требуются права администратора".to_string()));
    }

    let payout_id = path.into_inner();
    sqlx::query(
        "UPDATE payouts SET status = 'processing', processed_at = NOW() WHERE id = $1 AND status = 'pending'"
    )
    .bind(payout_id)
    .execute(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Выплата одобрена" })))
}

/// POST /api/v1/admin/payouts/{payout_id}/reject
/// Отклонение выплаты
async fn reject_payout(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    if claims.role != UserRole::Admin {
        return Err(AppError::ForbiddenError("Требуются права администратора".to_string()));
    }

    let payout_id = path.into_inner();
    sqlx::query(
        "UPDATE payouts SET status = 'rejected' WHERE id = $1 AND status = 'pending'"
    )
    .bind(payout_id)
    .execute(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Выплата отклонена" })))
}

// Вспомогательные структуры
#[derive(serde::Deserialize)]
struct UserQuery {
    search: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct AdminUser {
    id: Uuid,
    email: String,
    display_name: String,
    role: UserRole,
    is_active: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    last_login_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct AdminAuthor {
    id: Uuid,
    display_name: String,
    slug: String,
    is_verified: bool,
    total_subscribers: i64,
    total_revenue_kopecks: i64,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct AdminTransaction {
    id: Uuid,
    amount_kopecks: i32,
    commission_kopecks: i32,
    status: crate::models::payment::PaymentStatus,
    payment_method: crate::models::payment::PaymentMethodType,
    created_at: chrono::DateTime<chrono::Utc>,
    user_name: String,
    author_name: String,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct ContentReport {
    id: Uuid,
    content_id: Uuid,
    reporter_id: Option<Uuid>,
    reason: String,
    description: Option<String>,
    status: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct AdminPayout {
    id: Uuid,
    amount_kopecks: i64,
    method: crate::models::author::PayoutMethod,
    details: String,
    created_at: chrono::DateTime<chrono::Utc>,
    author_name: String,
}

#[derive(serde::Deserialize)]
struct RefundBody {
    reason: String,
}

#[derive(serde::Deserialize)]
struct ResolveReportBody {
    action: String, // "confirm" или "dismiss"
}

#[derive(serde::Deserialize)]
struct CommissionBody {
    percent: f64,
}
