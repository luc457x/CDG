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
}

impl CoinGeckoClient {
    pub fn new(cache: Arc<dyn CacheBackend>) -> Result<Self> {
        Ok(Self {
            client: Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
                .build()?,
            cache,
            base_url: "https://api.coingecko.com/api/v3".to_string(),
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
        let mut retry_delay = std::time::Duration::from_millis(10000);

        loop {
            if attempts == 0 {
                // Rate-limiting delay on cache misses to prevent CoinGecko API 429
                #[cfg(not(test))]
                {
                    tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
                }
            }

            // Fetch from API using reqwest's built-in query encoding
            let response = self
                .client
                .get(&base_endpoint)
                .query(query_params)
                .send()
                .await?;

            let status = response.status();

            if status == 429 {
                attempts += 1;
                if attempts >= max_attempts {
                    return Err(anyhow!(
                        "CoinGecko API Rate Limit Exceeded (429) after {} attempts",
                        max_attempts
                    ));
                }
                eprintln!(
                    "Warning: CoinGecko API Rate Limit Exceeded (429). Retrying in {:.1}s... (Attempt {}/{})",
                    retry_delay.as_secs_f32(),
                    attempts,
                    max_attempts
                );
                tokio::time::sleep(retry_delay).await;
                retry_delay *= 2;
                continue;
            }

            if !status.is_success() {
                return Err(anyhow!("CoinGecko API returned error status: {}", status));
            }

            let body = response.text().await?;

            if use_cache {
                self.cache.insert(&cache_url, &body).await?;
            }

            return Ok(body);
        }
    }

    pub async fn ping(&self) -> Result<Value> {
        let res = self.get_request("/ping", &[], false).await?;
        Ok(serde_json::from_str(&res)?)
    }

    pub async fn get_supported_vs_currencies(&self) -> Result<Vec<String>> {
        let res = self
            .get_request("/simple/supported_vs_currencies", &[], true)
            .await?;
        Ok(serde_json::from_str(&res)?)
    }

    pub async fn get_coins_list(&self) -> Result<Vec<Value>> {
        let res = self.get_request("/coins/list", &[], true).await?;
        Ok(serde_json::from_str(&res)?)
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
        let page_str;
        let query = if let Some(p) = page {
            page_str = p.to_string();
            vec![("page", page_str.as_str())]
        } else {
            vec![]
        };
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
