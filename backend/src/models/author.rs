use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Модель автора (расширение пользователя)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Author {
    pub id: Uuid,
    pub user_id: Uuid,
    pub slug: String, // URL-friendly идентификатор витрины
    pub display_name: String,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub cover_url: Option<String>,
    pub telegram_url: Option<String>,
    pub vk_url: Option<String>,
    pub youtube_url: Option<String>,
    pub is_verified: bool,
    pub is_active: bool,
    pub total_subscribers: i64,
    pub total_revenue_kopecks: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Публичная витрина автора (для гостей и подписчиков)
#[derive(Debug, Serialize, FromRow)]
pub struct AuthorShowcase {
    pub id: Uuid,
    pub slug: String,
    pub display_name: String,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub cover_url: Option<String>,
    pub telegram_url: Option<String>,
    pub vk_url: Option<String>,
    pub youtube_url: Option<String>,
    pub is_verified: bool,
    pub total_subscribers: i64,
    pub plans: Vec<PlanPublic>,
}

/// DTO для создания/обновления витрины
#[derive(Debug, Deserialize)]
pub struct UpdateAuthorRequest {
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub cover_url: Option<String>,
    pub telegram_url: Option<String>,
    pub vk_url: Option<String>,
    pub youtube_url: Option<String>,
}

/// Модель тарифного плана
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Plan {
    pub id: Uuid,
    pub author_id: Uuid,
    pub plan_type: PlanType,
    pub title: String,
    pub description: Option<String>,
    pub price_monthly_kopecks: i32,
    pub price_quarterly_kopecks: Option<i32>,
    pub price_yearly_kopecks: Option<i32>,
    pub is_enabled: bool,
    pub subscribers_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Тип плана (модуль подписки)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "plan_type", rename_all = "snake_case")]
pub enum PlanType {
    Music,
    Video,
    Combo,
}

/// Публичная информация о плане
#[derive(Debug, Serialize, FromRow)]
pub struct PlanPublic {
    pub id: Uuid,
    pub plan_type: PlanType,
    pub title: String,
    pub description: Option<String>,
    pub price_monthly_kopecks: i32,
    pub price_quarterly_kopecks: Option<i32>,
    pub price_yearly_kopecks: Option<i32>,
    pub subscribers_count: i64,
}

/// DTO для создания/обновления плана
#[derive(Debug, Deserialize)]
pub struct UpdatePlanRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub price_monthly_kopecks: Option<i32>,
    pub price_quarterly_kopecks: Option<i32>,
    pub price_yearly_kopecks: Option<i32>,
    pub is_enabled: Option<bool>,
}

/// Статистика автора
#[derive(Debug, Serialize)]
pub struct AuthorStats {
    pub total_subscribers: i64,
    pub active_subscribers: i64,
    pub monthly_revenue_kopecks: i64,
    pub total_revenue_kopecks: i64,
    pub conversion_rate: f64,
    pub top_tracks: Vec<TrackStat>,
    pub subscribers_by_plan: Vec<PlanSubscribers>,
    pub revenue_by_month: Vec<MonthlyRevenue>,
}

#[derive(Debug, Serialize)]
pub struct TrackStat {
    pub title: String,
    pub plays_count: i64,
}

#[derive(Debug, Serialize)]
pub struct PlanSubscribers {
    pub plan_type: PlanType,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct MonthlyRevenue {
    pub month: String,
    pub revenue_kopecks: i64,
}

/// Запрос на выплату
#[derive(Debug, Deserialize)]
pub struct PayoutRequest {
    pub amount_kopecks: i64,
    pub method: PayoutMethod,
    pub details: String,
}

#[derive(Debug, Deserialize, Serialize, sqlx::Type)]
#[sqlx(type_name = "payout_method", rename_all = "snake_case")]
pub enum PayoutMethod {
    BankAccount,
    CardMir,
    Sbp,
}

/// Запись о выплате
#[derive(Debug, Serialize, FromRow)]
pub struct Payout {
    pub id: Uuid,
    pub author_id: Uuid,
    pub amount_kopecks: i64,
    pub method: PayoutMethod,
    pub details: String,
    pub status: PayoutStatus,
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, sqlx::Type)]
#[sqlx(type_name = "payout_status", rename_all = "snake_case")]
pub enum PayoutStatus {
    Pending,
    Processing,
    Completed,
    Rejected,
}
