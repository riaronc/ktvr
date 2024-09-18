use sqlx::{Pool, Postgres};
use std::env;

/// Type alias for the PostgreSQL connection pool
pub type DbPool = Pool<Postgres>;

/// Establishes a connection pool to the PostgreSQL database
pub async fn establish_connection() -> Result<DbPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env file");

    Pool::<Postgres>::connect(&database_url).await
}