// src/handlers/health_check.rs

use actix_web::{HttpResponse, Responder};
use log::debug;

/// Handler for the `/health` endpoint.
/// Responds with a 200 OK status and body "OK".
pub async fn health_check() -> impl Responder {
    debug!("Health check: OK");
    HttpResponse::Ok().body("OK")
}