// src/handlers/health_check.rs

use actix_web::{HttpResponse, Responder};
use log::debug;
use apistos::{api_operation, ApiComponent};
/// Handler for the `/health` endpoint.
/// Responds with a 200 OK status and body "OK".

#[api_operation(summary = "Get an element from the todo list")]
pub async fn health_check() -> impl Responder {
    debug!("Health check: OK");
    HttpResponse::Ok().body("OK\n")
}