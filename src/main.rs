// src/main.rs

mod config;
mod models;
mod handlers;
mod services;
mod routes;
mod utils;
mod errors;
use crate::handlers::{
    shorten_url,
    redirect,
    health_check,
};
use actix_web::{web, App, HttpServer};
use actix_web::middleware::Logger as ActixLogger;
use dotenvy::dotenv;
use dotenvy;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use redis::aio::MultiplexedConnection;
use services::UrlService;
use config::Config;
use log::{info, error};
use apistos::app::OpenApiWrapper;
use apistos::server::Server;
use apistos::spec::Spec;
use apistos::web::{get, post, resource, scope};
use env_logger::Logger;
use crate::models::ShortenResponse;
use apistos::info::{Contact, Info, License};
use apistos::paths::ExternalDocumentation;

use apistos::SwaggerUIConfig;
use apistos::tag::Tag;
async fn load_env() -> () {
    dotenv().ok();

    // Determine the current environment
    let app_env = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

    // Load the appropriate .env file based on APP_ENV
    match app_env.as_str() {
        "prod" => {
            dotenvy::from_filename(".env.prod").ok();
        }
        "dev" => {
            dotenvy::from_filename(".env.dev").ok();
        }
        _ => {
            dotenvy::from_filename(".env").ok();
        }
    }

    // Initialize the logger
    env_logger::init();
}
use std::error::Error;

#[actix_web::main]
async fn main() -> Result<(), impl Error> {
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
        let spec = Spec {
            default_tags: vec!["api".to_owned()],
            tags: vec![
                Tag {
                    name: "api".to_string(),
                    description: Some("Everything about petstore".to_string()),
                    ..Default::default()
                },
                Tag {
                    name: "pet".to_string(),
                    description: Some("Everything about your Pets".to_string()),
                    ..Default::default()
                },
                Tag {
                    name: "store".to_string(),
                    description: Some("Access to Petstore orders".to_string()),
                    ..Default::default()
                },
                Tag {
                    name: "user".to_string(),
                    description: Some("Operations about user".to_string()),
                    ..Default::default()
                },
            ],
            info: Info {
                title: "Swagger Petstore - OpenAPI 3.0".to_string(),
                description: Some("This is a sample Pet Store Server based on the OpenAPI 3.0 specification.  You can find out more about\nSwagger at [http://swagger.io](http://swagger.io). In the third iteration of the pet store, we've switched to the design first approach!\nYou can now help us improve the API whether it's by making changes to the definition itself or to the code.\nThat way, with time, we can improve the API in general, and expose some of the new features in OAS3.\n\nSome useful links:\n- [The Pet Store repository](https://github.com/swagger-api/swagger-petstore)\n- [The source API definition for the Pet Store](https://github.com/swagger-api/swagger-petstore/blob/master/src/main/resources/openapi.yaml)".to_string()),
                terms_of_service: Some("http://swagger.io/terms/".to_string()),
                contact: Some(Contact {
                    email: Some("apiteam@swagger.io".to_string()),
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
            servers: vec![Server { url: "/api/v3".to_string(), ..Default::default() }],
            ..Default::default()
        };

        App::new()
            .document(spec)
            .app_data(web::Data::new(url_service.clone()))
            .app_data(web::Data::new(config.host_url.clone()))
            // .configure(routes::config)
            .build("/openapi.json")
            .service(
                routes::routes()
            )

            // .wrap(ActixLogger::default())

    })
        .bind("0.0.0.0:8080")?
        .run()
        .await
}

/// Establishes a Redis connection using the provided Redis URL.
/// Returns a MultiplexedConnection on success.
async fn establish_redis_connection(redis_url: &str) -> redis::RedisResult<MultiplexedConnection> {
    info!("Trying establish Redis by URL: {}",  redis_url);
    let client = redis::Client::open(redis_url)?;
    client.get_multiplexed_tokio_connection().await
}