// src/routes/mod.rs

use actix_web::web;
use crate::handlers::{
    shorten_url,
    redirect,
    health_check,
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/shorten")
            .route(web::post().to(shorten_url))
    )
        .service(
            web::resource("/health_check")
                .route(web::get().to(health_check))
        )
        .service(
            web::resource("/{short_id}")
                .route(web::get().to(redirect))
        );

}
