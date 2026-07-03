# Spec (SPEC.md)

## 1. Overview & Goals

Port the python CoinGecko and Yahoo Finance data collector to a robust, modular, and performance-efficient Rust application. Pre-process the data for analysis and downstream machine learning tasks, with low GCP hosting costs.

- **Goal 1**: Gather cryptocurrency data (CoinGecko) and traditional market benchmark data (Yahoo Finance).
- **Goal 2**: Cache raw API responses locally/persistently using SQLite (`sqlx` + `tokio`) to avoid API rate limits.
- **Goal 3**: Align and process data (returns, normalizations, and technical indicators) using `polars`.
- **Goal 4**: Export clean pre-processed data to Parquet for Python/Jupyter/ML training.
- **Goal 5**: Provide basic data visualizations (candlesticks, returns, portfolio allocation) using the `plotters` crate.
- **Goal 6**: Ensure complete architecture and output compatibility with GCP (Cloud SQL, BigQuery, GCS) and Vertex AI pipelines.

## 2. Scope

- **In-Scope**:
  - Full rewrite of `cdg` Python modules to Rust.
  - CoinGecko endpoints: ping, currency support list, coin search/list, markets, company treasuries, OHLC, historical prices.
  - Yahoo Finance scraper/client in Rust to get S&P500 and other benchmark stock indexes.
  - Data alignment: merging 24/7 crypto data with 5-day stock data (default: forward-fill weekends; configurable: drop weekends).
  - Feature engineering: log returns, simple returns, SMA, EMA, RSI, MACD, and Bollinger Bands calculated via `polars`.
  - ML preprocessing pipeline (`--prep-ml`) with Standard Z-Score and MinMax scaling.
  - Parquet and CSV export formats.
  - Simple PNG plots using the `plotters` crate (candles, risk/return, performance, allocation).
  - Cost optimizations: configurable `--light` mode to limit queries/data volume (fetch only Bitcoin and last 30 days of data).
  - Multi-cryptocurrency support: parsing a list of coins (e.g. `bitcoin,ethereum`) and fiat currencies (e.g. `usd,eur`), and calculating technical indicators for all queried combinations.
- **Out-of-Scope**:
  - Building/training ML models.
  - Web UI frontend (all operations run via CLI or cron).
  - Multi-tenant authentication.

## 3. Tech Stack

- **Language**: Rust (edition 2021)
- **Runtime**: `tokio` (async)
- **HTTP Client**: `reqwest` (async) + `serde` for JSON deserialization
- **DB/Cache**: SQLite via `sqlx` (async, schema-ready for future GCP Cloud SQL migration)
- **DataFrames**: `polars` (highly optimized column-oriented processing)
- **Visualization**: `plotters` (graphing and PNG generation)
- **CLI Framework**: `clap` (arguments parsing)

## 4. Functional Requirements (FR)

- **FR01 (API Fetching)**: Client must fetch real-time and historical coin data from CoinGecko, and benchmark index data from Yahoo Finance.
- **FR02 (Caching)**: SQLite database must cache API response bodies keyed by request URL with a default 5-minute expiry.
- **FR03 (Data Alignment)**: Merging crypto and traditional stock data must handle weekend gap alignment using configurable options (forward-fill vs drop weekends).
- **FR04 (Feature Engineering)**: Calculate technical indicators (SMA, EMA, RSI, MACD, Bollinger Bands) and simple/log returns over historical price series.
- **FR05 (ML Preparation)**: Scale/normalize features (MinMax and Standard scaling) and export them into Parquet/CSV format via `--prep-ml`.
- **FR06 (Plotting)**: Generate line plots for returns/performance and candlestick charts for markets, saving to PNG files.
- **FR07 (Cost optimization/Lightweight Mode)**: Reduce CPU/memory/network footprint when running with the `--light` flag by restricting queries to the last 30 days of history and skipping traditional stock benchmarks.
- **FR08 (Multi-Cryptocurrency & Multi-Currency)**: Support comma-separated lists of coins and currencies, fetching all combinations and naming them `{coin}_{currency}`.
- **FR09 (Conditional Indicators)**: Compute indicators for all combinations by default, but only for the first coin-currency combination under `--light` mode.
- **FR10 (Conditional Plotting)**: Generate a separate returns line plot for each coin-currency combination, but skip all plots under `--light` mode.
- **FR11 (CLI Subcommands)**: The application must support subcommands (`run-pipeline`, `ping`, `list-coins`, `trending`, `ohlcv`, `check-coin`) using `clap`.
- **FR12 (Interactive Mode)**: When no subcommand is specified, the application must run in an interactive mode prompting the user with a menu selection using `dialoguer`.
- **FR13 (Context Retrieval)**: Clients must provide methods for listing coins by market cap (`get_coins_markets`), trending search (`get_search_trending`), and raw OHLCV querying.
- **FR14 (Signal Handling)**: Interactive loop must handle terminal interrupt signals (Ctrl+C) gracefully.
- **FR15 (Raw OHLCV Export)**: Subcommand and interactive menu for OHLCV must support printing raw OHLCV to stdout or exporting it to JSON/CSV files.
- **FR16 (Raw OHLCV Folder Export)**: During pipeline runs, raw OHLCV files (in the configured raw format, defaulting to JSON) must be saved inside a folder named `raw_ohlcv` within the pipeline run directory.
- **FR17 (Interactive CLI Less-like Pager UX)**: In interactive mode, selecting any action must clear the terminal, run the action, print all info to stdout, and display a `[Back]` button. Selecting `[Back]` returns to the main actions menu (which is also displayed on a cleared terminal).
- **FR18 (Parallel Ingestion Concurrency Control)**: Parallel ingestion of CoinGecko charts and OHLC data must use a semaphore-based concurrency control defaulting to 3, configurable via CLI flag `--concurrency` and env var `COINGECKO_CONCURRENCY`.
- **FR19 (Asset-Specific Annualization)**: Portfolio expected returns and volatilities must be annualized using dynamic factors (365.0 for Crypto, 252.0 for Traditional stocks/benchmarks). An override CLI flag `--annualization-factor` or env var `ANNUALIZATION_FACTOR` can override all factors to a single custom value.
- **FR20 (Configurable Output Directory)**: The application must support a configurable base output directory `--output-dir` (default: `cdg_files`) and environment variable `CDG_OUTPUT_DIR`. All run and candlestick outputs, as well as the default SQLite database path and default output prefix, must resolve dynamically relative to this output directory.
- **FR21 (Configurable Raw Format)**: The application must support a configurable raw format option `--raw-format` (default: `json`) and environment variable `CDG_RAW_FORMAT`, supporting `json` and `csv`. All raw OHLCV files saved during pipeline runs or standalone OHLCV retrieval must be generated in this format only.


## 5. Business Rules (BR)

- **BR01**: Default coin identifier is always `bitcoin` (BTC) and default currency is `usd`.
- **BR02**: Cache hits bypass network requests entirely if `cached_at` is less than 5 minutes old (or configurable).
- **BR03**: Forward-fill weekend stock data replicates Friday's closing values to Saturday and Sunday.
- **BR04**: The heuristic to determine asset class maps assets in the CoinGecko coin list or ending in `-USD`/`-EUR` to Crypto (365.0), and others (like index symbols starting with `^` or standard tickers) to Traditional (252.0).

## 6. Non-Functional Requirements (NFR)

- **NFR01 (Perf)**: System memory consumption under `--light` mode must remain below 100MB to fit within GCP Free Tier Cloud Run / VM instances.
- **NFR02 (Portability)**: Output formats must be standardized (Parquet, CSV) to integrate natively with Python Pandas/Jupyter and GCP BigQuery/Cloud Storage.
- **NFR03 (Extensibility)**: DB layer using `sqlx` must compile-time verify queries, allowing migration to GCP PostgreSQL/MySQL with minimal code changes. Offline compilation is supported using prepared `.sqlx/` metadata.
- **NFR04 (SQLite Performance)**: SQLite cache database connection must enable Write-Ahead Logging (WAL) mode (`journal_mode=WAL`) and `synchronous=NORMAL` to prevent concurrent write database locking errors.
