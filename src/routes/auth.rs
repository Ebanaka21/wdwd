use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

use crate::config::Config;
use crate::errors::AppError;
use crate::models::user::*;
use crate::services::auth_service;

/// Конфигурация роутов аутентификации
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1/auth")
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
            .route("/refresh", web::post().to(refresh_token))
            .route("/logout", web::post().to(logout))
    );
}

/// POST /api/v1/auth/register
/// Регистрация нового пользователя (фанат или автор)
async fn register(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    body: web::Json<RegisterRequest>,
) -> Result<HttpResponse, AppError> {
    // Валидация
    if body.email.is_empty() || body.password.len() < 8 {
        return Err(AppError::ValidationError(
            "Email обязателен, пароль минимум 8 символов".to_string()
        ));
    }

    // Проверка, что email не занят
    let existing = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM users WHERE email = $1"
    )
    .bind(&body.email)
    .fetch_optional(pool.get_ref())
    .await?;

    if existing.is_some() {
        return Err(AppError::ValidationError(
            "Пользователь с таким email уже существует".to_string()
        ));
    }

    // Хеширование пароля
    let password_hash = bcrypt::hash(&body.password, 12)
        .map_err(|e| AppError::InternalError(format!("Hash error: {}", e)))?;

    let user_id = Uuid::new_v4();
    let now = Utc::now();

    // Создание пользователя
    sqlx::query(
        r#"
        INSERT INTO users (id, email, phone, password_hash, display_name, role, is_verified, is_active, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, false, true, $7, $7)
        "#
    )
    .bind(user_id)
    .bind(&body.email)
    .bind(&body.phone)
    .bind(&password_hash)
    .bind(&body.display_name)
    .bind(&body.role)
    .bind(now)
    .execute(pool.get_ref())
    .await?;

    // Если регистрируется автор — создаём запись в authors
    if body.role == UserRole::Author {
        let slug = generate_slug(&body.display_name);
        sqlx::query(
            r#"
            INSERT INTO authors (id, user_id, slug, display_name, is_verified, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, false, true, $5, $5)
            "#
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(&slug)
        .bind(&body.display_name)
        .bind(now)
        .execute(pool.get_ref())
        .await?;
    }

    // Генерация токенов
    let (token, refresh_token, expires_at) = auth_service::generate_tokens(
        user_id,
        &body.email,
        body.role.clone(),
        &config.jwt_secret,
        config.jwt_expiration_hours,
    )?;

    // Сохранение refresh token
    auth_service::save_refresh_token(pool.get_ref(), user_id, &refresh_token).await?;

    let user = UserPublic {
        id: user_id,
        display_name: body.display_name.clone(),
        email: body.email.clone(),
        role: body.role.clone(),
        avatar_url: None,
        is_verified: false,
        created_at: now,
    };

    Ok(HttpResponse::Created().json(AuthResponse {
        token,
        refresh_token,
        user,
        expires_at,
    }))
}

/// POST /api/v1/auth/login
/// Вход в систему
async fn login(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    body: web::Json<LoginRequest>,
) -> Result<HttpResponse, AppError> {
    // Поиск пользователя
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1 AND is_active = true"
    )
    .bind(&body.email)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::AuthError("Неверный email или пароль".to_string()))?;

    // Проверка пароля
    let valid = bcrypt::verify(&body.password, &user.password_hash)
        .map_err(|e| AppError::InternalError(format!("Verify error: {}", e)))?;

    if !valid {
        return Err(AppError::AuthError("Неверный email или пароль".to_string()));
    }

    // Проверка количества активных сессий
    let active_sessions = auth_service::count_active_sessions(pool.get_ref(), user.id).await?;
    if active_sessions >= config.max_sessions_per_user as i64 {
        // Удаляем самую старую сессию
        auth_service::remove_oldest_session(pool.get_ref(), user.id).await?;
    }

    // Генерация токенов
    let (token, refresh_token, expires_at) = auth_service::generate_tokens(
        user.id,
        &user.email,
        user.role.clone(),
        &config.jwt_secret,
        config.jwt_expiration_hours,
    )?;

    // Сохранение refresh token и сессии
    auth_service::save_refresh_token(pool.get_ref(), user.id, &refresh_token).await?;
    auth_service::create_session(pool.get_ref(), user.id, &token, &config).await?;

    // Обновление last_login_at
    sqlx::query("UPDATE users SET last_login_at = $1 WHERE id = $2")
        .bind(Utc::now())
        .bind(user.id)
        .execute(pool.get_ref())
        .await?;

    let user_public = UserPublic {
        id: user.id,
        display_name: user.display_name,
        email: user.email,
        role: user.role,
        avatar_url: user.avatar_url,
        is_verified: user.is_verified,
        created_at: user.created_at,
    };

    Ok(HttpResponse::Ok().json(AuthResponse {
        token,
        refresh_token,
        user: user_public,
        expires_at,
    }))
}

/// POST /api/v1/auth/refresh
/// Обновление access token по refresh token
async fn refresh_token(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    body: web::Json<RefreshTokenRequest>,
) -> Result<HttpResponse, AppError> {
    let (new_token, new_refresh, expires_at) = auth_service::refresh_tokens(
        pool.get_ref(),
        &body.refresh_token,
        &config.jwt_secret,
        config.jwt_expiration_hours,
    ).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "token": new_token,
        "refresh_token": new_refresh,
        "expires_at": expires_at,
    })))
}

/// POST /api/v1/auth/logout
/// Выход из системы (отзыв токенов)
async fn logout(
    pool: web::Data<PgPool>,
    body: web::Json<LogoutRequest>,
) -> Result<HttpResponse, AppError> {
    auth_service::revoke_refresh_token(pool.get_ref(), &body.refresh_token).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Выход выполнен" })))
}

#[derive(Deserialize)]
struct RefreshTokenRequest {
    refresh_token: String,
}

#[derive(Deserialize)]
struct LogoutRequest {
    refresh_token: String,
}

/// Генерация URL-friendly slug из имени
fn generate_slug(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("-")
}
