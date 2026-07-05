# System Architecture

[🏠 Home](../README.md) • [📖 Overview](README.md) • [🏗️ Architecture](architecture.md) • [💻 Setup](installation_usage.md) • [🔌 API & Cache](api_cache.md) • [📊 Processing & Optimization](analysis_optimization.md) • [⚙️ Custom Strategies](custom_strategies.md) • [🚀 Deployment](deployment.md)

---

This document outlines the design principles, data pipelines, and component relationships of the **CryptoDataGather (CDG)** system.

CDG is built around a unidirectional pipeline that retrieves raw financial data, persists it through an asynchronous SQLite cache, processes and aligns it using high-performance DataFrames, and outputs clean analytical datasets and optimized portfolios.

---

## 1. High-Level Data Flow

The architecture is divided into three major stages: **Ingestion**, **Transformation**, and **Output**.

```mermaid
graph TD
    subgraph Ingestion Layer
        CG[CoinGecko API] -->|OHLCV, Market Chart, Tickers, Trending| Cache{SQLite Cache}
        YF[Yahoo Finance API] -->|Stock Index Benchmarks| Cache
    end

    subgraph Transformation Layer (Polars)
        Cache -->|Read/Write Cached JSON| Align[Data Alignment Engine]
        Align -->|Merge 24/7 Crypto & 5-Day Stocks| Feat[Feature Engineering & Indicators]
        Feat -->|--prep-ml Standard / MinMax Scaling| ML[ML Feature Prep]
    end

    subgraph Optimization & Analysis Layer
        ML --> Opt[Monte Carlo Sharpe Optimizer]
        Opt -->|Max Sharpe / Min Vol Weights| BT[Strategy & Portfolio Backtester]
        BT -->|Transaction Fees & Slippage| BT
        BT -->|Rebalancing daily/weekly/monthly| BT
    end

    subgraph Output & Visualizations
        Align -->|Parquet / CSV| Export[Data Exporter]
        Feat -->|PNG| Charts[Performance & Risk Charts]
        Opt -->|PNG| EF[Efficient Frontier]
        BT -->|PNG| Equity[Equity Curves]
    end
```

---

## 2. Core Components

CDG compiles as both a CLI binary (`src/main.rs`) and a reusable library (`src/lib.rs`). Its logic is separated into specialized modules:

### 🔌 API Clients & Caching (`src/api/`, `src/cache.rs`)
- **CoinGecko Client (`api::coingecko`)**: Wraps CoinGecko API calls for ping, supported currencies, coins list, markets, trending, tickers, market charts (days and timestamp range), OHLC, company treasury holdings, global data, and DeFi statistics.
- **Yahoo Finance Client (`api::yahoo`)**: Connects to the Yahoo Finance API to fetch benchmark index price histories (like S&P 500, Dow Jones, NASDAQ, HSI, BVSP, TNX).
- **SQLite CacheBackend (`cache`)**: Intercepts HTTP request URLs, hashing and storing response bodies in SQLite. Rounding start/end times to daily UTC boundaries guarantees identical URL cache keys for repeated daily queries.

### 📊 Data Processing & Alignment (`src/analysis.rs`)
- **Alignment**: Handles structural gaps between 24/7 crypto data and 5-day stock markets. Users can configure weekend gap handling:
  - **Forward-Fill (Default)**: Thursday/Friday closing prices are carried over to Saturday and Sunday.
  - **Drop Weekends**: Weekend rows are completely dropped from the final aligned DataFrame.
- **Indicators**: Uses `polars` expressions to calculate:
  - Base returns: Simple and Logarithmic returns.
  - Classic indicators: Simple Moving Average (SMA), Exponential Moving Average (EMA), Relative Strength Index (RSI), MACD, and Bollinger Bands.
  - Volume/OHLC indicators: Average True Range (ATR), On-Balance Volume (OBV), Stochastic Oscillator, and Average Directional Index (ADX).
- **Feature Scaling**: When `--prep-ml` is enabled, computes Standard Z-Score and MinMax scaling column-wise, creating new columns for downstream ML training.
- **Orderbook Metrics**: When fetching tickers via CoinGecko, computes average bid-ask spread, total exchange volume, and cross-exchange price standard deviation.

### 📊 Portfolio Optimization (`src/optimization.rs`)
- Executes a Monte Carlo simulation (10,000 iterations) using annualized return assumptions. The annualization factor defaults to `365.0` (or `252.0` when `--drop-weekends` is enabled).
- Determines the **Max Sharpe Ratio** and **Minimum Volatility** portfolio allocations.

### 🔄 Pipeline & Backtesting (`src/pipeline.rs`, `src/backtest.rs`)
- **Pipeline**: Orchestrates the full end-to-end workflow: cache init, API fetching, alignment, indicators, ML prep, export, plotting, optimization, and backtesting.
- **Backtester**: Supports built-in strategies (`rsi`, `macd`, `bollinger`, `all`) and custom JSON strategies. Evaluates transaction fees, slippage, and rebalancing frequencies. Compares strategy equity against buy-and-hold and US Treasury 10Y benchmarks. Exports CSV and JSON reports.

### 🎨 Visualizations & Plotting (`src/plot.rs`)
- Uses the `plotters` crate to generate line plots for performance charts, returns comparison charts, efficient frontier scatter plots, and backtest equity curves comparing strategy vs buy-and-hold.

### 🖥️ Interactive UI (`src/ui.rs`)
- Provides a terminal menu interface powered by `dialoguer` for interactive pipeline runs, pinging, listing coins, trending, raw OHLCV retrieval, coin ID validation, and cache TTL configuration.

### 📁 Export (`src/export.rs`)
- Handles CSV and Apache Parquet exports of aligned datasets.
