// src/services/url_service.rs

use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use std::sync::Arc;
use tokio::sync::Mutex;
use url::Url;
use uuid::Uuid;
use crate::errors::ServiceError;

/// Service layer for URL shortening functionality.
#[derive(Clone)]
pub struct UrlService {
    // Wrapped in Arc<Mutex<>> to allow safe concurrent mutable access
    pub redis_conn: Arc<Mutex<MultiplexedConnection>>,
    key_prefix: String,
    expiration_seconds: usize,
}

impl UrlService {
    /// Creates a new UrlService instance.
    ///
    /// # Arguments
    ///
    /// * `redis_conn` - An Arc-wrapped Mutex-protected MultiplexedConnection to Redis.
    /// * `key_prefix` - A prefix for Redis keys to namespace URL mappings.
    /// * `expiration_seconds` - Time in seconds after which the short URL expires.
    pub fn new(
        redis_conn: Arc<Mutex<MultiplexedConnection>>,
        key_prefix: String,
        expiration_seconds: usize,
    ) -> Self {
        Self {
            redis_conn,
            key_prefix,
            expiration_seconds,
        }
    }

    /// Shortens a given URL by generating a unique short ID and storing the mapping in Redis.
    ///
    /// # Arguments
    ///
    /// * `original_url` - The original URL to be shortened.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` containing the short ID if successful.
    /// * `Err(ServiceError)` if an error occurs during the process.
    pub async fn shorten_url(&self, original_url: &str) -> Result<String, ServiceError> {
        // Validate the URL format
        if let Err(e) = Url::parse(original_url) {
            return Err(ServiceError::UrlParseError(e));
        }

        // Generate a unique short ID
        let short_id = self.generate_unique_id().await?;
        let key = format!("{}{}", self.key_prefix, short_id);

        // Acquire the mutex lock to get a mutable reference to the connection
        let mut conn = self.redis_conn.lock().await;
        // Store the mapping with an expiration time
        conn.set_ex(key, original_url, self.expiration_seconds)
            .await
            .map_err(ServiceError::from)?;

        Ok(short_id)
    }

    /// Retrieves the original URL associated with a given short ID from Redis.
    ///
    /// # Arguments
    ///
    /// * `short_id` - The short identifier for the URL.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(String))` containing the original URL if found.
    /// * `Ok(None)` if the short ID does not exist.
    /// * `Err(ServiceError)` if an error occurs during the retrieval.
    pub async fn get_original_url(&self, short_id: &str) -> Result<Option<String>, ServiceError> {
        let key = format!("{}{}", self.key_prefix, short_id);

        // Acquire the mutex lock to get a mutable reference to the connection
        let mut conn = self.redis_conn.lock().await;
        // Retrieve the original URL
        let original_url: Option<String> = conn.get(key).await.map_err(ServiceError::from)?;
        Ok(original_url)
    }

    /// Generates a unique short ID by ensuring it does not collide with existing IDs in Redis.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` containing a unique short ID.
    /// * `Err(ServiceError)` if an error occurs during ID generation or collision checking.
    async fn generate_unique_id(&self) -> Result<String, ServiceError> {
        loop {
            let short_id = self.generate_short_id();
            let key = format!("{}{}", self.key_prefix, short_id);

            // Acquire the mutex lock to check for existence
            let mut conn = self.redis_conn.lock().await;
            let exists: bool = conn.exists(&key).await.map_err(ServiceError::from)?;
            if !exists {
                return Ok(short_id);
            }
            // If exists, loop to generate a new ID
        }
    }

    /// Generates a random short ID using UUID and truncates it to 6 alphanumeric characters.
    ///
    /// # Returns
    ///
    /// * `String` containing the generated short ID.
    fn generate_short_id(&self) -> String {
        Uuid::new_v4()
            .simple()
            .to_string()
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .take(6)
            .collect()
    }
}