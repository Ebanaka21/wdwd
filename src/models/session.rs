use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Модель активной сессии пользователя
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String, // Хэш JWT для быстрой проверки
    pub device_info: String, // User-Agent
    pub ip_address: String,
    pub fingerprint: Option<String>, // Отпечаток браузера
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_activity_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Информация об устройстве
#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub user_agent: String,
    pub os: Option<String>,
    pub browser: Option<String>,
    pub device_type: DeviceType,
    pub ip_address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DeviceType {
    Desktop,
    Mobile,
    Tablet,
    Unknown,
}

/// DTO для проверки сессий
#[derive(Debug, Serialize)]
pub struct SessionCheckResult {
    pub is_allowed: bool,
    pub active_sessions: i32,
    pub max_sessions: i32,
    pub current_session: SessionInfo,
}

#[derive(Debug, Serialize, FromRow)]
pub struct SessionInfo {
    pub id: Uuid,
    pub device_info: String,
    pub ip_address: String,
    pub created_at: DateTime<Utc>,
    pub last_activity_at: DateTime<Utc>,
}

/// Модель refresh token
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub is_revoked: bool,
    pub created_at: DateTime<Utc>,
}
