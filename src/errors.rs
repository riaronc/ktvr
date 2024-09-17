// src/errors.rs

use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;

/// Struct for error responses.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub message: String,
}

/// Enum representing different service errors.
#[derive(Debug)]
pub enum ServiceError {
    RedisError(redis::RedisError),
    UrlParseError(url::ParseError),
    // Add other error variants as needed
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceError::RedisError(e) => write!(f, "Redis error: {}", e),
            ServiceError::UrlParseError(e) => write!(f, "URL parse error: {}", e),
        }
    }
}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        let error_response = ErrorResponse {
            message: self.to_string(),
        };
        match self {
            ServiceError::RedisError(_) => HttpResponse::InternalServerError().json(error_response),
            ServiceError::UrlParseError(_) => HttpResponse::BadRequest().json(error_response),
        }
    }
}

impl From<redis::RedisError> for ServiceError {
    fn from(error: redis::RedisError) -> Self {
        ServiceError::RedisError(error)
    }
}

impl From<url::ParseError> for ServiceError {
    fn from(error: url::ParseError) -> Self {
        ServiceError::UrlParseError(error)
    }
}
