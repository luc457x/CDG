# CryptoDataGather (CDG)

A robust, modular, and performance-efficient Rust application and library to collect, cache, and pre-process market data from CoinGecko (cryptocurrency) and Yahoo Finance (traditional stock benchmarks).

This project has been implemented in Rust, optimizing hosting footprint and data alignment speed while utilizing `polars` for fast DataFrames operations and `plotters` for generating charts.

## 📖 Documentation

A comprehensive set of modular guides is available under the `doc/` directory:
- **[Overview & Documentation Hub](doc/README.md)**
- **[System Architecture](doc/architecture.md)**
- **[Setup & CLI Usage Guide](doc/installation_usage.md)**
- **[API Clients & SQLite Caching](doc/api_cache.md)**
- **[Feature Engineering & Portfolio Math](doc/analysis_optimization.md)**
- **[Deployment & Cloud Operations](doc/deployment.md)**

## Features

- **Multi-Source Fetching**: Queries CoinGecko API for cryptocurrency prices and scrapes Yahoo Finance API for traditional market benchmarks (like S&P 500).
- **SQLite Caching Layer**: Uses an asynchronous SQLite caching system (`sqlx` + `tokio`) to persist API response bodies locally, preventing API rate limit blocks.
- **Data Alignment**: Merges 24/7 cryptocurrency data with 5-day traditional stock data using either weekend forward-fill alignment (default) or weekend dropping.
- **Technical Indicators**: Calculates simple returns, log returns, SMA, EMA, RSI, MACD, and Bollinger Bands using high-performance `polars` expressions.
- **ML Preprocessing**: Normalizes all indicators/prices using MinMax scaling and Standard Z-Score normalization (`--prep-ml`) for downstream Python / PyTorch / Jupyter ML training.
- **Plotting**: Generates high-quality PNG visualization charts for normalized performance, percentage returns, and risk/return scatter profiles.
- **Lightweight Mode**: A memory-friendly mode optimized for GCP free-tier Cloud Run / micro instances that limits calculations to Bitcoin and the last 30 days of data.
- **Portfolio Optimization**: Runs annualized Monte Carlo portfolio simulation to determine Max Sharpe Ratio and Minimum Volatility weights.
- **CLI Polish**: Integrates real-time progress feedback (spinners/progress bars) and logs final optimal portfolios in clean ASCII tables.
- **Clean Architecture**: Compiles as both a CLI tool (`cdg`) and a reusable library (`cdg`).


## Installation

Ensure you have Rust and Cargo installed.

```bash
git clone https://github.com/luc457x/CDG
cd CDG
cargo build --release
```

## Usage

Run the compiled binary using `cargo run`.

### Interactive Menu Mode
If run without any subcommand, the program launches the **Interactive Menu Mode**, providing a terminal UI to select and configure actions (e.g. running the pipeline, pinging servers, listing coins, checking coin IDs, and getting raw OHLCV data):
```bash
cargo run
```

### CLI Subcommands
For automation and scripting, the following subcommands are supported:

- **run-pipeline**: Runs the data collection, alignment, indicators computation, and portfolio optimization pipeline:
  ```bash
  cargo run -- run-pipeline -c bitcoin,ethereum -v usd -d 90
  ```
- **ping**: Pings CoinGecko and Yahoo Finance API servers to verify connectivity:
  ```bash
  cargo run -- ping
  ```
- **list-coins**: Fetches and displays the top 50 cryptocurrencies by market cap (USD):
  ```bash
  cargo run -- list-coins
  ```
- **trending**: Shows the current trending search coins on CoinGecko:
  ```bash
  cargo run -- trending
  ```
- **ohlcv**: Fetches and prints/exports raw candlestick data:
  ```bash
  cargo run -- ohlcv -c bitcoin -v usd --days 30 --format csv
  ```
- **check-coin**: Validates a CoinGecko ID and suggests close matches if invalid:
  ```bash
  cargo run -- check-coin --coin bnb
  ```

### CLI Arguments & Options Reference (for `run-pipeline`)

| Flag | Long Option | Description | Default |
| :--- | :--- | :--- | :--- |
| `-c` | `--coin` | Coin ID or comma-separated list of IDs from CoinGecko (e.g. `bitcoin,ethereum`) | `bitcoin` |
| `-v` | `--currency` | Vs currency or comma-separated list of fiat currencies (e.g. `usd,eur,brl`) | `usd` |
| `-d` | `--days` | Historical timeframe in days to retrieve | `90` |
| | `--prep-ml` | Enable MinMax and Z-Score features generation (see details below) | `false` |
| | `--light` | Enable Lightweight Mode (forces coin=bitcoin, days=30, skips benchmarks) | `false` |
| | `--drop-weekends` | Drop weekend data points instead of forward-filling traditional stock data | `false` |
| | `--db-path` | SQLite cache database file path | `cdg_files/cache.db` |
| `-o` | `--output-prefix` | Output file path prefix | `cdg_files/output` |
| | `--seed` | Optional RNG seed for deterministic Monte Carlo simulation | `None` |

---

## Technical Details & Pipeline Concepts

### 1. Machine Learning Preprocessing (`--prep-ml`)

When run with the `--prep-ml` flag, the pipeline generates normalized features for downstream model training (e.g., PyTorch, TensorFlow, or Scikit-Learn). 

**Important Behavior:**
- It does **not** overwrite or normalize your original columns in-place.
- Instead, it **creates two new columns** for every numerical column `{name}` in the aligned dataset (except the `date` column) using two separate normalization strategies:
  1. **MinMax Scaling (`{name}_minmax`)**:
     $$x_{\text{minmax}} = \frac{x - x_{\text{min}}}{x_{\text{max}} - x_{\text{min}}}$$
     This scales all values to fit strictly between `0.0` and `1.0`. It is highly useful for models sensitive to absolute value scales (like Neural Networks).
  2. **Standard Z-Score Scaling (`{name}_standard`)**:
     $$x_{\text{standard}} = \frac{x - \mu}{\sigma}$$
     *(where $\mu$ is the column mean and $\sigma$ is the column standard deviation)*.
     This shifts values to have a mean of `0.0` and a standard deviation of `1.0`. It is ideal for linear models, PCA, and models requiring zero-centered features.
- If a column has no valid values or zero variance, scaling is bypassed for safety.

### 2. Caching Layer & Timestamp Alignment

The SQLite cache avoids rate limit blocks (429 errors) and speeds up repeated runs:
- Request URLs are hashed and stored in the SQLite database with their response bodies.
- A default 5-minute Time-To-Live (TTL) is enforced.
- **Timestamp Boundary Alignment:** To make cache hits reliable across multiple runs throughout the same day, start and end timestamps for API range requests are rounded to the nearest daily boundary (e.g. `00:00:00 UTC`). This ensures that the generated query URL remains identical and hits the cache rather than triggering a new network request.
- **Demo Rate Limiter:** If a cache miss occurs, the client automatically waits for 2 seconds before calling the CoinGecko API. This delay prevents rate limits when fetching multiple assets or currencies.

### 3. Weekend Gap Alignment (`--drop-weekends`)

Cryptocurrency markets trade 24/7, while traditional stock markets (e.g., S&P 500) close on weekends. To merge them:
- **Default (Forward-Fill):** The traditional stock benchmarks copy Friday's closing values over to Saturday and Sunday. This aligns the datasets without losing weekend cryptocurrency data points.
- **Drop Weekends (`--drop-weekends`):** Any Saturday and Sunday rows are completely removed from the aligned DataFrame. Traditional benchmark data remains aligned directly to weekdays.

### 4. Technical Indicators Calculations

For all coin-currency combinations, the library calculates high-performance technical indicators via `polars`:
- **Returns**: Simple return and logarithmic return.
- **Moving Averages**: 20-period Simple Moving Average (SMA) and Exponential Moving Average (EMA).
- **RSI**: 14-period Relative Strength Index (Wilder's smoothing).
- **MACD**: Moving Average Convergence Divergence line, signal line, and histogram.
- **Bollinger Bands**: 20-period upper and lower bands (2 standard deviations).
- **Advanced Indicators** (only calculated for coins when OHLCV columns exist):
  - **ATR (Average True Range)**: 14-period smoothed volatility measure.
  - **Stochastic Oscillator**: 14-period `%K` and 3-period `%D` lines.
  - **ADX (Average Directional Index)**: 14-period Welles Wilder smoothed index showing trend strength.
  - **OBV (On-Balance Volume)**: Running volume total relative to price direction.

### 5. Portfolio Optimization & Markowitz Monte Carlo

When there are at least two assets to compare, the program automatically runs a Monte Carlo simulation of 10,000 random portfolios:
- **Expected Return and Volatility**: Calculations are annualized (using a standard 365-day year assumption for cryptocurrency assets).
- **Optimization Outputs**:
  - **Max Sharpe Ratio**: The portfolio maximizing expected excess returns per unit of volatility (assuming risk-free rate = 0).
  - **Minimum Volatility**: The portfolio with the absolute lowest risk.
- **Repeatability**: Use the `--seed <value>` flag to generate identical portfolio random allocations across runs (default: 1337).

---

## Output Files Structure

At the start of every pipeline run or standalone OHLCV retrieval, directories are created under `cdg_files/`:

### 1. Run Directory (for pipelines)
`cdg_files/run_YYYYMMDD_HHMMSS/`

Contains the complete aligned dataset, optimal weights, and generated charts:
```
cdg_files/run_20260613_091730/
├── data.csv                # Complete aligned dataset (prices, indicators, scaled features)
├── data.parquet            # Parquet format of the aligned dataset (optimized for Pandas/ML)
├── portfolio_weights.csv   # Max Sharpe Ratio and Minimum Volatility optimal allocations
├── efficient_frontier.png  # Scatter plot of simulated portfolios and frontier
├── performance.png         # Line plot comparing asset performance normalized to 100%
├── risk_return.png         # Scatter plot showing return mean vs volatility risk
└── bitcoin_usd_returns.png # Returns line charts for each coin-currency pair
```

### 2. Raw OHLCV Directory (created for raw exports)
`cdg_files/can_YYYYMMDD_HHMMSS/`

Contains raw fetched candlestick data in both JSON and CSV format for each coin-currency pair:
```
cdg_files/can_20260613_091730/
├── bitcoin_usd.csv         # Raw OHLCV data in CSV format (timestamp, open, high, low, close)
└── bitcoin_usd.json        # Raw OHLCV data in JSON format
```
