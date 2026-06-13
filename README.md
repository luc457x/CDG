# CryptoDataGather (CDG) - Rust Edition

A robust, modular, and performance-efficient Rust application and library to collect, cache, and pre-process market data from CoinGecko (cryptocurrency) and Yahoo Finance (traditional stock benchmarks).

This project has been implemented in Rust, optimizing hosting footprint and data alignment speed while utilizing `polars` for fast DataFrames operations and `plotters` for generating charts.

## Features

- **Multi-Source Fetching**: Queries CoinGecko API for cryptocurrency prices and scrapes Yahoo Finance API for traditional market benchmarks (like S&P 500).
- **SQLite Caching Layer**: Uses an asynchronous SQLite caching system (`sqlx` + `tokio`) to persist API response bodies locally, preventing API rate limit blocks.
- **Data Alignment**: Merges 24/7 cryptocurrency data with 5-day traditional stock data using either weekend forward-fill alignment (default) or weekend dropping.
- **Technical Indicators**: Calculates simple returns, log returns, SMA, EMA, RSI, MACD, and Bollinger Bands using high-performance `polars` expressions.
- **ML Preprocessing**: Normalizes all indicators/prices using MinMax scaling and Standard Z-Score normalization (`--prep-ml`) for downstream Python / PyTorch / Jupyter ML training.
- **Plotting**: Generates high-quality PNG visualization charts for normalized performance, percentage returns, and risk/return scatter profiles.
- **Lightweight Mode**: A memory-friendly mode optimized for GCP free-tier Cloud Run / micro instances that limits calculations to Bitcoin and the last 30 days of data.
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

### Commands

- **Basic pipeline run** (default parameters: bitcoin vs USD, 90 days range, includes benchmarks):
  ```bash
  cargo run
  ```

- **Lightweight mode** (restricts footprint; fetches bitcoin only, 30 days range, skips stock benchmarks):
  ```bash
  cargo run -- --light
  ```

- **Enable Machine Learning scaling**:
  ```bash
  cargo run -- --prep-ml
  ```

- **Drop weekends** (instead of forward-filling Friday's stock prices):
  ```bash
  cargo run -- --drop-weekends
  ```

- **Full Options Help**:
  ```bash
  cargo run -- --help
  ```

### CLI Arguments

| Flag | Long Option | Description | Default |
|------|-------------|-------------|---------|
| `-c` | `--coin` | Coin ID from CoinGecko | `bitcoin` |
| `-v` | `--currency`| Vs currency for CoinGecko | `usd` |
| `-d` | `--days` | Timeframe in days to retrieve | `90` |
| | `--prep-ml` | Enable MinMax and Z-Score scaling | `false` |
| | `--light` | Enable GCP lightweight constraints | `false` |
| | `--drop-weekends`| Drop weekends instead of forward-filling stock data | `false` |
| | `--db-path` | SQLite cache file path | `cdg_files/cache.db` |
| `-o` | `--output-prefix` | Output file path prefix | `cdg_files/output` |
