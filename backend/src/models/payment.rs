use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Статус платежа
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "payment_status", rename_all = "snake_case")]
pub enum PaymentStatus {
    Pending,
    Processing,
    Succeeded,
    Failed,
    Refunded,
    Cancelled,
}

/// Тип платежа
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "payment_type", rename_all = "snake_case")]
pub enum PaymentType {
    Subscription,     // Оплата подписки
    SubscriptionRenewal, // Автопродление
    OneTime,          // Разовый платёж
    Proration,        // Перерасчёт при смене тарифа
}

/// Способ оплаты
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "payment_method_type", rename_all = "snake_case")]
pub enum PaymentMethodType {
    CardMir,
    CardVisa,
    CardMastercard,
    Sbp,
    Yoomoney,
}

/// Модель платежа
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Payment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub author_id: Uuid,
    pub subscription_id: Option<Uuid>,
    pub payment_type: PaymentType,
    pub amount_kopecks: i32,
    pub commission_kopecks: i32,
    pub author_amount_kopecks: i32,
    pub currency: String, // "RUB"
    pub status: PaymentStatus,
    pub payment_method: PaymentMethodType,
    pub external_id: Option<String>, // ID в платёжной системе (ЮKassa и т.д.)
    pub description: String,
    pub metadata: Option<serde_json::Value>,
    pub idempotency_key: String,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub refunded_at: Option<DateTime<Utc>>,
}

/// Модель сохранённого способа оплаты (привязанная карта)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PaymentMethod {
    pub id: Uuid,
    pub user_id: Uuid,
    pub method_type: PaymentMethodType,
    pub card_last_four: String,
    pub card_expiry: String, // MM/YY
    pub card_holder: Option<String>,
    pub is_default: bool,
    pub recurring_token: Option<String>, // Токен для рекуррентных платежей
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// DTO для создания платежа (инициация оплаты)
#[derive(Debug, Deserialize)]
pub struct CreatePaymentRequest {
    pub subscription_id: Option<Uuid>,
    pub author_id: Uuid,
    pub plan_id: Uuid,
    pub payment_method_id: Option<Uuid>,
    pub payment_method_type: PaymentMethodType,
    pub save_card: bool, // Сохранить карту для рекуррентных платежей
    pub return_url: String, // URL для возврата после оплаты
}

/// DTO для webhook от платёжной системы
#[derive(Debug, Deserialize)]
pub struct PaymentWebhook {
    pub event: String, // "payment.succeeded", "payment.cancelled", и т.д.
    pub payment_id: String,
    pub amount: WebhookAmount,
    pub status: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct WebhookAmount {
    pub value: String,
    pub currency: String,
}

/// Ответ при создании платежа (для редиректа на оплату)
#[derive(Debug, Serialize)]
pub struct PaymentResponse {
    pub payment_id: Uuid,
    pub confirmation_url: String, // URL платёжной формы
    pub amount_kopecks: i32,
    pub status: PaymentStatus,
}

/// Публичная информация о платеже (для истории пользователя)
#[derive(Debug, Serialize, FromRow)]
pub struct PaymentPublic {
    pub id: Uuid,
    pub author_name: String,
    pub payment_type: PaymentType,
    pub amount_kopecks: i32,
    pub status: PaymentStatus,
    pub payment_method: PaymentMethodType,
    pub card_last_four: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// DTO для запроса возврата
#[derive(Debug, Deserialize)]
pub struct RefundRequest {
    pub payment_id: Uuid,
    pub amount_kopecks: Option<i32>, // None = полный возврат
    pub reason: String,
}

/// Модель уведомления о предстоящем списании
#[derive(Debug, FromRow)]
pub struct BillingNotification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub subscription_id: Uuid,
    pub notification_type: BillingNotificationType,
    pub sent_at: Option<DateTime<Utc>>,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "billing_notification_type", rename_all = "snake_case")]
pub enum BillingNotificationType {
    UpcomingCharge,     // За 3 дня до списания
    ChargeSucceeded,    // Успешное списание
    ChargeFailed,       // Неудачное списание
    GracePeriodStart,   // Начало льготного периода
    SubscriptionExpiring, // Подписка скоро истечёт
}
