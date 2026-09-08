use actix_web::{dev::{ServiceRequest, ServiceResponse, Transform, Service}, Error, HttpMessage, HttpResponse};
use futures::future::{ok, Ready, LocalBoxFuture};
use std::task::{Context, Poll};

use crate::config::Config;
use crate::models::user::{JwtClaims, UserRole};
use crate::services::auth_service;

/// Middleware для проверки JWT-токена
/// Извлекает токен из Authorization header, валидирует и добавляет claims в request extensions
pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareService { service })
    }
}

pub struct AuthMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Извлечение токена из Authorization header
        let auth_header = req.headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let config = req.app_data::<actix_web::web::Data<Config>>()
            .map(|c| c.get_ref().clone());

        let fut = self.service.call(req);

        Box::pin(async move {
            // Если нет config, пропускаем (не должно происходить)
            let config = match config {
                Some(c) => c,
                None => {
                    let res = HttpResponse::InternalServerError().finish().map_into_right_body();
                    return Ok(ServiceResponse::new(
                        fut.await?.request().clone(),
                        res.map_into_left_body(),
                    ));
                }
            };

            match auth_header {
                Some(header) if header.starts_with("Bearer ") => {
                    let token = &header[7..];
                    match auth_service::validate_token(token, &config.jwt_secret) {
                        Ok(claims) => {
                            // Токен валиден — продолжаем
                            let res = fut.await?;
                            Ok(res)
                        }
                        Err(_) => {
                            let res = HttpResponse::Unauthorized()
                                .json(serde_json::json!({
                                    "error": "unauthorized",
                                    "message": "Невалидный или истёкший токен"
                                }));
                            // Возвращаем ошибку
                            Err(actix_web::error::ErrorUnauthorized("Invalid token"))
                        }
                    }
                }
                _ => {
                    Err(actix_web::error::ErrorUnauthorized("Missing authorization header"))
                }
            }
        })
    }
}

/// Проверка роли пользователя
pub fn require_role(claims: &JwtClaims, required_role: UserRole) -> Result<(), String> {
    match (&claims.role, &required_role) {
        (UserRole::Admin, _) => Ok(()), // Админ имеет доступ ко всему
        (UserRole::Author, UserRole::Author) => Ok(()),
        (UserRole::Fan, UserRole::Fan) => Ok(()),
        _ => Err("Недостаточно прав".to_string()),
    }
}
