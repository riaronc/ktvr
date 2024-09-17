// tests/integration_tests.rs

use actix_web::{test, App};
use url_shortener::handlers::{shorten_url, redirect};
use url_shortener::services::UrlService;
use std::sync::Arc;
use redis::Client;
use redis::aio::ConnectionManager;
use serde_json::json;

#[actix_rt::test]
async fn test_shorten_and_redirect() {
    // Setup Redis connection
    let client = Client::open("redis://127.0.0.1/").unwrap();
    let conn_manager = ConnectionManager::new(client).await.unwrap();
    let redis_pool = Arc::new(conn_manager);
    let url_service = UrlService::new(redis_pool.clone(), "short_url:".to_string(), 7 * 24 * 60 * 60);

    // Setup Actix app
    let app = test::init_service(
        App::new()
            .data(url_service)
            .data("http://localhost:8080".to_string())
            .service(
                actix_web::web::resource("/shorten")
                    .route(actix_web::web::post().to(shorten_url))
            )
            .service(
                actix_web::web::resource("/{short_id}")
                    .route(actix_web::web::get().to(redirect))
            )
    ).await;

    // Shorten a URL
    let req = test::TestRequest::post()
        .uri("/shorten")
        .set_json(&json!({"url": "https://www.example.com"}))
        .to_request();
    let resp: serde_json::Value = test::read_response_json(&app, req).await;
    assert!(resp.get("short_url").is_some());

    let short_url = resp.get("short_url").unwrap().as_str().unwrap();
    let short_id = short_url.split('/').last().unwrap();

    // Test redirection
    let req = test::TestRequest::get()
        .uri(&format!("/{}", short_id))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::MOVED_PERMANENTLY);
}