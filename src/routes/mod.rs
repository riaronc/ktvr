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
            scope("shorten")
                .service(
                    resource("")
                        .route(post().to(shorten_url))
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

use crate::handlers::{
    redirect, health_check, shorten_url,
};
use apistos::web::{delete, get, post, put, resource, scope, Scope};

pub(crate) fn routes() -> Scope {
    scope("")
}
