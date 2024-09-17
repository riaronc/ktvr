// src/handlers/health_check.rs

use actix_web::{HttpResponse, Responder};

/// Handler for the `/health` endpoint.
/// Responds with a 200 OK status and body "OK".
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("OK")
}