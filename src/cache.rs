use anyhow::Result;
use chrono::Utc;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::path::Path;

#[async_trait::async_trait]
pub trait CacheBackend: Send + Sync {
    async fn get(&self, url: &str, ttl_secs: i64) -> Result<Option<String>>;
    async fn insert(&self, url: &str, body: &str) -> Result<()>;
}

#[derive(Clone)]
pub struct Cache {
    pool: SqlitePool,
}

#[async_trait::async_trait]
impl CacheBackend for Cache {
    async fn get(&self, url: &str, ttl_secs: i64) -> Result<Option<String>> {
        self.get_internal(url, ttl_secs).await
    }

    async fn insert(&self, url: &str, body: &str) -> Result<()> {
        self.insert_internal(url, body).await
    }
}

impl Cache {
    pub async fn new(db_path: &str) -> Result<Self> {
        // Ensure the directory exists
        if let Some(parent) = Path::new(db_path).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous};
        let options = SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        let cache = Self { pool };
        cache.init().await?;
        Ok(cache)
    }

    async fn init(&self) -> Result<()> {
        sqlx::query!(
            "CREATE TABLE IF NOT EXISTS api_cache (
                url TEXT PRIMARY KEY,
                response_body TEXT NOT NULL,
                cached_at_timestamp INTEGER NOT NULL
            );"
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_internal(&self, url: &str, ttl_secs: i64) -> Result<Option<String>> {
        let row = sqlx::query!(
            "SELECT response_body, cached_at_timestamp FROM api_cache WHERE url = ?",
            url
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(r) = row {
            let now = Utc::now().timestamp();
            if now - r.cached_at_timestamp < ttl_secs {
                return Ok(Some(r.response_body));
            } else {
                // Expired: delete it
                sqlx::query!("DELETE FROM api_cache WHERE url = ?", url)
                    .execute(&self.pool)
                    .await?;
            }
        }
        Ok(None)
    }

    pub async fn insert_internal(&self, url: &str, body: &str) -> Result<()> {
        let now = Utc::now().timestamp();
        sqlx::query!(
            "INSERT OR REPLACE INTO api_cache (url, response_body, cached_at_timestamp)
             VALUES (?, ?, ?)",
            url,
            body,
            now
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

pub async fn check_cache_hits(
    cache: std::sync::Arc<dyn CacheBackend>,
    urls: &[String],
    ttl_secs: i64,
) -> Result<(usize, usize)> {
    let mut hits = 0;
    let total = urls.len();
    if total == 0 {
        return Ok((0, 0));
    }
    for url in urls {
        if cache.get(url, ttl_secs).await.ok().flatten().is_some() {
            hits += 1;
        }
    }
    Ok((hits, total))
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

        // Test expiration: ttl_secs=0 means any cached entry is immediately expired.
        // This avoids a real sleep and keeps the test deterministic and fast.
        let val_expired = cache.get("http://test.com/api", 0).await.unwrap();
        assert_eq!(val_expired, None);

        // Cleanup
        let _ = std::fs::remove_file(db_path);
    }

    #[tokio::test]
    async fn test_check_cache_hits() {
        let db_path = "tests/test_check_cache_hits.db";
        let _ = std::fs::remove_file(db_path);

        let cache = std::sync::Arc::new(Cache::new(db_path).await.unwrap());
        cache.insert("http://test.com/1", "body1").await.unwrap();
        cache.insert("http://test.com/2", "body2").await.unwrap();

        let urls = vec![
            "http://test.com/1".to_string(),
            "http://test.com/2".to_string(),
            "http://test.com/3".to_string(),
        ];

        let (hits, total) = check_cache_hits(cache.clone(), &urls, 10).await.unwrap();
        assert_eq!(hits, 2);
        assert_eq!(total, 3);

        let _ = std::fs::remove_file(db_path);
    }
}
