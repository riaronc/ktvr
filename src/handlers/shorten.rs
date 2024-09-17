// src/handlers/shorten.rs

use actix_web::{web, HttpResponse, Responder};
use crate::models::ShortenRequest;
use crate::services::UrlService;
use crate::models::ShortenResponse;

pub async fn shorten_url(
    redis: web::Data<UrlService>,
    req: web::Json<ShortenRequest>,
    host: web::Data<String>,
) -> impl Responder {
    // Validate URL
    if let Err(e) = req.validate() {
        return HttpResponse::BadRequest().body(format!("Invalid URL: {}", e));
    }

    // Generate short URL
    match redis.shorten_url(&req.url).await {
        Ok(short_id) => {
            let short_url = format!("{}/{}", host.get_ref(), short_id);
            HttpResponse::Ok().json(ShortenResponse { short_url })
        }
        Err(e) => {
            eprintln!("Error shortening URL: {}", e);
            HttpResponse::InternalServerError().body("Failed to shorten URL")
        }
    }
}