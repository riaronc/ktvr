// src/handlers/redirect.rs

use actix_web::{web, HttpResponse, Responder, Error as ActixError};
use crate::services::UrlService;
use crate::errors::ServiceError;
use log::{info, debug, error};
use apistos::{api_operation, ApiComponent};

#[api_operation(summary = "Redirect a given URL")]
pub async fn redirect(
    url_service: web::Data<UrlService>,
    path: web::Path<String>,
) -> Result<HttpResponse, ActixError> {
    let short_id = path.into_inner();
    info!("Received redirect request for short ID: {}", short_id);
    debug!("Looking up original URL for short ID: {}", short_id);

    match url_service.get_original_url(&short_id).await {
        Ok(Some(original_url)) => {
            info!("Redirecting to original URL: {}", original_url);
            Ok(HttpResponse::MovedPermanently()
                .append_header(("Location", original_url))
                .finish())
        },
        Ok(None) => {
            info!("Short ID not found: {}", short_id);
            Ok(HttpResponse::NotFound().body("Short URL not found"))
        },
        Err(e) => {
            error!("Error retrieving original URL for {}: {}", short_id, e);
            Err(ActixError::from(e))
        }
    }
}
