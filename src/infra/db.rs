use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;
use std::time::Duration;

pub async fn init_db_pool() -> PgPool {
    let db_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set")
        .trim()
        .to_string();

    // Concurrent queries per instance. Keep low on serverless so that
    // (instances * DB_MAX_CONNECTIONS) stays under the pooler's ceiling.
    let max_connections = env::var("DB_MAX_CONNECTIONS")
        .ok()
        .and_then(|v| v.trim().parse::<u32>().ok())
        .unwrap_or(3);

    PgPoolOptions::new()
        .max_connections(max_connections)
        .min_connections(0)
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(300))
        .max_lifetime(Duration::from_secs(1800))
        .connect_lazy(&db_url)
        .expect("Failed to connect to PostgreSQL via pooler")
}