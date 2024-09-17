// src/handlers/mod.rs

pub mod shorten;
pub mod redirect;
pub mod health_check;

pub use shorten::shorten_url;
pub use redirect::redirect;
pub use health_check::health_check;