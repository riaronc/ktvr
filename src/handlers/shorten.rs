// src/handlers/shorten.rs

use actix_web::{web, HttpResponse, Responder, Error};
use actix_web::web::Json;
use crate::models::ShortenRequest;
use crate::services::UrlService;
use crate::models::ShortenResponse;
use crate::errors::ServiceError;
use log::{info, debug, error};
use apistos::{api_operation, ApiComponent};


#[api_operation(summary = "Shorten a URL")]
pub async fn shorten_url(
    url_service: web::Data<UrlService>,
    req: Json<ShortenRequest>,
    host: web::Data<String>,
) -> Result<Json<ShortenResponse>, Error> {
    let original_url = req.url.trim();
    info!("Received request to shorten URL: {}", original_url);

    // Shorten the URL
    let short_id = match url_service.shorten_url(original_url).await {
        Ok(id) => {
            debug!("Generated short ID: {}", id);
            id
        },
        Err(e) => {
            error!("{}", e);
            return Err(Error::from(e))
        }
    };
    let short_url = format!("{}/{}", host.get_ref(), short_id);
    info!("Short URL created: {}", short_url);

    Ok(Json(ShortenResponse { short_url }))
}
