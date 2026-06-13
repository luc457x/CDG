use crate::cache::Cache;
use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::Value;

pub struct CoinGeckoClient {
    client: Client,
    cache: Cache,
    base_url: String,
    ttl_secs: i64,
}

impl CoinGeckoClient {
    pub fn new(cache: Cache) -> Self {
        Self {
            client: Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
                .build()
                .unwrap(),
            cache,
            base_url: "https://api.coingecko.com/api/v3".to_string(),
            ttl_secs: 300, // 5 minutes cache default
        }
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
        // Build full URL
        let mut url = format!("{}{}", self.base_url, endpoint);
        if !query_params.is_empty() {
            let query_str = query_params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<String>>()
                .join("&");
            url = format!("{}?{}", url, query_str);
        }

        if use_cache {
            if let Some(cached) = self.cache.get(&url, self.ttl_secs).await? {
                return Ok(cached);
            }
        }

        // Rate-limiting delay on cache misses to prevent CoinGecko API 429
        #[cfg(not(test))]
        {
            tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
        }

        // Fetch from API
        let response = self.client.get(&url).send().await?;

        if response.status() == 429 {
            return Err(anyhow!("CoinGecko API Rate Limit Exceeded (429)"));
        }

        if !response.status().is_success() {
            return Err(anyhow!(
                "CoinGecko API returned error status: {}",
                response.status()
            ));
        }

        let body = response.text().await?;

        if use_cache {
            self.cache.insert(&url, &body).await?;
        }

        Ok(body)
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
}
