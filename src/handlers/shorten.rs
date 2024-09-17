// src/handlers/shorten.rs

use actix_web::{web, HttpResponse, Responder};
use crate::models::ShortenRequest;
use crate::services::UrlService;
use crate::models::ShortenResponse;
use crate::errors::ServiceError;
use log::{info, debug, error};

pub async fn shorten_url(
    url_service: web::Data<UrlService>,
    req: web::Json<ShortenRequest>,
    host: web::Data<String>,
) -> Result<impl Responder, ServiceError> {
    let original_url = req.url.trim();
    info!("Received request to shorten URL: {}", original_url);

    // Shorten the URL
    let short_id = match url_service.shorten_url(original_url).await {
        Ok(id) => {
            debug!("Generated short ID: {}", id);
            id
        },
        Err(e) => {
            error!("Error shortening URL: {}", e);
            return Err(e);
        }
    };
    let short_url = format!("{}/{}", host.get_ref(), short_id);
    info!("Short URL created: {}", short_url);

    Ok(HttpResponse::Ok().json(ShortenResponse { short_url }))
}
