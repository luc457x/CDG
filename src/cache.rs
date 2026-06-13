use anyhow::Result;
use chrono::Utc;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::path::Path;

#[derive(Clone)]
pub struct Cache {
    pool: SqlitePool,
}

impl Cache {
    pub async fn new(db_path: &str) -> Result<Self> {
        // Ensure the directory exists
        if let Some(parent) = Path::new(db_path).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        use sqlx::sqlite::SqliteConnectOptions;
        let options = SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        let cache = Self { pool };
        cache.init().await?;
        Ok(cache)
    }

    async fn init(&self) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS api_cache (
                url TEXT PRIMARY KEY,
                response_body TEXT NOT NULL,
                cached_at_timestamp INTEGER NOT NULL
            );",
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get(&self, url: &str, ttl_secs: i64) -> Result<Option<String>> {
        let row: Option<(String, i64)> = sqlx::query_as(
            "SELECT response_body, cached_at_timestamp FROM api_cache WHERE url = ?",
        )
        .bind(url)
        .fetch_optional(&self.pool)
        .await?;

        if let Some((body, timestamp)) = row {
            let now = Utc::now().timestamp();
            if now - timestamp < ttl_secs {
                return Ok(Some(body));
            } else {
                // Expired: delete it
                sqlx::query("DELETE FROM api_cache WHERE url = ?")
                    .bind(url)
                    .execute(&self.pool)
                    .await?;
            }
        }
        Ok(None)
    }

    pub async fn insert(&self, url: &str, body: &str) -> Result<()> {
        let now = Utc::now().timestamp();
        sqlx::query(
            "INSERT OR REPLACE INTO api_cache (url, response_body, cached_at_timestamp)
             VALUES (?, ?, ?)",
        )
        .bind(url)
        .bind(body)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_insert_get_expire() {
        let db_path = "tests/test_cache_sqlite.db";
        let _ = std::fs::remove_file(db_path);

        let cache = Cache::new(db_path).await.unwrap();

        // Test insert and immediate get
        cache
            .insert("http://test.com/api", "{\"status\":\"ok\"}")
            .await
            .unwrap();
        let val = cache.get("http://test.com/api", 10).await.unwrap();
        assert_eq!(val, Some("{\"status\":\"ok\"}".to_string()));

        // Test expiration
        tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
        let val_expired = cache.get("http://test.com/api", 1).await.unwrap();
        assert_eq!(val_expired, None);

        // Cleanup
        let _ = std::fs::remove_file(db_path);
    }
}
