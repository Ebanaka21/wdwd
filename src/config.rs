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
    pub cors_origin: String,
    pub s3_endpoint: String,
    pub s3_bucket: String,
    pub s3_access_key: String,
    pub s3_secret_key: String,
    pub s3_region: String,
    pub yookassa_shop_id: String,
    pub yookassa_secret_key: String,
    pub platform_commission_percent: f64,
    pub teaser_audio_seconds: u32,
    pub teaser_video_seconds: u32,
    pub max_sessions_per_user: u32,
    pub token_expiration_minutes: u32,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_password: String,
}

impl Config {
    pub fn from_env() -> Self {
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
            cors_origin: env::var("CORS_ORIGIN").unwrap_or_else(|_| "http://localhost:5173".to_string()),
            s3_endpoint: env::var("S3_ENDPOINT").expect("S3_ENDPOINT is required"),
            s3_bucket: env::var("S3_BUCKET").expect("S3_BUCKET is required"),
            s3_access_key: env::var("S3_ACCESS_KEY").expect("S3_ACCESS_KEY is required"),
            s3_secret_key: env::var("S3_SECRET_KEY").expect("S3_SECRET_KEY is required"),
            s3_region: env::var("S3_REGION").unwrap_or_else(|_| "ru-central1".to_string()),
            yookassa_shop_id: env::var("YOOKASSA_SHOP_ID").unwrap_or_default(),
            yookassa_secret_key: env::var("YOOKASSA_SECRET_KEY").unwrap_or_default(),
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
        }
    }
}
