use crate::cache::CacheBackend;
use anyhow::{anyhow, Result};
use reqwest::Client;
use std::sync::Arc;

pub struct YahooClient {
    client: Client,
    cache: Arc<dyn CacheBackend>,
    base_url: String,
    ttl_secs: i64,
    pb: Option<indicatif::ProgressBar>,
}

impl YahooClient {
    pub fn new(cache: Arc<dyn CacheBackend>) -> Result<Self> {
        Ok(Self {
            client: Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
                .build()?,
            cache,
            base_url: "https://query2.finance.yahoo.com/v8/finance/chart".to_string(),
            ttl_secs: 300, // 5 minutes cache default
            pb: None,
        })
    }

    pub fn with_ttl(mut self, ttl_secs: i64) -> Self {
        self.ttl_secs = ttl_secs;
        self
    }

    pub fn with_progress_bar(mut self, pb: indicatif::ProgressBar) -> Self {
        self.pb = Some(pb);
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

        let mut attempts = 0;
        let max_attempts = 4;
        let mut retry_delay = std::time::Duration::from_millis(2000);

        loop {
            let response = self.client.get(&url).send().await;

            match response {
                Ok(resp) => {
                    let status = resp.status();

                    if status == 429 || status.is_server_error() {
                        attempts += 1;
                        if attempts >= max_attempts {
                            return Err(anyhow!(
                                "Yahoo Finance API failed with status {} after {} attempts for ticker {}",
                                status,
                                max_attempts,
                                ticker
                            ));
                        }
                        let msg = format!(
                            "Warning: Yahoo Finance API transient issue ({}). Retrying in {:.1}s... (Attempt {}/{})",
                            status,
                            retry_delay.as_secs_f32(),
                            attempts,
                            max_attempts
                        );
                        if let Some(ref pb) = self.pb {
                            pb.set_message(msg);
                        } else {
                            eprintln!("{}", msg);
                        }
                        tokio::time::sleep(retry_delay).await;
                        retry_delay *= 2;
                        continue;
                    }

                    if !status.is_success() {
                        return Err(anyhow!(
                            "Yahoo Finance API returned error status: {} for ticker {}",
                            status,
                            ticker
                        ));
                    }

                    let body = resp.text().await?;

                    // Validate response body parses as JSON before storing in cache
                    if let Err(e) = serde_json::from_str::<serde_json::Value>(&body) {
                        return Err(anyhow!(
                            "Invalid JSON response from Yahoo Finance for ticker {}: {}, body: {}",
                            ticker,
                            e,
                            body
                        ));
                    }

                    self.cache.insert(&url, &body).await?;
                    return Ok(body);
                }
                Err(e) => {
                    attempts += 1;
                    if attempts >= max_attempts {
                        return Err(anyhow!(
                            "Yahoo Finance API request failed: {} after {} attempts for ticker {}",
                            e,
                            max_attempts,
                            ticker
                        ));
                    }
                    let msg = format!(
                        "Warning: Yahoo Finance API request failed: {}. Retrying in {:.1}s... (Attempt {}/{})",
                        e,
                        retry_delay.as_secs_f32(),
                        attempts,
                        max_attempts
                    );
                    if let Some(ref pb) = self.pb {
                        pb.set_message(msg);
                    } else {
                        eprintln!("{}", msg);
                    }
                    tokio::time::sleep(retry_delay).await;
                    retry_delay *= 2;
                }
            }
        }
    }

    pub async fn ping(&self) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        let rounded_now = (now / 3600) * 3600;
        self.fetch_ticker_chart("^GSPC", rounded_now - 86400, rounded_now).await?;
        Ok(())
    }
}
