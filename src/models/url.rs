// src/models/url.rs

use serde::{Deserialize, Serialize};
use url::Url;

/// Request payload for shortening a URL.
#[derive(Deserialize)]
pub struct ShortenRequest {
    pub url: String,
}

impl ShortenRequest {
    /// Validates the URL format.
    pub fn validate(&self) -> Result<Url, url::ParseError> {
        Url::parse(&self.url)
    }
}

/// Response payload containing the shortened URL.
#[derive(Serialize)]
pub struct ShortenResponse {
    pub short_url: String,
}