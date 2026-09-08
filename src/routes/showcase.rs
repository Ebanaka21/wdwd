use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::author::*;
use crate::models::content::ContentPublic;

/// Конфигурация публичных роутов витрины
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .route("/api/v1/showcase/{slug}", web::get().to(get_showcase))
            .route("/api/v1/showcase/{slug}/content", web::get().to(get_showcase_content))
            .route("/api/v1/showcase/{slug}/plans", web::get().to(get_showcase_plans))
    );
}

/// GET /api/v1/showcase/{slug}
/// Получение публичной витрины автора
async fn get_showcase(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let slug = path.into_inner();

    let author = sqlx::query_as::<_, Author>(
        "SELECT * FROM authors WHERE slug = $1 AND is_active = true"
    )
    .bind(&slug)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Автор не найден".to_string()))?;

    // Получение планов
    let plans = sqlx::query_as::<_, PlanPublic>(
        r#"
        SELECT id, plan_type, title, description, price_monthly_kopecks, 
               price_quarterly_kopecks, price_yearly_kopecks, subscribers_count
        FROM plans WHERE author_id = $1 AND is_enabled = true
        ORDER BY price_monthly_kopecks ASC
        "#
    )
    .bind(author.id)
    .fetch_all(pool.get_ref())
    .await?;

    let showcase = AuthorShowcase {
        id: author.id,
        slug: author.slug,
        display_name: author.display_name,
        description: author.description,
        avatar_url: author.avatar_url,
        cover_url: author.cover_url,
        telegram_url: author.telegram_url,
        vk_url: author.vk_url,
        youtube_url: author.youtube_url,
        is_verified: author.is_verified,
        total_subscribers: author.total_subscribers,
        plans,
    };

    Ok(HttpResponse::Ok().json(showcase))
}

/// GET /api/v1/showcase/{slug}/content
/// Получение контента автора (с тизерами для гостей)
async fn get_showcase_content(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let slug = path.into_inner();

    // Проверка существования автора
    let author = sqlx::query_as::<_, Author>(
        "SELECT * FROM authors WHERE slug = $1 AND is_active = true"
    )
    .bind(&slug)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Автор не найден".to_string()))?;

    // Получение опубликованного контента
    let content = sqlx::query_as::<_, ContentPublic>(
        r#"
        SELECT 
            c.id, c.content_type, c.title, c.description, c.cover_url,
            c.album, c.duration_seconds, c.format, c.teaser_url,
            c.teaser_start_seconds, c.teaser_duration_seconds,
            false as is_available,
            c.plays_count
        FROM contents c
        WHERE c.author_id = $1 AND c.status = 'published'
        ORDER BY c.sort_order ASC, c.created_at DESC
        "#
    )
    .bind(author.id)
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(content))
}

/// GET /api/v1/showcase/{slug}/plans
/// Получение тарифных планов автора
async fn get_showcase_plans(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let slug = path.into_inner();

    let author = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM authors WHERE slug = $1 AND is_active = true"
    )
    .bind(&slug)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Автор не найден".to_string()))?;

    let plans = sqlx::query_as::<_, PlanPublic>(
        r#"
        SELECT id, plan_type, title, description, price_monthly_kopecks,
               price_quarterly_kopecks, price_yearly_kopecks, subscribers_count
        FROM plans WHERE author_id = $1 AND is_enabled = true
        ORDER BY price_monthly_kopecks ASC
        "#
    )
    .bind(author)
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(plans))
}
