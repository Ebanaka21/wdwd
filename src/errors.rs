use thiserror::Error;
use actix_web::{HttpResponse, ResponseError};
use std::fmt;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Authorization error: {0}")]
    ForbiddenError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Payment error: {0}")]
    PaymentError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Internal server error: {0}")]
    InternalError(String),
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::DatabaseError(e) => {
                log::error!("Database error: {}", e);
                HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "internal_error".to_string(),
                    message: "Произошла внутренняя ошибка сервера".to_string(),
                })
            }
            AppError::AuthError(msg) => {
                HttpResponse::Unauthorized().json(ErrorResponse {
                    error: "unauthorized".to_string(),
                    message: msg.clone(),
                })
            }
            AppError::ForbiddenError(msg) => {
                HttpResponse::Forbidden().json(ErrorResponse {
                    error: "forbidden".to_string(),
                    message: msg.clone(),
                })
            }
            AppError::NotFound(msg) => {
                HttpResponse::NotFound().json(ErrorResponse {
                    error: "not_found".to_string(),
                    message: msg.clone(),
                })
            }
            AppError::ValidationError(msg) => {
                HttpResponse::BadRequest().json(ErrorResponse {
                    error: "validation_error".to_string(),
                    message: msg.clone(),
                })
            }
            AppError::PaymentError(msg) => {
                HttpResponse::BadRequest().json(ErrorResponse {
                    error: "payment_error".to_string(),
                    message: msg.clone(),
                })
            }
            AppError::StorageError(msg) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "storage_error".to_string(),
                    message: msg.clone(),
                })
            }
            AppError::RateLimitExceeded => {
                HttpResponse::TooManyRequests().json(ErrorResponse {
                    error: "rate_limit".to_string(),
                    message: "Превышен лимит запросов. Попробуйте позже.".to_string(),
                })
            }
            AppError::InternalError(msg) => {
                log::error!("Internal error: {}", msg);
                HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "internal_error".to_string(),
                    message: "Произошла внутренняя ошибка сервера".to_string(),
                })
            }
        }
    }
}

#[derive(serde::Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}
