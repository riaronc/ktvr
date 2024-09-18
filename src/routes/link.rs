// src/routes/link.rs

use crate::models::link::{CreateLinkRequest, CreateLinkResponse, Link};
use actix_web::error::Error as ActixError;
use actix_web::{web, HttpResponse, Responder};
use argon2::password_hash::Salt;
use argon2::{Argon2, PasswordHasher};
use chrono::{DateTime, Utc};
use log::{debug, info};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde_json::{json, to_string};
use sqlx::postgres::PgRow;
use sqlx::{Error, PgPool, Row};
use std::env;
use apistos::api_operation;
use uuid::Uuid;
/// Handler to create a new short link
///


#[api_operation(summary = "Create a short link")]
pub async fn create_link(
    pool: web::Data<PgPool>,
    payload: web::Json<CreateLinkRequest>,
) -> HttpResponse {
    // Validate the original URL
    if !is_valid_url(&payload.original_url) {
        return HttpResponse::BadRequest().body("Invalid URL");
    }

    // Generate a unique short code
    let short_code = match &payload.custom_alias {
        Some(alias) => {
            // Check if custom alias already exists
            let existing = sqlx::query!(
                "SELECT id FROM links WHERE short_code = $1",
                alias
            )
                .fetch_optional(pool.get_ref())
                .await
                .expect("Failed to execute query");

            if existing.is_some() {
                return HttpResponse::BadRequest().body("Custom alias already in use");
            }

            alias.clone()
        }
        None => generate_unique_short_code(pool.get_ref()).await.unwrap_or_else(|_| "abc123".to_string()),
    };

    // Handle password hashing if password is provided
    let password_hash = match &payload.password {
        Some(pwd) => Some(hash_password(pwd)),
        None => None,
    };

    // Insert into database
    let link_result = sqlx::query(
        r#"
        INSERT INTO links (
            original_url, short_code, expires_at, password_hash, click_limit, is_active
        ) VALUES (
            $1, $2, $3, $4, $5, $6
        )
        RETURNING id, original_url, short_code, created_at, expires_at as "expires_at!: Option<DateTime<Utc>>", password_hash, click_limit, is_active
        "#).bind(&payload.original_url).bind(&short_code)
        .bind(&payload.expires_at)
        .bind(&password_hash)
        .bind(payload.click_limit)
        .bind(true)
        .map(|row: PgRow| {
            Link {
                id: row.get(0),
                original_url: row.get(1),
                short_code: row.get(2),
                created_at: row.get(3),
                expires_at: row.get(4),
                password_hash: row.get(5),
                click_limit: row.get(6),
                is_active: row.get(7),
            }
        })
        .fetch_one(pool.get_ref())
        .await;

    match link_result {
        Ok(link) => {
            println!("Link: {:?}", link.short_code);
            let short_url = format!("https://ktvr.cc/{}", link.short_code);
            let response = CreateLinkResponse {
                short_url,
                expires_at: link.expires_at,
            };
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            log::error!("Failed to create short link: {}", e);
            HttpResponse::InternalServerError().json("Failed")
            // Err(ActixError::from(e))
        }
    }
}

/// Validates the given URL
fn is_valid_url(url: &str) -> bool {
    url::Url::parse(url).is_ok()
}

/// Generates a unique short code
async fn generate_unique_short_code(pool: &PgPool) -> Result<String, sqlx::Error> {
    loop {
        let code = generate_random_code(6);
        let exists = sqlx::query!(
            "SELECT id FROM links WHERE short_code = $1",
            code
        )
            .fetch_optional(pool)
            .await?;

        if exists.is_none() {
            return Ok(code);
        }
    }
}

/// Generates a random alphanumeric string of given length
fn generate_random_code(length: usize) -> String {
    use rand::{distributions::Alphanumeric, thread_rng, Rng};
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

/// Hashes the given password using Argon2
fn hash_password(password: &str) -> String {
    let salt: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();

    // Initialize Argon2 instance with default parameters
    let argon2 = Argon2::default();

    // Hash the password with the generated salt
    let password_hash = argon2.hash_password(password.as_bytes(), Salt::from_b64(&salt).unwrap()).unwrap().to_string();

    password_hash
}