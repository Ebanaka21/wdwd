use chrono::{Utc, Duration};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::Config;
use crate::errors::AppError;
use crate::models::user::{JwtClaims, UserRole};

/// Генерация access и refresh токенов
pub fn generate_tokens(
    user_id: Uuid,
    email: &str,
    role: UserRole,
    jwt_secret: &str,
    expiration_hours: i64,
) -> Result<(String, String, chrono::DateTime<Utc>), AppError> {
    let now = Utc::now();
    let exp = now + Duration::hours(expiration_hours);

    let claims = JwtClaims {
        sub: user_id,
        email: email.to_string(),
        role,
        exp: exp.timestamp(),
        iat: now.timestamp(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    ).map_err(|e| AppError::InternalError(format!("JWT encode error: {}", e)))?;

    // Refresh token — отдельный JWT с большим сроком жизни
    let refresh_exp = now + Duration::days(30);
    let refresh_claims = serde_json::json!({
        "sub": user_id,
        "type": "refresh",
        "exp": refresh_exp.timestamp(),
        "iat": now.timestamp(),
    });

    let refresh_token = encode(
        &Header::default(),
        &refresh_claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    ).map_err(|e| AppError::InternalError(format!("JWT encode error: {}", e)))?;

    Ok((token, refresh_token, exp))
}

/// Валидация JWT токена
pub fn validate_token(token: &str, jwt_secret: &str) -> Result<JwtClaims, AppError> {
    let token_data = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::default(),
    ).map_err(|e| AppError::AuthError(format!("Token validation error: {}", e)))?;

    Ok(token_data.claims)
}

/// Сохранение refresh token в БД
pub async fn save_refresh_token(
    pool: &PgPool,
    user_id: Uuid,
    refresh_token: &str,
) -> Result<(), AppError> {
    let token_hash = hash_token(refresh_token);

    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at, is_revoked, created_at)
        VALUES ($1, $2, $3, $4, false, NOW())
        "#
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(&token_hash)
    .bind(Utc::now() + Duration::days(30))
    .execute(pool)
    .await?;

    Ok(())
}

/// Обновление токенов (refresh flow)
pub async fn refresh_tokens(
    pool: &PgPool,
    refresh_token: &str,
    jwt_secret: &str,
    expiration_hours: i64,
) -> Result<(String, String, chrono::DateTime<Utc>), AppError> {
    let token_hash = hash_token(refresh_token);

    // Проверка, что refresh token существует и не отозван
    let token_record = sqlx::query_as::<_, RefreshTokenRecord>(
        "SELECT id, user_id, expires_at, is_revoked FROM refresh_tokens WHERE token_hash = $1"
    )
    .bind(&token_hash)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::AuthError("Refresh token не найден".to_string()))?;

    if token_record.is_revoked {
        return Err(AppError::AuthError("Refresh token отозван".to_string()));
    }

    if token_record.expires_at < Utc::now() {
        return Err(AppError::AuthError("Refresh token истёк".to_string()));
    }

    // Отзыв старого refresh token
    sqlx::query("UPDATE refresh_tokens SET is_revoked = true WHERE id = $1")
        .bind(token_record.id)
        .execute(pool)
        .await?;

    // Получение данных пользователя
    let user = sqlx::query_as::<_, UserMinimal>(
        "SELECT id, email, role FROM users WHERE id = $1 AND is_active = true"
    )
    .bind(token_record.user_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::AuthError("Пользователь не найден или заблокирован".to_string()))?;

    // Генерация новых токенов
    let (new_token, new_refresh, expires_at) = generate_tokens(
        user.id,
        &user.email,
        user.role,
        jwt_secret,
        expiration_hours,
    )?;

    // Сохранение нового refresh token
    save_refresh_token(pool, user.id, &new_refresh).await?;

    Ok((new_token, new_refresh, expires_at))
}

/// Отзыв refresh token
pub async fn revoke_refresh_token(pool: &PgPool, refresh_token: &str) -> Result<(), AppError> {
    let token_hash = hash_token(refresh_token);

    sqlx::query("UPDATE refresh_tokens SET is_revoked = true WHERE token_hash = $1")
        .bind(&token_hash)
        .execute(pool)
        .await?;

    Ok(())
}

/// Подсчёт активных сессий
pub async fn count_active_sessions(pool: &PgPool, user_id: Uuid) -> Result<i64, AppError> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_sessions WHERE user_id = $1 AND is_active = true AND expires_at > NOW()"
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(count)
}

/// Удаление самой старой сессии
pub async fn remove_oldest_session(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
    sqlx::query(
        r#"
        UPDATE user_sessions SET is_active = false
        WHERE id = (
            SELECT id FROM user_sessions
            WHERE user_id = $1 AND is_active = true
            ORDER BY last_activity_at ASC
            LIMIT 1
        )
        "#
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Создание новой сессии
pub async fn create_session(
    pool: &PgPool,
    user_id: Uuid,
    token: &str,
    config: &Config,
) -> Result<(), AppError> {
    let token_hash = hash_token(token);
    let now = Utc::now();

    sqlx::query(
        r#"
        INSERT INTO user_sessions (id, user_id, token_hash, device_info, ip_address, is_active, created_at, last_activity_at, expires_at)
        VALUES ($1, $2, $3, '', '', true, $4, $4, $5)
        "#
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(&token_hash)
    .bind(now)
    .bind(now + Duration::hours(config.jwt_expiration_hours))
    .execute(pool)
    .await?;

    Ok(())
}

/// Хеширование токена для хранения
fn hash_token(token: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    token.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

// Вспомогательные структуры
#[derive(sqlx::FromRow)]
struct RefreshTokenRecord {
    id: Uuid,
    user_id: Uuid,
    expires_at: chrono::DateTime<Utc>,
    is_revoked: bool,
}

#[derive(sqlx::FromRow)]
struct UserMinimal {
    id: Uuid,
    email: String,
    role: UserRole,
}
