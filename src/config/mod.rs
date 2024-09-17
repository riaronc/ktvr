// src/config/mod.rs

use dotenv::dotenv;
use std::env;

#[derive(Debug)]
pub struct Config {
    pub redis_url: String,
    pub host_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().ok(); // Load .env file

        Self {
            redis_url: env::var("REDIS_URL").expect("REDIS_URL must be set"),
            host_url: env::var("HOST_URL").unwrap_or_else(|_| "http://localhost:8080".to_string()),
        }
    }
}