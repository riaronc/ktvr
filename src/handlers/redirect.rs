// src/handlers/redirect.rs

use actix_web::{web, HttpResponse, Responder};
use crate::services::UrlService;

pub async fn redirect(
    redis: web::Data<UrlService>,
    path: web::Path<String>,
) -> impl Responder {
    let short_id = path.into_inner();

    match redis.get_original_url(&short_id).await {
        Ok(Some(original_url)) => {
            // Redirect with 301 Moved Permanently
            HttpResponse::MovedPermanently()
                .append_header(("Location", original_url))
                .finish()
        }
        Ok(None) => HttpResponse::NotFound().body("Short URL not found"),
        Err(e) => {
            eprintln!("Error retrieving original URL: {}", e);
            HttpResponse::InternalServerError().body("Internal Server Error")
        }
    }
}