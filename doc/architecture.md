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
        CG[CoinGecko API] -->|Cryptocurrency OHLCV & Market Cap| Cache{SQLite Cache}
        YF[Yahoo Finance API] -->|Stock Index Benchmarks| Cache
    end

    subgraph Transformation Layer (Polars)
        Cache -->|Read/Write Cached JSON| Align[Data Alignment Engine]
        Align -->|Merge 24/7 Crypto & 5-Day Stocks| Feat[Feature Engineering & Indicators]
        Feat -->|--prep-ml Standard / MinMax Scaling| Opt[Monte Carlo Sharpe Optimizer]
    end

    subgraph Output & Visualizations
        Opt -->|ASCII Table & CSV| Export[Parquet / CSV Exporter]
        Opt -->|plotters Crate| PNG[Visual Charts PNG]
    end
```

---

## 2. Core Components

CDG compiles as both a CLI binary (`src/main.rs`) and a reusable library (`src/lib.rs`). Its logic is separated into specialized modules:

### 🔌 API Clients & Caching (`src/api/`, `src/cache.rs`)
- **CoinGecko Client (`api::coingecko`)**: Wraps CoinGecko API calls for `/ping`, `/coins/list`, `/coins/markets` (top coins sorted by market cap), `/search/trending`, and `/coins/{id}/ohlc`.
- **Yahoo Finance Client (`api::yahoo`)**: Connects to the Yahoo Finance API to scrap benchmark index price histories (like S&P 500, Dow Jones, NASDAQ).
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

### 📊 Portfolio Optimization (`src/optimization.rs`)
- Executes a Monte Carlo simulation (10,000 iterations) using standard annualized return assumptions (365 days) for cryptocurrency assets.
- Determines the **Max Sharpe Ratio** and **Minimum Volatility** portfolio allocations.

### 🎨 Visualizations & Plotting (`src/plot.rs`)
- Uses the `plotters` crate to generate line plots for performance charts, returns comparison charts, candlestick charts for markets, and scatter plots representing the Efficient Frontier.
