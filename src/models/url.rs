// src/models/url.rs

use serde::{Deserialize, Serialize};
use url::Url;
use schemars::JsonSchema;
use apistos::ApiComponent;
/// Request payload for shortening a URL.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, ApiComponent)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
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
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, ApiComponent)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShortenResponse {
    pub short_url: String,
}