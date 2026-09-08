use chrono::{Utc, Duration};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use sha2::{Sha256, Digest};
use sqlx::PgPool;
use uuid::Uuid;
use hex;

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
        "jti": Uuid::new_v4().to_string(), // Уникальный ID для отзыва
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
    let mut validation = Validation::default();
    validation.validate_exp = true;
    validation.required_spec_claims.insert("exp".to_string());

    let token_data = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &validation,
    ).map_err(|e| AppError::AuthError(format!("Token validation error: {}", e)))?;

    Ok(token_data.claims)
}

/// FIX: SHA-256 вместо DefaultHasher для хэширования токенов
/// DefaultHasher не криптографический и может коллизировать
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

/// FIX: Валидация политики паролей
pub fn validate_password(password: &str, min_length: usize) -> Result<(), AppError> {
    if password.len() < min_length {
        return Err(AppError::ValidationError(
            format!("Пароль должен быть не менее {} символов", min_length)
        ));
    }

    // Проверка на распространённые пароли (список можно расширить)
    let common_passwords = [
        "password", "12345678", "qwerty", "letmein", "admin",
        "welcome", "monkey", "dragon", "master", "login",
    ];
    if common_passwords.contains(&password.to_lowercase().as_str()) {
        return Err(AppError::ValidationError(
            "Слишком простой пароль. Используйте более сложный.".to_string()
        ));
    }

    Ok(())
}

/// FIX: Проверка account lockout (защита от брутфорса)
pub async fn check_account_lockout(
    pool: &PgPool,
    email: &str,
    max_attempts: u32,
    lockout_minutes: u32,
) -> Result<(), AppError> {
    let lockout_info: Option<(i32, chrono::DateTime<Utc>)> = sqlx::query_as(
        "SELECT failed_attempts, locked_until FROM users WHERE email = $1"
    )
    .bind(email)
    .fetch_optional(pool)
    .await?
    .and_then(|(attempts, locked_until)| {
        if attempts > 0 {
            Some((attempts, locked_until))
        } else {
            None
        }
    });

    if let Some((attempts, locked_until)) = lockout_info {
        if attempts as u32 >= max_attempts && locked_until > Utc::now() {
            let minutes_left = (locked_until - Utc::now()).num_minutes();
            return Err(AppError::AuthError(
                format!("Аккаунт заблокирован. Попробуйте через {} мин.", minutes_left.max(1))
            ));
        }
    }

    Ok(())
}

/// FIX: Запись неудачной попытки входа
pub async fn record_failed_login(pool: &PgPool, email: &str, lockout_minutes: u32) -> Result<(), AppError> {
    sqlx::query(
        r#"
        UPDATE users SET
            failed_attempts = failed_attempts + 1,
            locked_until = CASE
                WHEN failed_attempts + 1 >= 5 THEN NOW() + ($1 || ' minutes')::interval
                ELSE locked_until
            END
        WHERE email = $2
        "#
    )
    .bind(lockout_minutes.to_string())
    .bind(email)
    .execute(pool)
    .await?;

    Ok(())
}

/// FIX: Сброс счётчика неудачных попыток при успешном входе
pub async fn reset_failed_login(pool: &PgPool, email: &str) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE users SET failed_attempts = 0, locked_until = NOW() WHERE email = $1"
    )
    .bind(email)
    .execute(pool)
    .await?;

    Ok(())
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

    // FIX: Rotation — отзыв старого refresh token
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

/// FIX: Audit log — запись действий в системе
pub async fn audit_log(
    pool: &PgPool,
    user_id: Option<Uuid>,
    action: &str,
    resource_type: &str,
    resource_id: Option<Uuid>,
    ip_address: &str,
    user_agent: &str,
    metadata: Option<serde_json::Value>,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO audit_logs (id, user_id, action, resource_type, resource_id, ip_address, user_agent, metadata, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())
        "#
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(action)
    .bind(resource_type)
    .bind(resource_id)
    .bind(ip_address)
    .bind(user_agent)
    .bind(metadata)
    .execute(pool)
    .await?;

    Ok(())
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
