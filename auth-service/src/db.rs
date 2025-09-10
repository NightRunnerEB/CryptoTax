use anyhow::Result;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

/// Создаём пул соединений с Postgres через DATABASE_URL
pub async fn make_pool(database_url: &str, max_size: u32) -> Result<Pool<Postgres>> {
    let pool = PgPoolOptions::new()
        .max_connections(max_size)
        .connect(database_url)
        .await?;
    Ok(pool)
}

/// Прогон миграций из ./migrations
pub async fn run_migrations(pool: &Pool<Postgres>) -> Result<()> {
    sqlx::migrate!("././migrations").run(pool).await?;
    Ok(())
}
