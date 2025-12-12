use anyhow::Result;
use sqlx::{Pool, Sqlite};

pub async fn run_migrations(pool: &Pool<Sqlite>) -> Result<()> {
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}