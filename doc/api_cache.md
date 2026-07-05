# API Clients & Caching

[🏠 Home](../README.md) • [📖 Overview](README.md) • [🏗️ Architecture](architecture.md) • [💻 Setup](installation_usage.md) • [🔌 API & Cache](api_cache.md) • [📊 Processing & Optimization](analysis_optimization.md) • [⚙️ Custom Strategies](custom_strategies.md) • [🚀 Deployment](deployment.md)

---

This guide describes how the system interacts with external APIs and manages request caching through a local database to prevent rate limit blocks.

---

## 1. Yahoo Finance Client (`src/api/yahoo.rs`)

The Yahoo Finance client fetches historical price charts for traditional stock market indexes used as comparison benchmarks (e.g., S&P 500 (`^GSPC`)).

- **HTTP Engine**: Built on `reqwest::Client` with a standard browser User-Agent header to avoid automated scraper detection blocks.
- **Endpoints**:
  - `GET /v8/finance/chart/{ticker}?period1={start}&period2={end}&interval=1d`
- **Ping Check**: Pings Yahoo Finance by querying the previous 24 hours of data for the S&P 500 index (`^GSPC`) rounded to the nearest hour.

---

## 2. CoinGecko Client (`src/api/coingecko.rs`)

The CoinGecko API client retrieves historical prices, cryptocurrency market stats, trending assets, exchange tickers, and corporate treasury data.

- **API Keys**: Supports Demo and Pro API keys via `COINGECKO_DEMO_API_KEY`, `COINGECKO_PRO_API_KEY`, or generic `COINGECKO_API_KEY` + `COINGECKO_API_KEY_TYPE=pro`. Pro keys switch the base URL to `pro-api.coingecko.com`.
- **Rate Limit Handlers (429 Retry)**:
  - On receiving an HTTP `429 Too Many Requests` response, it enters an exponential backoff loop, retrying up to **4 times** (doubling delay times, starting at **10 seconds**).
  - There is no fixed pre-request delay on cache misses.
- **Endpoint Integrations**:
  - `/ping`: Check system connectivity.
  - `/simple/supported_vs_currencies`: Retrieve vs currencies list.
  - `/coins/list`: Get a list of all coins.
  - `/coins/markets`: Fetch market data sorted by market cap.
  - `/simple/price`: Quick price lookup by coin IDs.
  - `/global`: Global cryptocurrency market data.
  - `/companies/public_treasury/{coin_id}`: Public company holdings for a coin.
  - `/coins/{id}/tickers`: Exchange tickers for a specific coin (used for orderbook metrics).
  - `/coins/{id}/market_chart`: Historical price chart for a coin (days-based range).
  - `/coins/{id}/market_chart/range`: Historical price chart for a coin (Unix timestamp range).
  - `/coins/{id}/ohlc`: Retrieve historical open-high-low-close candlestick data.
  - `/search/trending`: Returns the top trending coins.
  - `/global/decentralized_finance_defi`: DeFi global market data.

---

## 3. Orderbook Metrics (`src/analysis.rs`)

During every `run-pipeline` execution, CDG fetches exchange tickers for each target coin via `GET /coins/{id}/tickers` and computes aggregate orderbook quality metrics:

- **Average Bid-Ask Spread**: Mean `bid_ask_spread_percentage` across tracked exchanges.
- **Total Ticker Volume**: Sum of `volume` across exchange tickers.
- **Price Standard Deviation**: Standard deviation of `last_price` across exchanges, indicating cross-exchange price dispersion.

These metrics are printed to the terminal at the start of each pipeline run.

---

## 4. SQLite CacheBackend (`src/cache.rs`)

To guarantee high efficiency and compliance with strict external API rate limits, CDG implements a persistent local caching mechanism powered by SQLite (`sqlx` + `tokio`).

### Cache Table Schema

The database initializes a single table `api_cache` with the following structure:

```sql
CREATE TABLE IF NOT EXISTS api_cache (
    url TEXT PRIMARY KEY,
    response_body TEXT NOT NULL,
    cached_at_timestamp INTEGER NOT NULL
);
```

### Cache Lifetime & Logic

- **Trait Interface**: Exposes a generic `CacheBackend` trait:
  ```rust
  #[async_trait::async_trait]
  pub trait CacheBackend: Send + Sync {
      async fn get(&self, url: &str, ttl_secs: i64) -> Result<Option<String>>;
      async fn insert(&self, url: &str, body: &str) -> Result<()>;
  }
  ```
- **Read Operation (`get`)**: Hashed request URLs serve as primary keys. If a record matches the URL key:
  - If `current_timestamp - cached_at_timestamp < TTL`, it returns the cached response body string.
  - The default TTL is **300 seconds** (5 minutes). This can be customized using the `--cache-ttl` global command-line option, or configured dynamically in the interactive menu under the **"Settings"** -> **"Configure Cache TTL"** action.
  - Otherwise, it deletes the expired cache row and returns `None` (cache miss).
- **Write Operation (`insert`)**: Saves response bodies alongside the current UNIX timestamp, automatically overwriting old keys (`INSERT OR REPLACE`).

---

## 5. Cache Optimization: Timestamp Boundary Alignment

CryptoDataGather needs to fetch historical data for custom time periods (e.g. `--days 90`). Standard dynamic API timestamp ranges create minor hourly changes in request URLs, leading to cache misses.

To optimize cache hits, CDG rounds the start and end UNIX timestamps of historical queries to the nearest **daily UTC boundary** (`00:00:00 UTC`). This ensures that multiple pipeline runs on the same calendar day use the exact same request URLs and hit the SQLite cache instead of making network requests.
