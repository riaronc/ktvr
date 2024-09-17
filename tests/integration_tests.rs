// tests/integration_tests.rs

use actix_web::{test, App};
use serde_json::json;

use url_shortener::handlers::{health_check, shorten, redirect};
use url_shortener::services::UrlService;
use url_shortener::config::Config;
use actix_web::web;
use std::sync::Arc;
use tokio::sync::Mutex;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;

// Ensure your main application modules are accessible
// Update `Cargo.toml` with `url_shortener` as a path dependency if necessary

#[actix_rt::test]
async fn test_health_check() {
    // Initialize environment variables
    dotenv::dotenv().ok();

    // Load configuration
    let config = Config::from_env();

    // Establish Redis connection
    let redis_conn = match establish_redis_connection(&config.redis_url).await {
        Ok(conn) => Arc::new(Mutex::new(conn)),
        Err(_) => panic!("Failed to connect to Redis for testing"),
    };

    // Initialize UrlService
    let url_service = UrlService::new(
        redis_conn.clone(),
        "test_short_url:".to_string(),
        60, // 1 minute for testing
    );

    // Initialize Actix app with health check handler
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(url_service))
            .route("/health", web::get().to(health::health_check))
    ).await;

    // Create a request to the /health endpoint
    let req = test::TestRequest::get()
        .uri("/health")
        .to_request();

    // Send the request and get the response
    let resp = test::call_service(&app, req).await;

    // Assert that the status is 200 OK
    assert!(resp.status().is_success());

    // Extract the response body
    let body = test::read_body(resp).await;

    // Assert that the body is "OK"
    assert_eq!(body, "OK");
}

#[actix_rt::test]
async fn test_shorten_url() {
    // Initialize environment variables
    dotenv::dotenv().ok();

    // Load configuration
    let config = Config::from_env();

    // Establish Redis connection
    let redis_conn = match establish_redis_connection(&config.redis_url).await {
        Ok(conn) => Arc::new(Mutex::new(conn)),
        Err(_) => panic!("Failed to connect to Redis for testing"),
    };

    // Initialize UrlService
    let url_service = UrlService::new(
        redis_conn.clone(),
        "test_short_url:".to_string(),
        60, // 1 minute for testing
    );

    // Initialize Actix app with shorten_url handler
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(url_service))
            .app_data(web::Data::new(config.host_url.clone()))
            .route("/shorten", web::post().to(shorten::shorten_url))
    ).await;

    // Define the original URL to shorten
    let original_url = "https://www.rust-lang.org";

    // Create a POST request to the /shorten endpoint with JSON body
    let req = test::TestRequest::post()
        .uri("/shorten")
        .set_json(&json!({ "url": original_url }))
        .to_request();

    // Send the request and get the response
    let resp: serde_json::Value = test::read_response_json(&app, req).await;

    // Assert that the response contains the "short_url" field
    assert!(resp.get("short_url").is_some());

    // Extract the short URL
    let short_url = resp.get("short_url").unwrap().as_str().unwrap();

    // Optionally, verify the format of the short URL
    assert!(short_url.starts_with(&config.host_url));

    // Optionally, extract the short ID
    let short_id = short_url.trim_start_matches(&format!("{}/", config.host_url)).to_string();

    // Verify that the short ID exists in Redis
    let mut conn = redis_conn.lock().await;
    let original_url_in_redis: Option<String> = conn.get(format!("test_short_url:{}", short_id)).await.unwrap();
    assert_eq!(original_url_in_redis, Some(original_url.to_string()));
}

#[actix_rt::test]
async fn test_redirect() {
    // Initialize environment variables
    dotenv::dotenv().ok();

    // Load configuration
    let config = Config::from_env();

    // Establish Redis connection
    let redis_conn = match establish_redis_connection(&config.redis_url).await {
        Ok(conn) => Arc::new(Mutex::new(conn)),
        Err(_) => panic!("Failed to connect to Redis for testing"),
    };

    // Initialize UrlService
    let url_service = UrlService::new(
        redis_conn.clone(),
        "test_short_url:".to_string(),
        60, // 1 minute for testing
    );

    // Manually insert a short URL mapping into Redis for testing
    let short_id = "test123";
    let original_url = "https://www.rust-lang.org";
    {
        let mut conn = redis_conn.lock().await;
        conn.set_ex(format!("test_short_url:{}", short_id), original_url, 60).await.unwrap();
    }

    // Initialize Actix app with redirect handler
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(url_service))
            .route("/{short_id}", web::get().to(redirect::redirect))
    ).await;

    // Create a GET request to the /{short_id} endpoint
    let req = test::TestRequest::get()
        .uri(&format!("/{}", short_id))
        .to_request();

    // Send the request and get the response
    let resp = test::call_service(&app, req).await;

    // Assert that the status is 301 Moved Permanently
    assert_eq!(resp.status(), 301);

    // Assert that the Location header is set to the original URL
    let headers = resp.headers();
    assert_eq!(headers.get("Location").unwrap(), original_url);
}

/// Helper function to establish a Redis connection for tests
async fn establish_redis_connection(redis_url: &str) -> redis::RedisResult<MultiplexedConnection> {
    let client = redis::Client::open(redis_url)?;
    client.get_multiplexed_tokio_connection().await
}
