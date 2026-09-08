use chrono::{Utc, Duration};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::Config;
use crate::errors::AppError;

/// Claims для медиа-токена
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MediaTokenClaims {
    pub user_id: Uuid,
    pub content_id: Uuid,
    pub is_full_access: bool,
    pub exp: i64,
    pub iat: i64,
}

/// Проверка доступа пользователя к контенту
pub async fn check_content_access(
    pool: &PgPool,
    user_id: Uuid,
    content_id: Uuid,
) -> Result<bool, AppError> {
    // Получение author_id контента
    let author_id: Uuid = sqlx::query_scalar(
        "SELECT author_id FROM contents WHERE id = $1 AND status = 'published'"
    )
    .bind(content_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Контент не найден".to_string()))?;

    // Проверка активной подписки
    let has_access: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM subscriptions
            WHERE user_id = $1
            AND author_id = $2
            AND status IN ('active', 'grace_period')
            AND current_period_end > NOW()
        )
        "#
    )
    .bind(user_id)
    .bind(author_id)
    .fetch_one(pool)
    .await?;

    Ok(has_access)
}

/// Генерация медиа-токена для доступа к сегментам
pub fn generate_media_token(
    user_id: Uuid,
    content_id: Uuid,
    is_full_access: bool,
    expiration_minutes: u32,
) -> Result<String, AppError> {
    let now = Utc::now();
    let exp = now + Duration::minutes(expiration_minutes as i64);

    let claims = MediaTokenClaims {
        user_id,
        content_id,
        is_full_access,
        exp: exp.timestamp(),
        iat: now.timestamp(),
    };

    // Используем отдельный секрет для медиа-токенов
    let secret = "chistovik_media_token_secret_key_2026";

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    ).map_err(|e| AppError::InternalError(format!("Media token error: {}", e)))?;

    Ok(token)
}

/// Валидация медиа-токена
pub fn validate_media_token(token: &str) -> Result<MediaTokenClaims, AppError> {
    let secret = "chistovik_media_token_secret_key_2026";

    let token_data = decode::<MediaTokenClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    ).map_err(|_| AppError::AuthError("Невалидный медиа-токен".to_string()))?;

    Ok(token_data.claims)
}

/// Проверка rate limit через Redis
pub async fn check_rate_limit(
    pool: &PgPool,
    rate_key: &str,
) -> Result<(), AppError> {
    // В реальном коде — проверка через Redis:
    // let redis = redis_client.get_async_connection().await?;
    // let current: u32 = redis::cmd("INCR").arg(&rate_key).query_async(&mut redis).await?;
    // if current == 1 { redis::cmd("EXPIRE").arg(&rate_key).arg(60).query_async(&mut redis).await?; }
    // if current > 30 { return Err(AppError::RateLimitExceeded); }

    // Заглушка — всегда разрешаем
    Ok(())
}

/// Генерация HLS master playlist для аудио
pub async fn generate_audio_playlist(
    pool: &PgPool,
    content_id: Uuid,
    is_full_access: bool,
    token: &str,
) -> Result<String, AppError> {
    let file_url: String = sqlx::query_scalar(
        "SELECT file_url FROM contents WHERE id = $1"
    )
    .bind(content_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Контент не найден".to_string()))?;

    // Генерация M3U8 плейлиста
    let playlist = if is_full_access {
        format!(
            r#"#EXTM3U
#EXT-X-VERSION:3
#EXT-X-TARGETDURATION:10
#EXT-X-MEDIA-SEQUENCE:0
#EXTINF:10.0,
/stream/audio/{content_id}/segment_000.ts?token={token}
#EXTINF:10.0,
/stream/audio/{content_id}/segment_001.ts?token={token}
#EXTINF:10.0,
/stream/audio/{content_id}/segment_002.ts?token={token}
#EXT-X-ENDLIST"#
        )
    } else {
        // Тизер — только первый сегмент
        format!(
            r#"#EXTM3U
#EXT-X-VERSION:3
#EXT-X-TARGETDURATION:15
#EXT-X-MEDIA-SEQUENCE:0
#EXTINF:15.0,
/stream/audio/{content_id}/teaser_segment.ts?token={token}
#EXT-X-ENDLIST"#
        )
    };

    Ok(playlist)
}

/// Генерация HLS master playlist для видео (адаптивный стриминг)
pub async fn generate_video_playlist(
    pool: &PgPool,
    content_id: Uuid,
    is_full_access: bool,
    token: &str,
) -> Result<String, AppError> {
    let playlist = if is_full_access {
        format!(
            r#"#EXTM3U
#EXT-X-VERSION:3

#EXT-X-STREAM-INF:BANDWIDTH=500000,RESOLUTION=640x360
/stream/video/{content_id}/360p/playlist.m3u8?token={token}

#EXT-X-STREAM-INF:BANDWIDTH=1500000,RESOLUTION=1280x720
/stream/video/{content_id}/720p/playlist.m3u8?token={token}

#EXT-X-STREAM-INF:BANDWIDTH=3000000,RESOLUTION=1920x1080
/stream/video/{content_id}/1080p/playlist.m3u8?token={token}"#
        )
    } else {
        // Тизер — только 360p
        format!(
            r#"#EXTM3U
#EXT-X-VERSION:3

#EXT-X-STREAM-INF:BANDWIDTH=500000,RESOLUTION=640x360
/stream/video/{content_id}/teaser/playlist.m3u8?token={token}"#
        )
    };

    Ok(playlist)
}

/// Получение сегмента из S3-хранилища
pub async fn get_segment_from_storage(
    config: &Config,
    content_id: Uuid,
    segment_path: &str,
) -> Result<Vec<u8>, AppError> {
    // В реальном коде — запрос к S3 через rust-s3:
    // let bucket = Bucket::new(
    //     &config.s3_bucket,
    //     Region::Custom { endpoint: config.s3_endpoint.clone(), region: config.s3_region.clone() },
    //     Credentials::new(&config.s3_access_key, &config.s3_secret_key, None, None, None)?,
    // )?;
    // let (data, code) = bucket.get_object(segment_path).await?;

    // Заглушка
    Ok(Vec::new())
}

/// Получение видео-сегмента с наложением водяного знака
pub async fn get_video_segment_with_watermark(
    config: &Config,
    pool: &PgPool,
    content_id: Uuid,
    segment_name: &str,
    user_id: Uuid,
) -> Result<Vec<u8>, AppError> {
    // Получение имени пользователя для водяного знака
    let display_name: String = sqlx::query_scalar(
        "SELECT display_name FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .unwrap_or_else(|| format!("user_{}", &user_id.to_string()[..8]));

    // В реальном коде:
    // 1. Получение сегмента из S3
    // 2. Наложение водяного знака через FFmpeg:
    //    ffmpeg -i segment.ts -vf "drawtext=text='{display_name}':fontsize=24:fontcolor=white@0.3:x=10:y=10" -c copy output.ts
    // 3. Возврат модифицированного сегмента

    let _watermark_command = format!(
        "ffmpeg -i segment.ts -vf \"drawtext=text='{}':fontsize=24:fontcolor=white@0.3:x=10:y=10\" -c copy output.ts",
        display_name
    );

    // Заглушка — возвращаем оригинальный сегмент
    get_segment_from_storage(config, content_id, segment_name).await
}
