// src/main.rs

mod config;
mod models;

use crate::models::*;

mod handlers;
mod services;
mod routes;
mod utils;
mod errors;

use crate::handlers::{
    health_check,
    redirect,
    shorten_url,
};
use actix_web::middleware::Logger as ActixLogger;
use actix_web::{web, App, HttpServer};
use apistos::app::OpenApiWrapper;
use apistos::info::{Contact, Info, License};
use apistos::paths::{ExternalDocumentation, PathItem};
use apistos::server::Server;
use apistos::spec::Spec;
use apistos::tag::Tag;
use apistos::web::{get, post, resource, scope};
use apistos::OpenApi;
use apistos::SwaggerUIConfig;
use config::Config;
use dotenvy::dotenv;
use log::{error, info};
use redis::aio::MultiplexedConnection;
use services::UrlService;
use sqlx::PgPool;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
// Import PgPool from SQLx

use std::error::Error;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load environment variables from .env file
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
    info!("Connected to Redis: {}", config.redis_url);

    // Initialize UrlService with Redis connection, key prefix, and expiration time
    let url_service = UrlService::new(
        redis_conn.clone(),
        "short_url:".to_string(),
        7 * 24 * 60 * 60, // 7 days in seconds
    );

    // Establish PostgreSQL connection pool
    let pg_pool = match establish_postgres_connection().await {
        Ok(pool) => Arc::new(pool),
        Err(e) => {
            error!("Failed to connect to PostgreSQL: {}", e);
            std::process::exit(1);
        }
    };
    info!("Connected to PostgreSQL database");

    // Clone pool and other shared data for use in the closure
    let pg_pool_clone = pg_pool.clone();
    let url_service_clone = url_service.clone();
    let host_url = config.host_url.clone();

    // Start HTTP server
    info!("Starting server at {}", config.host_url);

    HttpServer::new(move || {
        // Define OpenAPI specification
        let spec = Spec {
            default_tags: vec!["api".to_owned()],
            tags: vec![
                Tag {
                    name: "api".to_string(),
                    description: Some("API Endpoints".to_string()),
                    ..Default::default()
                },
            ],
            info: Info {
                title: "KTVR Swagger - OpenAPI 3.0".to_string(),
                description: Some("A URL Shortener Service".to_string()),
                terms_of_service: Some("http://swagger.io/terms/".to_string()),
                contact: Some(Contact {
                    email: Some("riaronc@gmail.com".to_string()),
                    ..Default::default()
                }),
                license: Some(License {
                    name: "Apache 2.0".to_string(),
                    url: Some("http://www.apache.org/licenses/LICENSE-2.0.html".to_string()),
                    ..Default::default()
                }),
                version: "1.0.17".to_string(),
                ..Default::default()
            },
            external_docs: Some(ExternalDocumentation {
                description: Some("Find out more about Swagger".to_string()),
                url: "http://swagger.io".to_string(),
                ..Default::default()
            }),
            servers: vec![
                Server { url: "/api/v1".to_string(), ..Default::default() },
            ],
            ..Default::default()
        };

        App::new()
            .document(spec)
            // Add PgPool to application data
            .app_data(web::Data::new(pg_pool_clone.clone()))
            // Add UrlService to application data
            .app_data(web::Data::new(url_service_clone.clone()))
            // Add host_url to application data
            .app_data(web::Data::new(host_url.clone()))
            // Configure routes
            .configure(routes::config)
            // Integrate Apistos Swagger UI

            // Add Actix Web Logger middleware
            .wrap(ActixLogger::default())
            .build_with(
                "/openapi.json/",
                apistos::app::BuildConfig::default()
                    .with(SwaggerUIConfig::new(&"/docs/")),
            )
    })
        .bind("0.0.0.0:8080")?
        .run()
        .await?;

    Ok(())
}

/// Establishes a Redis connection using the provided Redis URL.
/// Returns a MultiplexedConnection on success.
async fn establish_redis_connection(redis_url: &str) -> redis::RedisResult<MultiplexedConnection> {
    info!("Trying to establish Redis connection with URL: {}", redis_url);
    let client = redis::Client::open(redis_url)?;
    client.get_multiplexed_tokio_connection().await
}

/// Establishes a PostgreSQL connection pool using the DATABASE_URL from the environment.
/// Returns a PgPool on success.
async fn establish_postgres_connection() -> Result<PgPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env file");
    PgPool::connect(&database_url).await
}