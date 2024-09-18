// src/models/url.rs

use serde::{Deserialize, Serialize};
use url::Url;
use schemars::JsonSchema;
use apistos::ApiComponent;
/// Request payload for shortening a URL.
#[derive(Serialize, JsonSchema, ApiComponent, Deserialize, Debug, Clone)]
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
#[derive(Serialize, JsonSchema, ApiComponent, Deserialize, Debug, Clone)]
pub struct ShortenResponse {
    pub short_url: String,
}