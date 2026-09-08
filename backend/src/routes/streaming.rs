use actix_web::{web, HttpRequest, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::Config;
use crate::errors::AppError;
use crate::services::streaming_service;

/// Конфигурация роутов стриминга медиа
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/stream")
            .route("/token", web::post().to(get_media_token))
            .route("/audio/{content_id}/master.m3u8", web::get().to(stream_audio_master))
            .route("/audio/{content_id}/{segment}", web::get().to(stream_audio_segment))
            .route("/video/{content_id}/master.m3u8", web::get().to(stream_video_master))
            .route("/video/{content_id}/{segment}", web::get().to(stream_video_segment))
            .route("/teaser/{content_id}", web::get().to(get_teaser))
    );
}

/// POST /stream/token
/// Получение токена доступа к медиа-контенту
async fn get_media_token(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    req: HttpRequest,
    body: web::Json<MediaTokenRequest>,
) -> Result<HttpResponse, AppError> {
    // Извлечение пользователя из Authorization header
    let auth_header = req.headers().get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::AuthError("Требуется авторизация".to_string()))?;

    let token = auth_header.strip_prefix("Bearer ")
        .ok_or_else(|| AppError::AuthError("Неверный формат токена".to_string()))?;

    let claims = crate::services::auth_service::validate_token(token, &config.jwt_secret)?;
    let user_id = claims.sub;

    // Проверка доступа к контенту
    let has_access = streaming_service::check_content_access(
        pool.get_ref(),
        user_id,
        body.content_id,
    ).await?;

    // FIX: Генерация токена с секретом из config (не хардкод!)
    let media_token = streaming_service::generate_media_token(
        user_id,
        body.content_id,
        has_access,
        config.token_expiration_minutes,
        &config.media_token_secret,
    )?;

    // Запись события воспроизведения
    let ip = req.peer_addr()
        .map(|addr| addr.ip().to_string())
        .unwrap_or_default();

    sqlx::query(
        r#"
        INSERT INTO play_events (id, user_id, content_id, is_teaser, played_seconds, total_seconds, completed, device_info, ip_address, created_at)
        VALUES ($1, $2, $3, $4, 0, 0, false, '', $5, NOW())
        "#
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(body.content_id)
    .bind(!has_access)
    .bind(&ip)
    .execute(pool.get_ref())
    .await?;

    // Audit log
    let _ = crate::services::auth_service::audit_log(
        pool.get_ref(),
        Some(user_id),
        "media_token_issued",
        "content",
        Some(body.content_id),
        &ip,
        req.headers().get("User-Agent").and_then(|v| v.to_str().ok()).unwrap_or(""),
        Some(serde_json::json!({ "is_full_access": has_access })),
    ).await;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "token": media_token,
        "is_full_access": has_access,
        "expires_in_seconds": config.token_expiration_minutes * 60,
    })))
}

/// GET /stream/audio/{content_id}/master.m3u8
async fn stream_audio_master(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let content_id = path.into_inner();

    let token = req.match_info().query("token");
    // FIX: Валидация с секретом из config
    let media_claims = streaming_service::validate_media_token(token, &config.media_token_secret)?;

    if media_claims.content_id != content_id {
        return Err(AppError::ForbiddenError("Токен не соответствует контенту".to_string()));
    }

    // FIX: Валидация Referer
    let referer = req.headers().get("Referer")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if !referer.is_empty() && !config.is_origin_allowed(referer) {
        return Err(AppError::ForbiddenError("Недопустимый источник запроса".to_string()));
    }

    let playlist = streaming_service::generate_audio_playlist(
        pool.get_ref(),
        content_id,
        media_claims.is_full_access,
        token,
    ).await?;

    Ok(HttpResponse::Ok()
        .content_type("application/vnd.apple.mpegurl")
        .insert_header(("Cache-Control", "no-cache, no-store"))
        .body(playlist))
}

/// GET /stream/audio/{content_id}/{segment}
async fn stream_audio_segment(
    pool: web::Data<PgPool>,
    redis: web::Data<redis::Client>,
    config: web::Data<Config>,
    path: web::Path<(Uuid, String)>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (content_id, segment_name) = path.into_inner();

    let token = req.match_info().query("token");
    let media_claims = streaming_service::validate_media_token(token, &config.media_token_secret)?;

    if media_claims.content_id != content_id {
        return Err(AppError::ForbiddenError("Токен не соответствует контенту".to_string()));
    }

    // FIX: Реальный rate limiting через Redis
    let rate_key = format!("rate:{}:{}", media_claims.user_id, content_id);
    streaming_service::check_rate_limit(
        redis.get_ref(),
        &rate_key,
        config.rate_limit_media,
        60,
    ).await?;

    let segment_data = streaming_service::get_segment_from_storage(
        config.get_ref(),
        content_id,
        &segment_name,
    ).await?;

    Ok(HttpResponse::Ok()
        .content_type("video/mp2t")
        .insert_header(("Cache-Control", "no-cache, no-store"))
        .body(segment_data))
}

/// GET /stream/video/{content_id}/master.m3u8
async fn stream_video_master(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let content_id = path.into_inner();

    let token = req.match_info().query("token");
    let media_claims = streaming_service::validate_media_token(token, &config.media_token_secret)?;

    if media_claims.content_id != content_id {
        return Err(AppError::ForbiddenError("Токен не соответствует контенту".to_string()));
    }

    let referer = req.headers().get("Referer")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if !referer.is_empty() && !config.is_origin_allowed(referer) {
        return Err(AppError::ForbiddenError("Недопустимый источник запроса".to_string()));
    }

    let playlist = streaming_service::generate_video_playlist(
        pool.get_ref(),
        content_id,
        media_claims.is_full_access,
        token,
    ).await?;

    Ok(HttpResponse::Ok()
        .content_type("application/vnd.apple.mpegurl")
        .insert_header(("Cache-Control", "no-cache, no-store"))
        .body(playlist))
}

/// GET /stream/video/{content_id}/{segment}
async fn stream_video_segment(
    pool: web::Data<PgPool>,
    redis: web::Data<redis::Client>,
    config: web::Data<Config>,
    path: web::Path<(Uuid, String)>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (content_id, segment_name) = path.into_inner();

    let token = req.match_info().query("token");
    let media_claims = streaming_service::validate_media_token(token, &config.media_token_secret)?;

    if media_claims.content_id != content_id {
        return Err(AppError::ForbiddenError("Токен не соответствует контенту".to_string()));
    }

    let rate_key = format!("rate:{}:{}", media_claims.user_id, content_id);
    streaming_service::check_rate_limit(
        redis.get_ref(),
        &rate_key,
        config.rate_limit_media,
        60,
    ).await?;

    // FIX: Реальные водяные знаки через FFmpeg
    let segment_data = streaming_service::get_video_segment_with_watermark(
        config.get_ref(),
        pool.get_ref(),
        content_id,
        &segment_name,
        media_claims.user_id,
    ).await?;

    Ok(HttpResponse::Ok()
        .content_type("video/mp2t")
        .insert_header(("Cache-Control", "no-cache, no-store"))
        .body(segment_data))
}

/// GET /stream/teaser/{content_id}
async fn get_teaser(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let content_id = path.into_inner();

    let teaser_url: String = sqlx::query_scalar(
        "SELECT teaser_url FROM contents WHERE id = $1 AND status = 'published'"
    )
    .bind(content_id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Контент не найден".to_string()))?;

    if teaser_url.is_empty() {
        return Err(AppError::NotFound("Тизер не найден".to_string()));
    }

    let teaser_data = streaming_service::get_segment_from_storage(
        config.get_ref(),
        content_id,
        &teaser_url,
    ).await?;

    Ok(HttpResponse::Ok()
        .content_type("audio/mpeg")
        .insert_header(("Cache-Control", "public, max-age=3600"))
        .body(teaser_data))
}

#[derive(serde::Deserialize)]
struct MediaTokenRequest {
    content_id: Uuid,
}
