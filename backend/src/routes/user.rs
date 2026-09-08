use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::user::*;

/// Конфигурация роутов пользователя
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/user")
            .route("/profile", web::get().to(get_profile))
            .route("/profile", web::put().to(update_profile))
            .route("/library", web::get().to(get_library))
            .route("/sessions", web::get().to(get_sessions))
            .route("/sessions/{session_id}", web::delete().to(revoke_session))
    );
}

/// GET /api/v1/user/profile
/// Получение профиля пользователя
async fn get_profile(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
) -> Result<HttpResponse, AppError> {
    let user = sqlx::query_as::<_, UserPublic>(
        "SELECT id, display_name, email, role, avatar_url, is_verified, created_at FROM users WHERE id = $1"
    )
    .bind(claims.sub)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Пользователь не найден".to_string()))?;

    Ok(HttpResponse::Ok().json(user))
}

/// PUT /api/v1/user/profile
/// Обновление профиля
async fn update_profile(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
    body: web::Json<UpdateProfileRequest>,
) -> Result<HttpResponse, AppError> {
    sqlx::query(
        r#"
        UPDATE users SET
            display_name = COALESCE($1, display_name),
            phone = COALESCE($2, phone),
            avatar_url = COALESCE($3, avatar_url),
            updated_at = NOW()
        WHERE id = $4
        "#
    )
    .bind(&body.display_name)
    .bind(&body.phone)
    .bind(&body.avatar_url)
    .bind(claims.sub)
    .execute(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Профиль обновлён" })))
}

/// GET /api/v1/user/library
/// Библиотека пользователя (весь доступный контент из подписок)
async fn get_library(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
) -> Result<HttpResponse, AppError> {
    let library = sqlx::query_as::<_, LibraryItem>(
        r#"
        SELECT DISTINCT ON (c.id)
            c.id, c.content_type, c.title, c.description, c.cover_url,
            c.album, c.duration_seconds, c.format, c.file_url,
            a.display_name as author_name, a.slug as author_slug, a.avatar_url as author_avatar_url
        FROM contents c
        JOIN authors a ON c.author_id = a.id
        JOIN subscriptions s ON s.author_id = a.id
        WHERE s.user_id = $1 
        AND s.status IN ('active', 'grace_period')
        AND s.current_period_end > NOW()
        AND c.status = 'published'
        ORDER BY c.id, c.created_at DESC
        "#
    )
    .bind(claims.sub)
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(library))
}

/// GET /api/v1/user/sessions
/// Активные сессии пользователя
async fn get_sessions(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
) -> Result<HttpResponse, AppError> {
    let sessions = sqlx::query_as::<_, crate::models::session::SessionInfo>(
        r#"
        SELECT id, device_info, ip_address, created_at, last_activity_at
        FROM user_sessions
        WHERE user_id = $1 AND is_active = true
        ORDER BY last_activity_at DESC
        "#
    )
    .bind(claims.sub)
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(sessions))
}

/// DELETE /api/v1/user/sessions/{session_id}
/// Завершение сессии
async fn revoke_session(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let session_id = path.into_inner();

    sqlx::query(
        "UPDATE user_sessions SET is_active = false WHERE id = $1 AND user_id = $2"
    )
    .bind(session_id)
    .bind(claims.sub)
    .execute(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Сессия завершена" })))
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct LibraryItem {
    id: Uuid,
    content_type: crate::models::content::ContentType,
    title: String,
    description: Option<String>,
    cover_url: Option<String>,
    album: Option<String>,
    duration_seconds: i32,
    format: String,
    file_url: String,
    author_name: String,
    author_slug: String,
    author_avatar_url: Option<String>,
}
