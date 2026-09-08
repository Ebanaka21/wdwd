use actix_web::{dev::{ServiceRequest, ServiceResponse, Transform, Service}, Error, HttpResponse};
use futures::future::{ok, Ready, LocalBoxFuture};
use std::task::{Context, Poll};

/// Middleware для добавления security headers
/// Защита от XSS, clickjacking, MIME sniffing и т.д.
pub struct SecurityHeaders;

impl<S, B> Transform<S, ServiceRequest> for SecurityHeaders
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = SecurityHeadersService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(SecurityHeadersService { service })
    }
}

pub struct SecurityHeadersService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for SecurityHeadersService<S>
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
        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;

            // Content Security Policy — защита от XSS
            res.headers_mut().insert(
                actix_web::http::header::HeaderName::from_static("content-security-policy"),
                actix_web::http::header::HeaderValue::from_str(
                    "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com; img-src 'self' data: https:; media-src 'self' https://storage.yandexcloud.net; connect-src 'self' https://api.yookassa.ru; frame-ancestors 'none';"
                ).unwrap(),
            );

            // Защита от clickjacking
            res.headers_mut().insert(
                actix_web::http::header::X_FRAME_OPTIONS,
                actix_web::http::header::HeaderValue::from_static("DENY"),
            );

            // Защита от MIME type sniffing
            res.headers_mut().insert(
                actix_web::http::header::HeaderName::from_static("x-content-type-options"),
                actix_web::http::header::HeaderValue::from_static("nosniff"),
            );

            // Referrer Policy
            res.headers_mut().insert(
                actix_web::http::header::HeaderName::from_static("referrer-policy"),
                actix_web::http::header::HeaderValue::from_static("strict-origin-when-cross-origin"),
            );

            // Permissions Policy (бывший Feature-Policy)
            res.headers_mut().insert(
                actix_web::http::header::HeaderName::from_static("permissions-policy"),
                actix_web::http::header::HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
            );

            // Strict Transport Security (HSTS)
            res.headers_mut().insert(
                actix_web::http::header::HeaderName::from_static("strict-transport-security"),
                actix_web::http::header::HeaderValue::from_static("max-age=31536000; includeSubDomains"),
            );

            // Запрет кеширования для API ответов
            if res.request().path().starts_with("/api/") {
                res.headers_mut().insert(
                    actix_web::http::header::CACHE_CONTROL,
                    actix_web::http::header::HeaderValue::from_static("no-store, no-cache, must-revalidate, private"),
                );
                res.headers_mut().insert(
                    actix_web::http::header::HeaderName::from_static("pragma"),
                    actix_web::http::header::HeaderValue::from_static("no-cache"),
                );
            }

            Ok(res)
        })
    }
}
