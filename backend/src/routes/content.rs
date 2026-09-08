use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::Config;
use crate::errors::AppError;
use crate::models::content::*;
use crate::models::user::JwtClaims;
use crate::services::content_service;

/// Конфигурация роутов контента
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/content")
            .route("", web::get().to(list_content))
            .route("", web::post().to(create_content))
            .route("/{content_id}", web::get().to(get_content))
            .route("/{content_id}", web::put().to(update_content))
            .route("/{content_id}", web::delete().to(delete_content))
            .route("/{content_id}/publish", web::post().to(publish_content))
            .route("/upload", web::post().to(upload_file))
    );
}

/// GET /api/v1/content
/// Список контента автора
async fn list_content(
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

    let content = sqlx::query_as::<_, Content>(
        "SELECT * FROM contents WHERE author_id = $1 ORDER BY sort_order ASC, created_at DESC"
    )
    .bind(author_id)
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(content))
}

/// POST /api/v1/content
/// Создание записи контента (без файла — файл загружается отдельно)
async fn create_content(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
    body: web::Json<CreateContentRequest>,
) -> Result<HttpResponse, AppError> {
    let author_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM authors WHERE user_id = $1"
    )
    .bind(claims.sub)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Автор не найден".to_string()))?;

    let content_id = Uuid::new_v4();
    let teaser_start = body.teaser_start_seconds.unwrap_or(0);
    let teaser_duration = body.teaser_duration_seconds.unwrap_or(
        match body.content_type {
            ContentType::Audio => 15,
            ContentType::Video => 60,
        }
    );

    // Валидация длительности тизера
    match body.content_type {
        ContentType::Audio if teaser_duration > 15 => {
            return Err(AppError::ValidationError(
                "Тизер аудио не может превышать 15 секунд".to_string()
            ));
        }
        ContentType::Video if teaser_duration > 60 => {
            return Err(AppError::ValidationError(
                "Тизер видео не может превышать 60 секунд".to_string()
            ));
        }
        _ => {}
    }

    sqlx::query(
        r#"
        INSERT INTO contents (
            id, author_id, content_type, title, description, album,
            duration_seconds, file_url, teaser_url, format, file_size_bytes,
            teaser_start_seconds, teaser_duration_seconds,
            status, is_downloadable, sort_order, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, 0, '', '', '', 0, $7, $8, 'draft', $9, 0, NOW(), NOW())
        "#
    )
    .bind(content_id)
    .bind(author_id)
    .bind(&body.content_type)
    .bind(&body.title)
    .bind(&body.description)
    .bind(&body.album)
    .bind(teaser_start)
    .bind(teaser_duration)
    .bind(body.is_downloadable.unwrap_or(false))
    .execute(pool.get_ref())
    .await?;

    Ok(HttpResponse::Created().json(serde_json::json!({
        "id": content_id,
        "message": "Контент создан. Загрузите файл."
    })))
}

/// GET /api/v1/content/{content_id}
/// Получение информации о контенте
async fn get_content(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let content_id = path.into_inner();

    let content = sqlx::query_as::<_, Content>(
        "SELECT * FROM contents WHERE id = $1"
    )
    .bind(content_id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Контент не найден".to_string()))?;

    // Проверка, что контент принадлежит автору
    let author_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM authors WHERE user_id = $1"
    )
    .bind(claims.sub)
    .fetch_optional(pool.get_ref())
    .await?;

    if let Some(aid) = author_id {
        if content.author_id != aid {
            return Err(AppError::ForbiddenError("Нет доступа".to_string()));
        }
    }

    Ok(HttpResponse::Ok().json(content))
}

/// PUT /api/v1/content/{content_id}
/// Обновление контента
async fn update_content(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateContentRequest>,
) -> Result<HttpResponse, AppError> {
    let content_id = path.into_inner();

    // Проверка прав
    let author_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM authors WHERE user_id = $1"
    )
    .bind(claims.sub)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Автор не найден".to_string()))?;

    let content_author = sqlx::query_scalar::<_, Uuid>(
        "SELECT author_id FROM contents WHERE id = $1"
    )
    .bind(content_id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Контент не найден".to_string()))?;

    if content_author != author_id {
        return Err(AppError::ForbiddenError("Нет доступа".to_string()));
    }

    sqlx::query(
        r#"
        UPDATE contents SET
            title = COALESCE($1, title),
            description = COALESCE($2, description),
            cover_url = COALESCE($3, cover_url),
            album = COALESCE($4, album),
            teaser_start_seconds = COALESCE($5, teaser_start_seconds),
            teaser_duration_seconds = COALESCE($6, teaser_duration_seconds),
            is_downloadable = COALESCE($7, is_downloadable),
            status = COALESCE($8, status),
            sort_order = COALESCE($9, sort_order),
            updated_at = NOW()
        WHERE id = $10
        "#
    )
    .bind(&body.title)
    .bind(&body.description)
    .bind(&body.cover_url)
    .bind(&body.album)
    .bind(body.teaser_start_seconds)
    .bind(body.teaser_duration_seconds)
    .bind(body.is_downloadable)
    .bind(&body.status)
    .bind(body.sort_order)
    .bind(content_id)
    .execute(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Контент обновлён" })))
}

/// DELETE /api/v1/content/{content_id}
/// Удаление контента
async fn delete_content(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let content_id = path.into_inner();

    let author_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM authors WHERE user_id = $1"
    )
    .bind(claims.sub)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Автор не найден".to_string()))?;

    let result = sqlx::query(
        "DELETE FROM contents WHERE id = $1 AND author_id = $2"
    )
    .bind(content_id)
    .bind(author_id)
    .execute(pool.get_ref())
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Контент не найден".to_string()));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Контент удалён" })))
}

/// POST /api/v1/content/{content_id}/publish
/// Публикация контента
async fn publish_content(
    pool: web::Data<PgPool>,
    claims: web::ReqData<JwtClaims>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let content_id = path.into_inner();

    let author_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM authors WHERE user_id = $1"
    )
    .bind(claims.sub)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Автор не найден".to_string()))?;

    // Проверка, что файл загружен
    let file_url: String = sqlx::query_scalar(
        "SELECT file_url FROM contents WHERE id = $1 AND author_id = $2"
    )
    .bind(content_id)
    .bind(author_id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Контент не найден".to_string()))?;

    if file_url.is_empty() {
        return Err(AppError::ValidationError("Сначала загрузите файл".to_string()));
    }

    sqlx::query(
        "UPDATE contents SET status = 'published', published_at = NOW(), updated_at = NOW() WHERE id = $1"
    )
    .bind(content_id)
    .execute(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Контент опубликован" })))
}

/// POST /api/v1/content/upload
/// Загрузка файла контента (presigned URL для S3)
async fn upload_file(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    claims: web::ReqData<JwtClaims>,
    body: web::Json<UploadFileRequest>,
) -> Result<HttpResponse, AppError> {
    let author_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM authors WHERE user_id = $1"
    )
    .bind(claims.sub)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Автор не найден".to_string()))?;

    // Генерация presigned URL для загрузки в S3
    let object_key = format!(
        "authors/{}/content/{}/{}.{}",
        author_id,
        body.content_id,
        Uuid::new_v4(),
        body.file_extension
    );

    // В реальном коде здесь генерируется presigned URL через rust-s3
    let upload_url = format!(
        "https://{}/{}/{}",
        config.s3_bucket, config.s3_endpoint, object_key
    );

    // После загрузки — обновляем запись в БД
    sqlx::query(
        "UPDATE contents SET file_url = $1, updated_at = NOW() WHERE id = $2 AND author_id = $3"
    )
    .bind(&object_key)
    .bind(body.content_id)
    .bind(author_id)
    .execute(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "upload_url": upload_url,
        "object_key": object_key,
        "message": "Загрузите файл по указанному URL"
    })))
}

#[derive(serde::Deserialize)]
struct UploadFileRequest {
    content_id: Uuid,
    file_extension: String,
    file_size_bytes: i64,
}
