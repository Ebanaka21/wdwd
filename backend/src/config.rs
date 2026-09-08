use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub database_max_connections: u32,
    pub redis_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
    pub media_token_secret: String, // FIX: убран хардкод
    pub cors_origin: String,
    pub s3_endpoint: String,
    pub s3_bucket: String,
    pub s3_access_key: String,
    pub s3_secret_key: String,
    pub s3_region: String,
    pub yookassa_shop_id: String,
    pub yookassa_secret_key: String,
    pub yookassa_webhook_secret: String, // FIX: HMAC верификация webhook
    pub platform_commission_percent: f64,
    pub teaser_audio_seconds: u32,
    pub teaser_video_seconds: u32,
    pub max_sessions_per_user: u32,
    pub token_expiration_minutes: u32,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_password: String,
    // Rate limiting
    pub rate_limit_api: u32,
    pub rate_limit_auth: u32,
    pub rate_limit_media: u32,
    // Security
    pub password_min_length: usize,
    pub max_login_attempts: u32,
    pub lockout_duration_minutes: u32,
    pub cors_allowed_origins: Vec<String>, // FIX: валидация CORS
    // Telemetry
    pub otlp_endpoint: Option<String>,
    pub metrics_enabled: bool,
    // FFmpeg
    pub ffmpeg_path: String,
    pub ffprobe_path: String,
}

impl Config {
    pub fn from_env() -> Self {
        let cors_origin = env::var("CORS_ORIGIN").unwrap_or_else(|_| "http://localhost:5173".to_string());
        let cors_allowed_origins: Vec<String> = env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| cors_origin.clone())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Config {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT").unwrap_or_else(|_| "8080".to_string()).parse().unwrap(),
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL is required"),
            database_max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "20".to_string())
                .parse()
                .unwrap(),
            redis_url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET is required"),
            jwt_expiration_hours: env::var("JWT_EXPIRATION_HOURS")
                .unwrap_or_else(|_| "72".to_string())
                .parse()
                .unwrap(),
            media_token_secret: env::var("MEDIA_TOKEN_SECRET")
                .expect("MEDIA_TOKEN_SECRET is required — не используйте хардкод!"),
            cors_origin,
            cors_allowed_origins,
            s3_endpoint: env::var("S3_ENDPOINT").expect("S3_ENDPOINT is required"),
            s3_bucket: env::var("S3_BUCKET").expect("S3_BUCKET is required"),
            s3_access_key: env::var("S3_ACCESS_KEY").expect("S3_ACCESS_KEY is required"),
            s3_secret_key: env::var("S3_SECRET_KEY").expect("S3_SECRET_KEY is required"),
            s3_region: env::var("S3_REGION").unwrap_or_else(|_| "ru-central1".to_string()),
            yookassa_shop_id: env::var("YOOKASSA_SHOP_ID").unwrap_or_default(),
            yookassa_secret_key: env::var("YOOKASSA_SECRET_KEY").unwrap_or_default(),
            yookassa_webhook_secret: env::var("YOOKASSA_WEBHOOK_SECRET")
                .expect("YOOKASSA_WEBHOOK_SECRET is required для HMAC верификации"),
            platform_commission_percent: env::var("PLATFORM_COMMISSION_PERCENT")
                .unwrap_or_else(|_| "8.0".to_string())
                .parse()
                .unwrap(),
            teaser_audio_seconds: env::var("TEASER_AUDIO_SECONDS")
                .unwrap_or_else(|_| "15".to_string())
                .parse()
                .unwrap(),
            teaser_video_seconds: env::var("TEASER_VIDEO_SECONDS")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .unwrap(),
            max_sessions_per_user: env::var("MAX_SESSIONS_PER_USER")
                .unwrap_or_else(|_| "2".to_string())
                .parse()
                .unwrap(),
            token_expiration_minutes: env::var("TOKEN_EXPIRATION_MINUTES")
                .unwrap_or_else(|_| "120".to_string())
                .parse()
                .unwrap(),
            smtp_host: env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.yandex.ru".to_string()),
            smtp_port: env::var("SMTP_PORT")
                .unwrap_or_else(|_| "465".to_string())
                .parse()
                .unwrap(),
            smtp_user: env::var("SMTP_USER").unwrap_or_default(),
            smtp_password: env::var("SMTP_PASSWORD").unwrap_or_default(),
            // Rate limiting
            rate_limit_api: env::var("RATE_LIMIT_API").unwrap_or_else(|_| "100".to_string()).parse().unwrap(),
            rate_limit_auth: env::var("RATE_LIMIT_AUTH").unwrap_or_else(|_| "10".to_string()).parse().unwrap(),
            rate_limit_media: env::var("RATE_LIMIT_MEDIA").unwrap_or_else(|_| "30".to_string()).parse().unwrap(),
            // Security
            password_min_length: env::var("PASSWORD_MIN_LENGTH").unwrap_or_else(|_| "8".to_string()).parse().unwrap(),
            max_login_attempts: env::var("MAX_LOGIN_ATTEMPTS").unwrap_or_else(|_| "5".to_string()).parse().unwrap(),
            lockout_duration_minutes: env::var("LOCKOUT_DURATION_MINUTES").unwrap_or_else(|_| "15".to_string()).parse().unwrap(),
            // Telemetry
            otlp_endpoint: env::var("OTLP_ENDPOINT").ok(),
            metrics_enabled: env::var("METRICS_ENABLED").unwrap_or_else(|_| "true".to_string()).parse().unwrap(),
            // FFmpeg
            ffmpeg_path: env::var("FFMPEG_PATH").unwrap_or_else(|_| "ffmpeg".to_string()),
            ffprobe_path: env::var("FFPROBE_PATH").unwrap_or_else(|_| "ffprobe".to_string()),
        }
    }

    /// Валидация CORS origin
    pub fn is_origin_allowed(&self, origin: &str) -> bool {
        self.cors_allowed_origins.iter().any(|o| o == origin)
    }
}
