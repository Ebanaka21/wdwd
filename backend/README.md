# Чистовик — Backend API

Backend для платформы закрытого контента «Чистовик».

## Технологический стек

- **Язык:** Rust
- **Web-фреймворк:** Actix-web 4
- **База данных:** PostgreSQL 15+ (через SQLx)
- **Кэш / Rate limiting:** Redis
- **Хранилище файлов:** S3-совместимое (Yandex Cloud / Selectel)
- **Платежи:** ЮKassa
- **Аутентификация:** JWT + Refresh Token Rotation
- **Медиа:** FFmpeg (HLS, транскодирование, водяные знаки)

## Безопасность

### Аутентификация
- JWT с валидацией expiration
- Refresh Token Rotation (каждый refresh отзывет старый)
- SHA-256 для хэширования токенов (не DefaultHasher!)
- Account lockout после N неудачных попыток
- Валидация политики паролей

### Защита API
- Security Headers: CSP, X-Frame-Options, HSTS, X-Content-Type-Options
- CORS с валидацией origin
- Rate limiting через Redis (Lua script для атомарности)
- Constant-time comparison для HMAC (защита от timing attacks)

### Защита платежей
- HMAC-SHA256 верификация webhook от ЮKassa
- Idempotency key для предотвращения дублей
- Audit log всех финансовых операций

### Защита медиа
- HLS с AES-128 шифрованием
- Временные токены доступа (2 часа)
- Водяные знаки через FFmpeg (user_id + display_name)
- Rate limiting на стриминг (30 запросов/мин)
- Проверка Referer (защита от хотлинкинга)
- Реальное S3-хранилище через rust-s3

### Аудит
- Audit log всех критичных действий
- Запись failed login attempts
- Отслеживание сессий

## Структура проекта

```
backend/
├── src/
│   ├── main.rs              # Точка входа + graceful shutdown
│   ├── config.rs             # Конфигурация из env
│   ├── errors.rs             # Обработка ошибок
│   ├── tests.rs              # Unit tests
│   ├── models/               # Модели данных
│   │   ├── user.rs           # Пользователи, JWT
│   │   ├── author.rs         # Авторы, тарифы, выплаты
│   │   ├── content.rs        # Контент, сегменты, токены
│   │   ├── subscription.rs   # Подписки, перерасчёт
│   │   ├── payment.rs        # Платежи, способы оплаты
│   │   └── session.rs        # Сессии, устройства
│   ├── routes/               # API-эндпоинты
│   │   ├── auth.rs           # Регистрация, вход, refresh
│   │   ├── showcase.rs       # Публичная витрина
│   │   ├── author.rs         # ЛК автора
│   │   ├── content.rs        # Управление контентом
│   │   ├── subscription.rs   # Подписки
│   │   ├── payment.rs        # Платежи, webhook с HMAC
│   │   ├── user.rs           # ЛК пользователя
│   │   ├── admin.rs          # Админ-панель
│   │   └── streaming.rs      # HLS-стриминг с водяными знаками
│   ├── middleware/            # Middleware
│   │   ├── auth.rs           # JWT-аутентификация
│   │   ├── rate_limit.rs     # Rate limiting (Redis)
│   │   └── security.rs       # Security headers (CSP, HSTS)
│   └── services/             # Бизнес-логика
│       ├── auth_service.rs   # JWT, SHA-256, password policy
│       ├── content_service.rs # FFmpeg, транскодирование
│       ├── payment_service.rs # ЮKassa, HMAC webhook
│       └── streaming_service.rs # HLS, S3, водяные знаки
├── migrations/
│   ├── 001_init.sql          # Схема БД
│   └── 002_security.sql      # Audit logs, account lockout
├── Cargo.toml
├── .env.example
└── README.md
```

## Запуск

```bash
# 1. Установить зависимости
cargo build

# 2. Настроить .env
cp .env.example .env
# Заполнить .env (особенно секреты!)

# 3. Запустить PostgreSQL и Redis
docker-compose up -d postgres redis

# 4. Запустить сервер
cargo run

# 5. Запустить тесты
cargo test
```

## Переменные окружения

### Критичные (обязательны)
- `DATABASE_URL` — подключение к PostgreSQL
- `JWT_SECRET` — секрет для JWT (мин. 32 символа)
- `MEDIA_TOKEN_SECRET` — секрет для медиа-токенов (НЕ хардкод!)
- `YOOKASSA_WEBHOOK_SECRET` — HMAC секрет для webhook

### Безопасность
- `PASSWORD_MIN_LENGTH` — мин. длина пароля (по умолчанию 8)
- `MAX_LOGIN_ATTEMPTS` — макс. попыток до блокировки (по умолчанию 5)
- `LOCKOUT_DURATION_MINUTES` — длительность блокировки (по умолчанию 15)

### Rate limiting
- `RATE_LIMIT_API` — запросов/мин для API (по умолчанию 100)
- `RATE_LIMIT_AUTH` — запросов/мин для auth (по умолчанию 10)
- `RATE_LIMIT_MEDIA` — запросов/мин для стриминга (по умолчанию 30)

## API Endpoints

### Публичные (без авторизации)

| Метод | Путь | Описание |
|-------|------|----------|
| POST | `/api/v1/auth/register` | Регистрация |
| POST | `/api/v1/auth/login` | Вход |
| POST | `/api/v1/auth/refresh` | Обновление токена |
| POST | `/api/v1/auth/logout` | Выход |
| GET | `/api/v1/showcase/{slug}` | Витрина автора |
| GET | `/api/v1/showcase/{slug}/content` | Контент автора |
| GET | `/api/v1/showcase/{slug}/plans` | Тарифы автора |

### Защищённые (JWT)

| Метод | Путь | Описание |
|-------|------|----------|
| GET | `/api/v1/user/profile` | Профиль |
| GET | `/api/v1/user/library` | Библиотека |
| GET | `/api/v1/subscriptions` | Мои подписки |
| POST | `/api/v1/subscriptions` | Оформить подписку |
| POST | `/api/v1/payments/create` | Создать платёж |
| GET | `/api/v1/author/profile` | Профиль автора |
| POST | `/api/v1/content` | Загрузить контент |
| POST | `/api/v1/author/payouts` | Запрос выплаты |

### Стриминг

| Метод | Путь | Описание |
|-------|------|----------|
| POST | `/stream/token` | Получить медиа-токен |
| GET | `/stream/audio/{id}/master.m3u8` | HLS аудио |
| GET | `/stream/video/{id}/master.m3u8` | HLS видео |
| GET | `/stream/teaser/{id}` | Тизер |

### Админ

| Метод | Путь | Описание |
|-------|------|----------|
| GET | `/api/v1/admin/stats` | Статистика |
| GET | `/api/v1/admin/users` | Пользователи |
| POST | `/api/v1/admin/users/{id}/ban` | Бан |
| GET | `/api/v1/admin/authors` | Авторы |
| POST | `/api/v1/admin/authors/{id}/verify` | Верификация |
| GET | `/api/v1/admin/transactions` | Транзакции |
| POST | `/api/v1/admin/payouts/{id}/approve` | Одобрить выплату |

## Тестирование

```bash
# Unit tests
cargo test

# С покрытием
cargo tarpaulin

# Integration tests (требует БД)
cargo test --features integration
```

## Production Checklist

- [ ] Все секреты в env vars (не хардкод!)
- [ ] HTTPS включён
- [ ] Rate limiting настроен
- [ ] Audit logging активен
- [ ] Backup БД настроен
- [ ] Мониторинг (Prometheus + Grafana)
- [ ] Alerting на ошибки
- [ ] GDPR compliance (удаление данных)
- [ ] 152-ФЗ compliance (персональные данные)
