use crate::cache::Cache;
use anyhow::{anyhow, Result};
use reqwest::Client;

pub struct YahooClient {
    client: Client,
    cache: Cache,
    base_url: String,
    ttl_secs: i64,
}

impl YahooClient {
    pub fn new(cache: Cache) -> Result<Self> {
        Ok(Self {
            client: Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
                .build()?,
            cache,
            base_url: "https://query2.finance.yahoo.com/v8/finance/chart".to_string(),
            ttl_secs: 300, // 5 minutes cache default
        })
    }

    pub fn with_ttl(mut self, ttl_secs: i64) -> Self {
        self.ttl_secs = ttl_secs;
        self
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    pub async fn fetch_ticker_chart(
        &self,
        ticker: &str,
        from_timestamp: i64,
        to_timestamp: i64,
    ) -> Result<String> {
        let url = format!(
            "{}/{}?period1={}&period2={}&interval=1d",
            self.base_url, ticker, from_timestamp, to_timestamp
        );

        if let Some(cached) = self.cache.get(&url, self.ttl_secs).await? {
            return Ok(cached);
        }

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Yahoo Finance API returned error status: {} for ticker {}",
                response.status(),
                ticker
            ));
        }

        let body = response.text().await?;
        self.cache.insert(&url, &body).await?;

        Ok(body)
    }

    pub async fn ping(&self) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        self.fetch_ticker_chart("^GSPC", now - 86400, now).await?;
        Ok(())
    }
}
