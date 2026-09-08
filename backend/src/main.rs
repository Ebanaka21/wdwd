use actix_web::{web, App, HttpServer, middleware};
use actix_cors::Cors;
use sqlx::postgres::PgPoolOptions;
use std::env;
use dotenv::dotenv;

mod config;
mod models;
mod routes;
mod middleware as app_middleware;
mod services;
mod errors;

use config::Config;

#[cfg(test)]
mod tests;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let config = Config::from_env();

    // Database connection pool
    let pool = PgPoolOptions::new()
        .max_connections(config.database_max_connections)
        .connect(&config.database_url)
        .await
        .expect("Failed to create database pool");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    log::info!("Starting server at http://{}:{}", config.host, config.port);

    // Redis connection
    let redis_client = redis::Client::open(config.redis_url.clone())
        .expect("Failed to create Redis client");

    let config_data = web::Data::new(config.clone());
    let pool_data = web::Data::new(pool.clone());
    let redis_data = web::Data::new(redis_client);

    // FIX: Graceful shutdown через tokio signal
    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin(&config.cors_origin)
            .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE"])
            .allowed_headers(vec!["Content-Type", "Authorization"])
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            // FIX: Security headers middleware
            .wrap(app_middleware::security::SecurityHeaders)
            .app_data(config_data.clone())
            .app_data(pool_data.clone())
            .app_data(redis_data.clone())
            // Public routes
            .configure(routes::auth::configure)
            .configure(routes::showcase::configure)
            // Protected routes
            .service(
                web::scope("/api/v1")
                    .wrap(app_middleware::auth::AuthMiddleware)
                    .configure(routes::author::configure)
                    .configure(routes::content::configure)
                    .configure(routes::subscription::configure)
                    .configure(routes::payment::configure)
                    .configure(routes::user::configure)
                    .configure(routes::admin::configure)
            )
            // Media streaming routes (separate auth via tokens)
            .configure(routes::streaming::configure)
    })
    .bind(format!("{}:{}", config.host, config.port))?
    .run();

    // FIX: Graceful shutdown
    log::info!("Server started. Press Ctrl+C to stop.");
    
    tokio::select! {
        result = server => {
            if let Err(e) = result {
                log::error!("Server error: {}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            log::info!("Received Ctrl+C, shutting down gracefully...");
        }
    }

    log::info!("Server stopped.");
    Ok(())
}
