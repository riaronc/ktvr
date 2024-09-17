// src/main.rs

mod config;
mod models;
mod handlers;
mod services;
mod routes;
mod utils;
mod errors;

use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use redis::aio::MultiplexedConnection;
use services::UrlService;
use config::Config;
use log::{info, error};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize environment variables from .env file
    dotenv().ok();

    // Initialize the logger
    env_logger::init();

    // Load configuration from environment variables
    let config = Config::from_env();

    // Establish Redis connection
    let redis_conn = match establish_redis_connection(&config.redis_url).await {
        Ok(conn) => Arc::new(Mutex::new(conn)),
        Err(e) => {
            error!("Failed to connect to Redis: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize UrlService with Redis connection, key prefix, and expiration time
    let url_service = UrlService::new(
        redis_conn.clone(),
        "short_url:".to_string(),
        7 * 24 * 60 * 60, // 7 days in seconds
    );

    // Start HTTP server
    info!("Starting server at {}", config.host_url);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(url_service.clone()))
            .app_data(web::Data::new(config.host_url.clone()))
            .configure(routes::config)
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}

/// Establishes a Redis connection using the provided Redis URL.
/// Returns a MultiplexedConnection on success.
async fn establish_redis_connection(redis_url: &str) -> redis::RedisResult<MultiplexedConnection> {
    let client = redis::Client::open(redis_url)?;
    client.get_multiplexed_tokio_connection().await
}