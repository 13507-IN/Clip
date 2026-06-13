use sqlx::{PgPool, Error as SqlxError};

use crate::model::UrlEntry;

pub struct Store {
    pool: PgPool,
}

impl Store {
    pub async fn new(database_url: &str) -> Result<Self, SqlxError> {
        let pool = PgPool::connect(database_url).await?;
        
        let store = Self { pool };
        store.migrate().await?;
        Ok(store)
    }

    async fn migrate(&self) -> Result<(), SqlxError> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS urls (
                id BIGSERIAL PRIMARY KEY,
                short_code TEXT NOT NULL UNIQUE,
                original TEXT NOT NULL,
                created_at TEXT NOT NULL,
                clicks BIGINT NOT NULL DEFAULT 0
            );"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_short_code ON urls(short_code);")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE UNIQUE INDEX IF NOT EXISTS idx_original ON urls(original);")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn insert(&self, short_code: &str, original: &str) -> Result<UrlEntry, SqlxError> {
        let now = chrono::Utc::now().to_rfc3339();
        
        let row = sqlx::query_as::<_, UrlEntry>(
            "INSERT INTO urls (short_code, original, created_at) 
             VALUES ($1, $2, $3)
             RETURNING id, short_code, original, created_at, clicks"
        )
        .bind(short_code)
        .bind(original)
        .bind(&now)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn get_by_short_code(&self, code: &str) -> Result<Option<UrlEntry>, SqlxError> {
        let row = sqlx::query_as::<_, UrlEntry>(
            "SELECT id, short_code, original, created_at, clicks
             FROM urls WHERE short_code = $1"
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn get_by_original(&self, original: &str) -> Result<Option<UrlEntry>, SqlxError> {
        let row = sqlx::query_as::<_, UrlEntry>(
            "SELECT id, short_code, original, created_at, clicks
             FROM urls WHERE original = $1"
        )
        .bind(original)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn increment_clicks(&self, code: &str) -> Result<(), SqlxError> {
        sqlx::query("UPDATE urls SET clicks = clicks + 1 WHERE short_code = $1")
        .bind(code)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
