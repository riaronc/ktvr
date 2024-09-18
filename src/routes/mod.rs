mod link;

use crate::handlers::{
    health_check, redirect,
};
use apistos::web::{get, post, resource, scope, Scope};

// src/routes/mod.rs
//
// use actix_web::web;
// use crate::handlers::{
//     shorten_url,
//     redirect,
//     health_check,
// };
use apistos::web::ServiceConfig;
// use apistos::web::{resource, get, scope, post};
//
pub fn config(cfg: &mut ServiceConfig) {
    cfg
        .service(
            scope("/api")
                .service(
                    resource("/shorten")
                        .route(post().to(link::create_link))
                )
        )
        .service(
            resource("health_check").route(get().to(health_check))
        )


        .service(
            resource("{short_id}")
                .route(get().to(redirect))
        );
}
