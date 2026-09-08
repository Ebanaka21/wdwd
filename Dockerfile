FROM rust:1.75-slim as builder

WORKDIR /app

# Установка зависимостей для сборки
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Копирование манифеста и загрузка зависимостей
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release || true
RUN rm -rf src

# Копирование исходного кода
COPY src ./src
COPY migrations ./migrations

# Сборка
RUN cargo build --release

# Финальный образ
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    ffmpeg \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/chistovik-backend /app/server
COPY --from=builder /app/migrations /app/migrations

EXPOSE 8080

CMD ["/app/server"]
