use chrono::{Utc, Duration};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;
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
    pub jti: String, // Уникальный ID для аудита
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

/// FIX: Генерация медиа-токена с секретом из env (не хардкод!)
pub fn generate_media_token(
    user_id: Uuid,
    content_id: Uuid,
    is_full_access: bool,
    expiration_minutes: u32,
    secret: &str, // FIX: передаётся из config
) -> Result<String, AppError> {
    let now = Utc::now();
    let exp = now + Duration::minutes(expiration_minutes as i64);

    let claims = MediaTokenClaims {
        user_id,
        content_id,
        is_full_access,
        exp: exp.timestamp(),
        iat: now.timestamp(),
        jti: Uuid::new_v4().to_string(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    ).map_err(|e| AppError::InternalError(format!("Media token error: {}", e)))?;

    Ok(token)
}

/// FIX: Валидация медиа-токена с секретом из env
pub fn validate_media_token(token: &str, secret: &str) -> Result<MediaTokenClaims, AppError> {
    let mut validation = Validation::default();
    validation.validate_exp = true;

    let token_data = decode::<MediaTokenClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    ).map_err(|_| AppError::AuthError("Невалидный медиа-токен".to_string()))?;

    Ok(token_data.claims)
}

/// FIX: Реальный rate limiting через Redis
pub async fn check_rate_limit(
    redis: &redis::Client,
    rate_key: &str,
    max_requests: u32,
    window_seconds: u64,
) -> Result<(), AppError> {
    let mut conn = redis.get_multiplexed_async_connection()
        .await
        .map_err(|e| AppError::InternalError(format!("Redis connection error: {}", e)))?;

    // INCR + EXPIRE атомарно через Lua script
    let script = redis::Script::new(r#"
        local current = redis.call('INCR', KEYS[1])
        if current == 1 then
            redis.call('EXPIRE', KEYS[1], ARGV[1])
        end
        return current
    "#);

    let current: u32 = script
        .key(rate_key)
        .arg(window_seconds)
        .invoke_async(&mut conn)
        .await
        .map_err(|e| AppError::InternalError(format!("Redis error: {}", e)))?;

    if current > max_requests {
        return Err(AppError::RateLimitExceeded);
    }

    Ok(())
}

/// FIX: Реальное получение сегмента из S3
pub async fn get_segment_from_storage(
    config: &Config,
    content_id: Uuid,
    segment_path: &str,
) -> Result<Vec<u8>, AppError> {
    let region = Region::Custom {
        endpoint: config.s3_endpoint.clone(),
        region: config.s3_region.clone(),
    };

    let credentials = Credentials::new(
        Some(&config.s3_access_key),
        Some(&config.s3_secret_key),
        None,
        None,
        None,
    ).map_err(|e| AppError::StorageError(format!("S3 credentials error: {}", e)))?;

    let bucket = Bucket::new(
        &config.s3_bucket,
        region,
        credentials,
    ).map_err(|e| AppError::StorageError(format!("S3 bucket error: {}", e)))?;

    let object_key = format!("content/{}/{}", content_id, segment_path);

    let response = bucket.get_object(&object_key)
        .await
        .map_err(|e| AppError::StorageError(format!("S3 get error: {}", e)))?;

    if response.response_code() != 200 {
        return Err(AppError::StorageError(format!(
            "S3 returned status {}", response.response_code()
        )));
    }

    Ok(response.to_vec())
}

/// FIX: Реальное наложение водяных знаков через FFmpeg
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

    // Получение оригинального сегмента из S3
    let original_segment = get_segment_from_storage(config, content_id, segment_name).await?;

    // Создание временных файлов
    let temp_dir = std::env::temp_dir();
    let input_path = temp_dir.join(format!("input_{}.ts", Uuid::new_v4()));
    let output_path = temp_dir.join(format!("output_{}.ts", Uuid::new_v4()));

    // Запись оригинального сегмента
    tokio::fs::write(&input_path, &original_segment)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to write temp file: {}", e)))?;

    // Наложение водяного знака через FFmpeg
    let watermark_text = format!("{} · {}", display_name, &user_id.to_string()[..8]);
    let output = tokio::process::Command::new(&config.ffmpeg_path)
        .args(&[
            "-i", input_path.to_str().unwrap(),
            "-vf", &format!(
                "drawtext=text='{}':fontsize=18:fontcolor=white@0.4:x=10:y=10:borderw=1:bordercolor=black@0.3",
                watermark_text
            ),
            "-c:v", "libx264",
            "-preset", "ultrafast",
            "-c:a", "copy",
            "-f", "mpegts",
            output_path.to_str().unwrap(),
            "-y",
        ])
        .output()
        .await
        .map_err(|e| AppError::InternalError(format!("FFmpeg error: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::InternalError(format!("FFmpeg failed: {}", stderr)));
    }

    // Чтение результата
    let watermarked_segment = tokio::fs::read(&output_path)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to read output: {}", e)))?;

    // Очистка временных файлов
    let _ = tokio::fs::remove_file(input_path).await;
    let _ = tokio::fs::remove_file(output_path).await;

    Ok(watermarked_segment)
}

/// Генерация HLS master playlist для аудио
pub async fn generate_audio_playlist(
    pool: &PgPool,
    content_id: Uuid,
    is_full_access: bool,
    token: &str,
) -> Result<String, AppError> {
    let _file_url: String = sqlx::query_scalar(
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
    _pool: &PgPool,
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
