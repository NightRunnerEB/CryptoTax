use anyhow::{Context, Result};
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::time::Duration;

pub async fn make_pool(database_url: &str, max_size: u32, timeout: u64) -> Result<Pool<Postgres>> {
    let pool = PgPoolOptions::new()
        .max_connections(max_size)
        .acquire_timeout(Duration::from_secs(timeout))
        .connect(database_url)
        .await
        .with_context(|| "connect Postgres")?;
    Ok(pool)
}

pub async fn run_migrations(pool: &Pool<Postgres>) -> Result<()> {
    sqlx::migrate!("././migrations").run(pool).await?;
    Ok(())
}
