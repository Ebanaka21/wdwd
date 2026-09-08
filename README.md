# Чистовик — Backend API

Backend для платформы закрытого контента «Чистовик».

## Технологический стек

- **Язык:** Rust
- **Web-фреймворк:** Actix-web 4
- **База данных:** PostgreSQL 15+ (через SQLx)
- **Кэш / Rate limiting:** Redis
- **Хранилище файлов:** S3-совместимое (Yandex Cloud / Selectel)
- **Платежи:** ЮKassa
- **Аутентификация:** JWT
- **Медиа:** FFmpeg (HLS, транскодирование)

## Структура проекта

```
backend/
├── src/
│   ├── main.rs              # Точка входа, настройка сервера
│   ├── config.rs             # Конфигурация из env
│   ├── errors.rs             # Обработка ошибок
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
│   │   ├── payment.rs        # Платежи, вебхуки
│   │   ├── user.rs           # ЛК пользователя
│   │   ├── admin.rs          # Админ-панель
│   │   └── streaming.rs      # HLS-стриминг
│   ├── middleware/            # Middleware
│   │   ├── auth.rs           # JWT-аутентификация
│   │   └── rate_limit.rs     # Rate limiting
│   └── services/             # Бизнес-логика
│       ├── auth_service.rs   # JWT, сессии
│       ├── content_service.rs # FFmpeg, транскодирование
│       ├── payment_service.rs # ЮKassa, рекурренты
│       └── streaming_service.rs # HLS, токены, водяные знаки
├── migrations/
│   └── 001_init.sql          # Схема БД
├── Cargo.toml
├── .env.example
└── README.md
```

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

## Запуск

```bash
# 1. Установить зависимости
cargo build

# 2. Настроить .env
cp .env.example .env
# Заполнить .env

# 3. Запустить PostgreSQL и Redis
docker-compose up -d postgres redis

# 4. Запустить сервер
cargo run
```

## Защита контента

1. **HLS-стриминг** — видео/аудио разбивается на зашифрованные сегменты
2. **Токены доступа** — временные JWT для каждого сегмента (2 часа)
3. **Водяные знаки** — идентификатор пользователя на видео
4. **Rate limiting** — ограничение частоты запросов
5. **Ограничение сессий** — максимум 2 устройства
6. **Защита от хотлинкинга** — проверка Referer
7. **Тизеры** — отдельные файлы, не содержащие полный контент
