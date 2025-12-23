use anyhow::Result;
use sqlx::SqlitePool;
use std::path::Path;

#[derive(Clone)]
pub struct Database {
    pub pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        // Create directory if it doesn't exist
        if let Some(parent) = Path::new(database_url.trim_start_matches("sqlite://")).parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let pool = SqlitePool::connect(database_url).await?;

        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Database { pool })
    }
}

pub async fn init_database() -> Result<SqlitePool> {
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://game.db".to_string());

    let db = Database::new(&database_url).await?;
    Ok(db.pool)
}
