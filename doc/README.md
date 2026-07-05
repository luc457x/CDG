# CryptoDataGather (CDG) Documentation Hub

[🏠 Home](../README.md) • [📖 Overview](README.md) • [🏗️ Architecture](architecture.md) • [💻 Setup](installation_usage.md) • [🔌 API & Cache](api_cache.md) • [📊 Processing & Optimization](analysis_optimization.md) • [⚙️ Custom Strategies](custom_strategies.md) • [🚀 Deployment](deployment.md)

---

Welcome to the comprehensive documentation hub for the **CryptoDataGather (CDG)** system. This documentation is organized into modular guides covering everything from system design and API interactions to installation, custom analytical formulas, strategy backtesting, and deployment workflows.

## System Overview

CDG is a robust, modular, and performance-efficient Rust application and library designed to fetch, cache, align, and process financial and cryptocurrency market data.

It is structured to run efficiently on low-cost compute resources (such as Google Cloud Platform's Free Tier), utilizing:
- **SQLite** as an asynchronous local caching layer to prevent API rate limits.
- **Polars** for fast column-oriented DataFrame manipulation, alignment, and technical indicator calculations.
- **Plotters** for rendering candlestick charts, portfolio efficient frontier visualizations, and backtest equity curves.

Additional capabilities include:
- **Orderbook Metrics**: Real-time average spread, volume, and price dispersion metrics from CoinGecko exchange tickers.
- **Strategy Backtesting**: Built-in strategies (`rsi`, `macd`, `bollinger`, `all`) and custom JSON-defined strategies with transaction fees, slippage, and rebalancing support.
- **Portfolio Backtesting**: Rebalanced portfolio equity curves compared against buy-and-hold and US Treasury 10Y benchmarks.

---

## Documentation Directory Map

Use the following links to navigate the documentation:

1. **[🏗️ System Architecture](architecture.md)**
   - High-level design, data flow, and components diagram (Mermaid).
2. **[💻 Setup & Usage](installation_usage.md)**
   - Quickstart, interactive menu mode, CLI subcommand reference, and configuration flags.
3. **[🔌 API Clients & Caching](api_cache.md)**
   - Implementation details of CoinGecko and Yahoo Finance API clients, plus the SQLite persistent caching layer.
4. **[📊 Data Processing & Portfolio Optimization](analysis_optimization.md)**
   - Explains technical indicator calculations (SMA, EMA, RSI, MACD, Bollinger Bands, ATR, OBV, Stochastic, ADX), machine learning preprocessing/scaling (`--prep-ml`), and the Markowitz Mean-Variance optimization engine.
5. **[⚙️ Custom Strategies](custom_strategies.md)**
    - Built-in strategies (`rsi`, `macd`, `bollinger`, `all`), defining custom JSON logic trees, operators, shifts, and sizing, plus backtesting mechanics (fees, slippage, rebalancing, reports).
6. **[🚀 Deployment & Operations](deployment.md)**
   - Standard directory layouts (`cdg_files/`), environment variable requirements, and containerization/GCP orchestration patterns.
