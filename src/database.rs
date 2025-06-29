use sqlx::{PgPool, postgres::PgPoolOptions, Postgres, Transaction};
use std::time::Duration;

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(32) // Increased max connections for higher throughput
        .min_connections(8)  // Higher minimum to reduce connection overhead
        .acquire_timeout(Duration::from_secs(8)) // Shorter timeout for faster failure
        .idle_timeout(Duration::from_secs(300))   // Shorter idle timeout to free resources
        .max_lifetime(Duration::from_secs(1800))  // Keep connection lifetime
        .test_before_acquire(true) // Ensure connection health
        .connect(database_url)
        .await
}

pub async fn test_connection(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1")
        .fetch_one(pool)
        .await?;
    Ok(())
}

// Optimized query helpers for common operations
pub async fn get_user_by_id_optimized(pool: &PgPool, user_id: i32) -> Result<Option<sqlx::postgres::PgRow>, sqlx::Error> {
    sqlx::query(
        "SELECT id, username, email, first_name, last_name, phone, role, professional_role, company_name, is_active, email_verified, created_at 
         FROM users 
         WHERE id = $1 AND is_active = true"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn get_user_by_email_or_username_optimized(pool: &PgPool, identifier: &str) -> Result<Option<sqlx::postgres::PgRow>, sqlx::Error> {
    sqlx::query(
        "SELECT id, username, email, password_hash, first_name, last_name, phone, role, professional_role, company_name, is_active, email_verified, created_at, updated_at
         FROM users 
         WHERE (username = $1 OR email = $1) AND is_active = true
         LIMIT 1"
    )
    .bind(identifier)
    .fetch_optional(pool)
    .await
}

// Batch operations for better performance
pub async fn get_multiple_users(pool: &PgPool, user_ids: &[i32]) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
    sqlx::query(
        "SELECT id, username, email, first_name, last_name, phone, role, professional_role, company_name, is_active, email_verified, created_at 
         FROM users 
         WHERE id = ANY($1) AND is_active = true
         ORDER BY id"
    )
    .bind(user_ids)
    .fetch_all(pool)
    .await
}

// Transaction helper for atomic operations
#[allow(dead_code)]
pub async fn begin_transaction(pool: &PgPool) -> Result<Transaction<'_, Postgres>, sqlx::Error> {
    pool.begin().await
} 