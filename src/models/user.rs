use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Роль пользователя в системе
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "snake_case")]
pub enum UserRole {
    Guest,
    Fan,
    Author,
    Admin,
}

/// Модель пользователя (универсальная — может быть фанатом, автором или админом)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub phone: Option<String>,
    pub password_hash: String,
    pub display_name: String,
    pub role: UserRole,
    pub avatar_url: Option<String>,
    pub is_verified: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

/// DTO для регистрации
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub display_name: String,
    pub phone: Option<String>,
    pub role: UserRole, // Fan или Author
}

/// DTO для входа
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// DTO ответа с токеном
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub refresh_token: String,
    pub user: UserPublic,
    pub expires_at: DateTime<Utc>,
}

/// Публичная информация о пользователе
#[derive(Debug, Serialize, FromRow)]
pub struct UserPublic {
    pub id: Uuid,
    pub display_name: String,
    pub email: String,
    pub role: UserRole,
    pub avatar_url: Option<String>,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
}

/// Обновление профиля
#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub phone: Option<String>,
    pub avatar_url: Option<String>,
}

/// Claims для JWT
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: Uuid,
    pub email: String,
    pub role: UserRole,
    pub exp: i64,
    pub iat: i64,
}
