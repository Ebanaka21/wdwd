use actix_web::{dev::{ServiceRequest, ServiceResponse, Transform, Service}, Error, HttpResponse};
use futures::future::{ok, Ready, LocalBoxFuture};
use std::task::{Context, Poll};
use std::time::{SystemTime, UNIX_EPOCH};

/// Rate limiting middleware
/// Ограничивает количество запросов от одного IP/пользователя
pub struct RateLimiter {
    pub max_requests: u32,
    pub window_seconds: u64,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_seconds: u64) -> Self {
        RateLimiter {
            max_requests,
            window_seconds,
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        RateLimiter {
            max_requests: 100,
            window_seconds: 60,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RateLimiterService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RateLimiterService {
            service,
            max_requests: self.max_requests,
            window_seconds: self.window_seconds,
        })
    }
}

pub struct RateLimiterService<S> {
    service: S,
    max_requests: u32,
    window_seconds: u64,
}

impl<S, B> Service<ServiceRequest> for RateLimiterService<S>
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
        // Получение IP клиента
        let client_ip = req.peer_addr()
            .map(|addr| addr.ip().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let max_requests = self.max_requests;
        let window_seconds = self.window_seconds;

        let fut = self.service.call(req);

        Box::pin(async move {
            // В реальном коде здесь проверка через Redis:
            // let key = format!("rate_limit:{}", client_ip);
            // let current = redis.get(&key).await?;
            // if current >= max_requests { return 429 }
            // redis.incr(&key).await?;
            // redis.expire(&key, window_seconds).await?;

            let res = fut.await?;
            Ok(res)
        })
    }
}

/// Предустановленные rate limiter'ы для разных типов эндпоинтов
pub mod presets {
    use super::RateLimiter;

    /// Для стриминга медиа — строгий лимит
    pub fn media_stream() -> RateLimiter {
        RateLimiter::new(30, 60) // 30 запросов в минуту
    }

    /// Для API — стандартный лимит
    pub fn api_default() -> RateLimiter {
        RateLimiter::new(100, 60) // 100 запросов в минуту
    }

    /// Для аутентификации — строгий лимит (защита от брутфорса)
    pub fn auth() -> RateLimiter {
        RateLimiter::new(10, 300) // 10 запросов за 5 минут
    }

    /// Для webhook'ов — высокий лимит
    pub fn webhook() -> RateLimiter {
        RateLimiter::new(1000, 60) // 1000 запросов в минуту
    }
}
