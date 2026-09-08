use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::author::PlanType;

/// Статус подписки
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "subscription_status", rename_all = "snake_case")]
pub enum SubscriptionStatus {
    Active,
    GracePeriod,    // Льготный период после неудачного платежа
    Paused,         // Заморозка
    Cancelled,
    Expired,
}

/// Период подписки
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "subscription_period", rename_all = "snake_case")]
pub enum SubscriptionPeriod {
    Monthly,
    Quarterly,
    Yearly,
}

/// Модель подписки
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Subscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub author_id: Uuid,
    pub plan_id: Uuid,
    pub plan_type: PlanType,
    pub status: SubscriptionStatus,
    pub period: SubscriptionPeriod,
    pub price_kopecks: i32,
    pub current_period_start: DateTime<Utc>,
    pub current_period_end: DateTime<Utc>,
    pub next_billing_date: DateTime<Utc>,
    pub auto_renew: bool,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub paused_at: Option<DateTime<Utc>>,
    pub pause_end_at: Option<DateTime<Utc>>,
    pub grace_period_end: Option<DateTime<Utc>>,
    pub consecutive_failures: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Публичная информация о подписке (для пользователя)
#[derive(Debug, Serialize, FromRow)]
pub struct SubscriptionPublic {
    pub id: Uuid,
    pub author_id: Uuid,
    pub author_name: String,
    pub author_avatar_url: Option<String>,
    pub author_slug: String,
    pub plan_type: PlanType,
    pub plan_title: String,
    pub status: SubscriptionStatus,
    pub price_kopecks: i32,
    pub next_billing_date: DateTime<Utc>,
    pub auto_renew: bool,
    pub current_period_end: DateTime<Utc>,
}

/// DTO для оформления подписки
#[derive(Debug, Deserialize)]
pub struct CreateSubscriptionRequest {
    pub author_id: Uuid,
    pub plan_id: Uuid,
    pub period: SubscriptionPeriod,
    pub payment_method_id: Option<Uuid>,
}

/// DTO для изменения подписки (добавление/удаление модуля)
#[derive(Debug, Deserialize)]
pub struct ModifySubscriptionRequest {
    pub subscription_id: Uuid,
    pub action: SubscriptionAction,
    pub new_plan_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub enum SubscriptionAction {
    Upgrade,        // Переход на более дорогой план (например, Music -> Combo)
    Downgrade,      // Переход на более дешёвый план
    Cancel,         // Отмена (доступ до конца периода)
    Pause,          // Заморозка на 1-3 месяца
    Resume,         // Разморозка
    ToggleAutoRenew,
}

/// DTO для заморозки подписки
#[derive(Debug, Deserialize)]
pub struct PauseSubscriptionRequest {
    pub subscription_id: Uuid,
    pub pause_months: i32, // 1, 2 или 3
}

/// Результат перерасчёта при изменении подписки
#[derive(Debug, Serialize)]
pub struct ProrationResult {
    pub remaining_days: i32,
    pub old_plan_daily_price_kopecks: i32,
    pub new_plan_daily_price_kopecks: i32,
    pub credit_kopecks: i32,     // Кредит от старого плана
    pub charge_kopecks: i32,     // Доплата за новый план
    pub net_amount_kopecks: i32, // Итого к доплате или возврату
}

/// Модель для проверки доступа к контенту
#[derive(Debug, Serialize)]
pub struct AccessCheck {
    pub has_access: bool,
    pub subscription_id: Option<Uuid>,
    pub access_type: AccessType,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub enum AccessType {
    Teaser,
    Full,
}
