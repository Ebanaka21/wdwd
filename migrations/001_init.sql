-- =============================================
-- Чистовик — Инициализация базы данных
-- PostgreSQL 15+
-- =============================================

-- Расширения
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- =============================================
-- Типы (ENUM)
-- =============================================

CREATE TYPE user_role AS ENUM ('guest', 'fan', 'author', 'admin');
CREATE TYPE plan_type AS ENUM ('music', 'video', 'combo');
CREATE TYPE content_type AS ENUM ('audio', 'video');
CREATE TYPE content_status AS ENUM ('draft', 'processing', 'published', 'archived');
CREATE TYPE subscription_status AS ENUM ('active', 'grace_period', 'paused', 'cancelled', 'expired');
CREATE TYPE subscription_period AS ENUM ('monthly', 'quarterly', 'yearly');
CREATE TYPE payment_status AS ENUM ('pending', 'processing', 'succeeded', 'failed', 'refunded', 'cancelled');
CREATE TYPE payment_type AS ENUM ('subscription', 'subscription_renewal', 'one_time', 'proration');
CREATE TYPE payment_method_type AS ENUM ('card_mir', 'card_visa', 'card_mastercard', 'sbp', 'yoomoney');
CREATE TYPE payout_method AS ENUM ('bank_account', 'card_mir', 'sbp');
CREATE TYPE payout_status AS ENUM ('pending', 'processing', 'completed', 'rejected');
CREATE TYPE billing_notification_type AS ENUM ('upcoming_charge', 'charge_succeeded', 'charge_failed', 'grace_period_start', 'subscription_expiring');

-- =============================================
-- Пользователи
-- =============================================

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) NOT NULL UNIQUE,
    phone VARCHAR(20),
    password_hash VARCHAR(255) NOT NULL,
    display_name VARCHAR(100) NOT NULL,
    role user_role NOT NULL DEFAULT 'fan',
    avatar_url TEXT,
    is_verified BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login_at TIMESTAMPTZ
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_role ON users(role);
CREATE INDEX idx_users_is_active ON users(is_active);

-- =============================================
-- Авторы
-- =============================================

CREATE TABLE authors (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    slug VARCHAR(100) NOT NULL UNIQUE,
    display_name VARCHAR(100) NOT NULL,
    description TEXT,
    avatar_url TEXT,
    cover_url TEXT,
    telegram_url VARCHAR(255),
    vk_url VARCHAR(255),
    youtube_url VARCHAR(255),
    is_verified BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    total_subscribers BIGINT NOT NULL DEFAULT 0,
    total_revenue_kopecks BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_authors_slug ON authors(slug);
CREATE INDEX idx_authors_user_id ON authors(user_id);
CREATE INDEX idx_authors_is_active ON authors(is_active);

-- =============================================
-- Тарифные планы
-- =============================================

CREATE TABLE plans (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    author_id UUID NOT NULL REFERENCES authors(id) ON DELETE CASCADE,
    plan_type plan_type NOT NULL,
    title VARCHAR(100) NOT NULL,
    description TEXT,
    price_monthly_kopecks INTEGER NOT NULL,
    price_quarterly_kopecks INTEGER,
    price_yearly_kopecks INTEGER,
    is_enabled BOOLEAN NOT NULL DEFAULT true,
    subscribers_count BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(author_id, plan_type)
);

CREATE INDEX idx_plans_author_id ON plans(author_id);
CREATE INDEX idx_plans_is_enabled ON plans(is_enabled);

-- =============================================
-- Контент (треки и видео)
-- =============================================

CREATE TABLE contents (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    author_id UUID NOT NULL REFERENCES authors(id) ON DELETE CASCADE,
    content_type content_type NOT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    cover_url TEXT,
    album VARCHAR(255),
    duration_seconds INTEGER NOT NULL DEFAULT 0,
    file_url TEXT NOT NULL DEFAULT '',
    teaser_url TEXT NOT NULL DEFAULT '',
    format VARCHAR(20) NOT NULL DEFAULT '',
    file_size_bytes BIGINT NOT NULL DEFAULT 0,
    teaser_start_seconds INTEGER NOT NULL DEFAULT 0,
    teaser_duration_seconds INTEGER NOT NULL DEFAULT 15,
    status content_status NOT NULL DEFAULT 'draft',
    is_downloadable BOOLEAN NOT NULL DEFAULT false,
    plays_count BIGINT NOT NULL DEFAULT 0,
    teaser_plays_count BIGINT NOT NULL DEFAULT 0,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published_at TIMESTAMPTZ
);

CREATE INDEX idx_contents_author_id ON contents(author_id);
CREATE INDEX idx_contents_status ON contents(status);
CREATE INDEX idx_contents_type ON contents(content_type);
CREATE INDEX idx_contents_sort ON contents(author_id, sort_order);

-- =============================================
-- Медиа-сегменты (HLS)
-- =============================================

CREATE TABLE media_segments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    content_id UUID NOT NULL REFERENCES contents(id) ON DELETE CASCADE,
    segment_index INTEGER NOT NULL,
    segment_url TEXT NOT NULL,
    duration_ms INTEGER NOT NULL,
    encryption_key_id VARCHAR(100) NOT NULL,
    is_teaser BOOLEAN NOT NULL DEFAULT false
);

CREATE INDEX idx_media_segments_content ON media_segments(content_id);

-- =============================================
-- Подписки
-- =============================================

CREATE TABLE subscriptions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    author_id UUID NOT NULL REFERENCES authors(id) ON DELETE CASCADE,
    plan_id UUID NOT NULL REFERENCES plans(id),
    plan_type plan_type NOT NULL,
    status subscription_status NOT NULL DEFAULT 'active',
    period subscription_period NOT NULL DEFAULT 'monthly',
    price_kopecks INTEGER NOT NULL,
    current_period_start TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    current_period_end TIMESTAMPTZ NOT NULL,
    next_billing_date TIMESTAMPTZ NOT NULL,
    auto_renew BOOLEAN NOT NULL DEFAULT true,
    cancelled_at TIMESTAMPTZ,
    paused_at TIMESTAMPTZ,
    pause_end_at TIMESTAMPTZ,
    grace_period_end TIMESTAMPTZ,
    consecutive_failures INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_subscriptions_user ON subscriptions(user_id);
CREATE INDEX idx_subscriptions_author ON subscriptions(author_id);
CREATE INDEX idx_subscriptions_status ON subscriptions(status);
CREATE INDEX idx_subscriptions_billing ON subscriptions(next_billing_date, auto_renew, status);
CREATE INDEX idx_subscriptions_active ON subscriptions(user_id, author_id, status) WHERE status IN ('active', 'grace_period');

-- =============================================
-- Платежи
-- =============================================

CREATE TABLE payments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id),
    author_id UUID NOT NULL REFERENCES authors(id),
    subscription_id UUID REFERENCES subscriptions(id),
    payment_type payment_type NOT NULL DEFAULT 'subscription',
    amount_kopecks INTEGER NOT NULL,
    commission_kopecks INTEGER NOT NULL DEFAULT 0,
    author_amount_kopecks INTEGER NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'RUB',
    status payment_status NOT NULL DEFAULT 'pending',
    payment_method payment_method_type NOT NULL,
    external_id VARCHAR(255),
    description TEXT NOT NULL DEFAULT '',
    metadata JSONB,
    idempotency_key VARCHAR(100) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    refunded_at TIMESTAMPTZ
);

CREATE INDEX idx_payments_user ON payments(user_id);
CREATE INDEX idx_payments_author ON payments(author_id);
CREATE INDEX idx_payments_status ON payments(status);
CREATE INDEX idx_payments_external ON payments(external_id);
CREATE INDEX idx_payments_monthly ON payments(author_id, created_at) WHERE status = 'succeeded';

-- =============================================
-- Способы оплаты (сохранённые карты)
-- =============================================

CREATE TABLE payment_methods (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    method_type payment_method_type NOT NULL,
    card_last_four VARCHAR(4) NOT NULL,
    card_expiry VARCHAR(5) NOT NULL,
    card_holder VARCHAR(100),
    is_default BOOLEAN NOT NULL DEFAULT false,
    recurring_token VARCHAR(255),
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_payment_methods_user ON payment_methods(user_id);

-- =============================================
-- Выплаты авторам
-- =============================================

CREATE TABLE payouts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    author_id UUID NOT NULL REFERENCES authors(id),
    amount_kopecks BIGINT NOT NULL,
    method payout_method NOT NULL,
    details TEXT NOT NULL DEFAULT '',
    status payout_status NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed_at TIMESTAMPTZ
);

CREATE INDEX idx_payouts_author ON payouts(author_id);
CREATE INDEX idx_payouts_status ON payouts(status);

-- =============================================
-- Сессии пользователей
-- =============================================

CREATE TABLE user_sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    device_info TEXT NOT NULL DEFAULT '',
    ip_address VARCHAR(45) NOT NULL DEFAULT '',
    fingerprint VARCHAR(255),
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_activity_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_sessions_user ON user_sessions(user_id, is_active);
CREATE INDEX idx_sessions_expires ON user_sessions(expires_at);

-- =============================================
-- Refresh токены
-- =============================================

CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    is_revoked BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_refresh_tokens_hash ON refresh_tokens(token_hash);
CREATE INDEX idx_refresh_tokens_user ON refresh_tokens(user_id, is_revoked);

-- =============================================
-- События воспроизведения
-- =============================================

CREATE TABLE play_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id),
    content_id UUID NOT NULL REFERENCES contents(id),
    is_teaser BOOLEAN NOT NULL DEFAULT false,
    played_seconds INTEGER NOT NULL DEFAULT 0,
    total_seconds INTEGER NOT NULL DEFAULT 0,
    completed BOOLEAN NOT NULL DEFAULT false,
    device_info TEXT NOT NULL DEFAULT '',
    ip_address VARCHAR(45) NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_play_events_content ON play_events(content_id);
CREATE INDEX idx_play_events_user ON play_events(user_id);
CREATE INDEX idx_play_events_daily ON play_events(content_id, created_at);

-- =============================================
-- Жалобы на контент (модерация)
-- =============================================

CREATE TABLE content_reports (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    content_id UUID NOT NULL REFERENCES contents(id),
    reporter_id UUID REFERENCES users(id),
    reason VARCHAR(100) NOT NULL,
    description TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ
);

CREATE INDEX idx_reports_status ON content_reports(status);
CREATE INDEX idx_reports_content ON content_reports(content_id);

-- =============================================
-- Уведомления о биллинге
-- =============================================

CREATE TABLE billing_notifications (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    subscription_id UUID NOT NULL REFERENCES subscriptions(id),
    notification_type billing_notification_type NOT NULL,
    sent_at TIMESTAMPTZ,
    is_read BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_billing_notifications_user ON billing_notifications(user_id, is_read);

-- =============================================
-- Настройки платформы
-- =============================================

CREATE TABLE platform_settings (
    key VARCHAR(100) PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Начальные настройки
INSERT INTO platform_settings (key, value) VALUES
    ('commission_percent', '8.0'),
    ('max_sessions_per_user', '2'),
    ('teaser_audio_seconds', '15'),
    ('teaser_video_seconds', '60'),
    ('grace_period_days', '5'),
    ('renewal_reminder_days', '3');
