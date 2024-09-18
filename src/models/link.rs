use apistos::ApiComponent;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
/// Represents a shortened URL
#[derive(Debug, Serialize, Deserialize, FromRow, JsonSchema, ApiComponent)]
pub struct Link {
    pub id: Uuid,
    pub original_url: String,
    pub short_code: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub password_hash: Option<String>,
    pub click_limit: Option<i32>,
    pub is_active: bool,
}

/// Payload for creating a new short link
#[derive(Debug, Deserialize, JsonSchema, ApiComponent)]
pub struct CreateLinkRequest {
    pub original_url: String,
    pub custom_alias: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub password: Option<String>,
    pub click_limit: Option<i32>,
}

/// Response after creating a new short link
#[derive(Debug, Serialize, JsonSchema, ApiComponent)]
pub struct CreateLinkResponse {
    pub short_url: String,
    pub expires_at: Option<DateTime<Utc>>,
}