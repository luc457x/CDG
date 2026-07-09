use crate::cache::CacheBackend;
use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct CoinSuggestion {
    pub id: String,
    pub symbol: String,
    pub name: String,
}

pub struct CoinGeckoClient {
    client: Client,
    cache: Arc<dyn CacheBackend>,
    base_url: String,
    ttl_secs: i64,
    demo_key: Option<String>,
    pro_key: Option<String>,
    pb: Option<indicatif::ProgressBar>,
    retry_delay_ms: u64,
}

impl CoinGeckoClient {
    pub fn new(cache: Arc<dyn CacheBackend>) -> Result<Self> {
        let mut base_url = "https://api.coingecko.com/api/v3".to_string();
        let mut demo_key = std::env::var("COINGECKO_DEMO_API_KEY").ok();
        let mut pro_key = std::env::var("COINGECKO_PRO_API_KEY").ok();

        if demo_key.is_none() && pro_key.is_none() {
            if let Ok(key) = std::env::var("COINGECKO_API_KEY") {
                let key_type = std::env::var("COINGECKO_API_KEY_TYPE").unwrap_or_default();
                if key_type.to_lowercase() == "pro" {
                    pro_key = Some(key);
                } else {
                    demo_key = Some(key);
                }
            }
        }

        if pro_key.is_some() {
            base_url = "https://pro-api.coingecko.com/api/v3".to_string();
        }

        Ok(Self {
            client: Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
                .timeout(std::time::Duration::from_secs(30))
                .build()?,
            cache,
            base_url,
            ttl_secs: 300, // 5 minutes cache default
            demo_key,
            pro_key,
            pb: None,
            retry_delay_ms: 10000,
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

    pub fn with_retry_delay_ms(mut self, ms: u64) -> Self {
        self.retry_delay_ms = ms;
        self
    }

    async fn get_request(
        &self,
        endpoint: &str,
        query_params: &[(&str, &str)],
        use_cache: bool,
    ) -> Result<String> {
        // Build canonical URL via reqwest so query params are percent-encoded
        let base_endpoint = format!("{}{}", self.base_url, endpoint);
        let cache_url = if query_params.is_empty() {
            base_endpoint.clone()
        } else {
            reqwest::Url::parse_with_params(&base_endpoint, query_params)
                .map(|u| u.to_string())
                .unwrap_or(base_endpoint.clone())
        };

        if use_cache {
            if let Some(cached) = self.cache.get(&cache_url, self.ttl_secs).await? {
                return Ok(cached);
            }
        }

        let mut attempts = 0;
        let max_attempts = 4;
        let mut retry_delay = std::time::Duration::from_millis(self.retry_delay_ms);

        loop {
            // Fetch from API using reqwest's built-in query encoding
            let mut request_builder = self.client.get(&base_endpoint).query(query_params);

            if let Some(ref key) = self.demo_key {
                request_builder = request_builder.header("x-cg-demo-api-key", key);
            } else if let Some(ref key) = self.pro_key {
                request_builder = request_builder.header("x-cg-pro-api-key", key);
            }

            let response = match request_builder.send().await {
                Ok(r) => r,
                Err(e) => {
                    // Retry on timeout or connection errors
                    let is_retryable = e.is_timeout() || e.is_connect() || e.is_request();
                    attempts += 1;
                    if !is_retryable || attempts >= max_attempts {
                        return Err(anyhow!(
                            "CoinGecko request failed after {} attempt(s): {}",
                            attempts,
                            e
                        ));
                    }
                    let msg = format!(
                        "Warning: CoinGecko network error: {}. Retrying in {:.1}s... (Attempt {}/{})",
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
                    continue;
                }
            };

            let status = response.status();

            if status == 429 || status.is_server_error() {
                attempts += 1;
                if attempts >= max_attempts {
                    return Err(anyhow!(
                        "CoinGecko API returned {} after {} attempts",
                        status,
                        max_attempts
                    ));
                }
                let msg = format!(
                    "Warning: CoinGecko API returned {}. Retrying in {:.1}s... (Attempt {}/{})",
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
                return Err(anyhow!("CoinGecko API returned error status: {}", status));
            }

            let body = response.text().await?;

            // Validate response body parses as JSON before storing in cache
            if let Err(e) = serde_json::from_str::<serde_json::Value>(&body) {
                return Err(anyhow::anyhow!(
                    "Invalid JSON response from CoinGecko: {}, body: {}",
                    e,
                    body
                ));
            }

            if use_cache {
                self.cache.insert(&cache_url, &body).await?;
            }

            return Ok(body);
        }
    }

    pub async fn ping(&self) -> Result<Value> {
        let res = self.get_request("/ping", &[], true).await?;
        Ok(serde_json::from_str(&res)?)
    }

    pub async fn get_supported_vs_currencies(&self) -> Result<Vec<String>> {
        let res = self
            .get_request("/simple/supported_vs_currencies", &[], true)
            .await?;
        Ok(serde_json::from_str(&res)?)
    }

    pub async fn get_coins_list(&self) -> Result<Vec<Value>> {
        // /coins/list is static-ish; cache with 24h TTL regardless of client default.
        const COINS_LIST_TTL: i64 = 86400;
        let endpoint = "/coins/list";
        let base_endpoint = format!("{}{}", self.base_url, endpoint);

        // Check cache first with 24h TTL
        if let Some(cached) = self.cache.get(&base_endpoint, COINS_LIST_TTL).await? {
            return Ok(serde_json::from_str(&cached)?);
        }

        // Not cached — perform network request (reuse get_request with cache disabled, then store manually)
        let body = self.get_request(endpoint, &[], false).await?;
        self.cache.insert(&base_endpoint, &body).await?;
        Ok(serde_json::from_str(&body)?)
    }

    pub async fn get_global(&self) -> Result<Value> {
        let res = self.get_request("/global", &[], true).await?;
        Ok(serde_json::from_str(&res)?)
    }

    pub async fn get_companies_public_treasury(&self, coin_id: &str) -> Result<Value> {
        let endpoint = format!("/companies/public_treasury/{}", coin_id);
        let res = self.get_request(&endpoint, &[], true).await?;
        Ok(serde_json::from_str(&res)?)
    }

    pub async fn get_coins_markets(&self, vs_currency: &str, per_page: u32) -> Result<Vec<Value>> {
        let per_page_str = per_page.to_string();
        let query = [
            ("vs_currency", vs_currency),
            ("order", "market_cap_desc"),
            ("per_page", &per_page_str),
            ("page", "1"),
            ("sparkline", "false"),
        ];
        let res = self.get_request("/coins/markets", &query, true).await?;
        Ok(serde_json::from_str(&res)?)
    }

    pub async fn get_price(&self, coin_ids: &str, vs_currencies: &str) -> Result<Value> {
        let query = [
            ("ids", coin_ids),
            ("vs_currencies", vs_currencies),
            ("include_market_cap", "true"),
            ("include_24hr_vol", "true"),
            ("include_24hr_change", "true"),
        ];
        let res = self.get_request("/simple/price", &query, true).await?;
        Ok(serde_json::from_str(&res)?)
    }

    pub async fn get_coin_market_chart(
        &self,
        coin_id: &str,
        vs_currency: &str,
        days: &str,
    ) -> Result<Value> {
        let endpoint = format!("/coins/{}/market_chart", coin_id);
        let query = [("vs_currency", vs_currency), ("days", days)];
        let res = self.get_request(&endpoint, &query, true).await?;
        Ok(serde_json::from_str(&res)?)
    }

    pub async fn get_coin_market_chart_range(
        &self,
        coin_id: &str,
        vs_currency: &str,
        from: i64,
        to: i64,
    ) -> Result<Value> {
        let endpoint = format!("/coins/{}/market_chart/range", coin_id);
        let from_str = from.to_string();
        let to_str = to.to_string();
        let query = [
            ("vs_currency", vs_currency),
            ("from", &from_str),
            ("to", &to_str),
        ];
        let res = self.get_request(&endpoint, &query, true).await?;
        Ok(serde_json::from_str(&res)?)
    }

    pub async fn get_coin_ohlc(
        &self,
        coin_id: &str,
        vs_currency: &str,
        days: &str,
    ) -> Result<Vec<Vec<f64>>> {
        let endpoint = format!("/coins/{}/ohlc", coin_id);
        let query = [("vs_currency", vs_currency), ("days", days)];
        let res = self.get_request(&endpoint, &query, true).await?;
        Ok(serde_json::from_str(&res)?)
    }

    pub async fn get_coin_tickers(&self, coin_id: &str, page: Option<u32>) -> Result<Value> {
        let endpoint = format!("/coins/{}/tickers", coin_id);
        let mut query = Vec::new();
        let page_str = page.map(|p| p.to_string());
        if let Some(ref p_str) = page_str {
            query.push(("page", p_str.as_str()));
        }
        let res = self.get_request(&endpoint, &query, true).await?;
        Ok(serde_json::from_str(&res)?)
    }

    pub async fn get_search_trending(&self) -> Result<Value> {
        let res = self.get_request("/search/trending", &[], true).await?;
        Ok(serde_json::from_str(&res)?)
    }

    pub async fn get_global_decentralized_finance_defi(&self) -> Result<Value> {
        let res = self
            .get_request("/global/decentralized_finance_defi", &[], true)
            .await?;
        Ok(serde_json::from_str(&res)?)
    }

    pub async fn check_coin_id(&self, input: &str) -> Result<Option<Vec<CoinSuggestion>>> {
        let coin_list = self.get_coins_list().await?;
        let input_lower = input.to_lowercase();

        // Check exact ID match
        let mut exact_id_found = false;
        for coin in &coin_list {
            if let Some(id) = coin.get("id").and_then(|v| v.as_str()) {
                if id.to_lowercase() == input_lower {
                    exact_id_found = true;
                    break;
                }
            }
        }

        if exact_id_found {
            return Ok(None);
        }

        // Gather suggestions
        let mut suggestions = Vec::new();

        // 1. Exact symbol matches
        for coin in &coin_list {
            let id = coin
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let symbol = coin
                .get("symbol")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let name = coin
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            if symbol.to_lowercase() == input_lower {
                suggestions.push(CoinSuggestion { id, symbol, name });
            }
        }

        // 2. Substring matches
        if suggestions.len() < 10 {
            for coin in &coin_list {
                let id = coin
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let symbol = coin
                    .get("symbol")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let name = coin
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                if suggestions.iter().any(|s| s.id == id) {
                    continue;
                }

                if id.to_lowercase().contains(&input_lower)
                    || name.to_lowercase().contains(&input_lower)
                {
                    suggestions.push(CoinSuggestion { id, symbol, name });
                    if suggestions.len() >= 10 {
                        break;
                    }
                }
            }
        }

        Ok(Some(suggestions))
    }
}
