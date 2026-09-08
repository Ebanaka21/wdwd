use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Тип контента
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "content_type", rename_all = "snake_case")]
pub enum ContentType {
    Audio,
    Video,
}

/// Статус контента
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "content_status", rename_all = "snake_case")]
pub enum ContentStatus {
    Draft,
    Processing,
    Published,
    Archived,
}

/// Модель единицы контента (трек или видео)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Content {
    pub id: Uuid,
    pub author_id: Uuid,
    pub content_type: ContentType,
    pub title: String,
    pub description: Option<String>,
    pub cover_url: Option<String>,
    pub album: Option<String>, // Только для аудио
    pub duration_seconds: i32,
    pub file_url: String, // URL в S3 (зашифрованный)
    pub teaser_url: String, // URL тизера в S3
    pub teaser_start_seconds: i32,
    pub teaser_duration_seconds: i32,
    pub format: String, // FLAC, WAV, MP3, MP4, MOV
    pub file_size_bytes: i64,
    pub status: ContentStatus,
    pub is_downloadable: bool, // Разрешено ли скачивание подписчикам
    pub plays_count: i64,
    pub teaser_plays_count: i64,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
}

/// Публичная карточка контента (для витрины)
#[derive(Debug, Serialize, FromRow)]
pub struct ContentPublic {
    pub id: Uuid,
    pub content_type: ContentType,
    pub title: String,
    pub description: Option<String>,
    pub cover_url: Option<String>,
    pub album: Option<String>,
    pub duration_seconds: i32,
    pub format: String,
    pub teaser_url: String,
    pub teaser_start_seconds: i32,
    pub teaser_duration_seconds: i32,
    pub is_available: bool, // true если у пользователя есть подписка
    pub plays_count: i64,
}

/// DTO для загрузки контента
#[derive(Debug, Deserialize)]
pub struct CreateContentRequest {
    pub title: String,
    pub description: Option<String>,
    pub content_type: ContentType,
    pub album: Option<String>,
    pub teaser_start_seconds: Option<i32>,
    pub teaser_duration_seconds: Option<i32>,
    pub is_downloadable: Option<bool>,
}

/// DTO для обновления контента
#[derive(Debug, Deserialize)]
pub struct UpdateContentRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub cover_url: Option<String>,
    pub album: Option<String>,
    pub teaser_start_seconds: Option<i32>,
    pub teaser_duration_seconds: Option<i32>,
    pub is_downloadable: Option<bool>,
    pub status: Option<ContentStatus>,
    pub sort_order: Option<i32>,
}

/// Модель HLS-сегмента (для защищённого стриминга)
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct MediaSegment {
    pub id: Uuid,
    pub content_id: Uuid,
    pub segment_index: i32,
    pub segment_url: String,
    pub duration_ms: i32,
    pub encryption_key_id: String,
    pub is_teaser: bool,
}

/// Модель токена доступа к медиа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaAccessToken {
    pub token: String,
    pub user_id: Uuid,
    pub content_id: Uuid,
    pub is_full_access: bool, // true = полный контент, false = тизер
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Запрос на получение токена доступа
#[derive(Debug, Deserialize)]
pub struct MediaTokenRequest {
    pub content_id: Uuid,
    pub access_type: AccessType,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum AccessType {
    Teaser,
    Full,
}

/// Запись о прослушивании/просмотре
#[derive(Debug, FromRow)]
pub struct PlayEvent {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub content_id: Uuid,
    pub is_teaser: bool,
    pub played_seconds: i32,
    pub total_seconds: i32,
    pub completed: bool,
    pub device_info: String,
    pub ip_address: String,
    pub created_at: DateTime<Utc>,
}
