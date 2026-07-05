# Setup & Usage

[🏠 Home](../README.md) • [📖 Overview](README.md) • [🏗️ Architecture](architecture.md) • [💻 Setup](installation_usage.md) • [🔌 API & Cache](api_cache.md) • [📊 Processing & Optimization](analysis_optimization.md) • [⚙️ Custom Strategies](custom_strategies.md) • [🚀 Deployment](deployment.md)

---

This guide provides step-by-step instructions on setting up, building, and running **CryptoDataGather (CDG)**.

---

## 1. Prerequisites

Ensure you have the following installed on your machine:
- **Rust Toolchain**: `rustc` and `cargo` (minimum version matching Rust Edition 2021). You can install them via [rustup](https://rustup.rs/).
- **SQLite3**: Required by the cache backend (built automatically via standard Cargo dependencies).

---

## 2. Installation & Build

Clone the repository and build the project in release mode:

```bash
# Clone the repository
git clone https://github.com/luc457x/CDG
cd CDG

# Build the project
cargo build --release
```

The compiled binary will be located at `target/release/cdg` (or `target/release/cdg.exe` on Windows).

---

## 3. Usage Modes

CDG can be executed in two different ways: **Interactive Menu Mode** and **CLI Subcommand Mode**.

### 📱 Interactive Menu Mode

If you run the application with no arguments or subcommands, it automatically enters the interactive terminal menu:

```bash
cargo run
```

This mode clears the terminal and presents a menu driven by the keyboard:
- **Run Pipeline**: Run full data fetching, indicators, optimization, and backtesting pipeline interactively.
- **Ping Services**: Ping the API servers to verify connectivity.
- **List Supported Coins**: Displays the top 50 cryptocurrencies by market cap in a clean terminal table.
- **Get Trending Coins**: Retrieves and prints current trending cryptocurrencies from CoinGecko.
- **Get Raw OHLCV Data**: Queries, displays, and exports raw historical candlestick data.
- **Check Coin ID Validity**: Validates if a coin exists on CoinGecko, and returns suggestions if not.
- **Settings**: Sub-menu to configure cache TTL dynamically (under **Configure Cache TTL**).
- **Exit**: Close the application.

> [!NOTE]
> Selecting any action in interactive mode clears the screen, runs the action, and shows a `[Back]` button. Choosing `[Back]` returns you to the main action menu.
> Interactive mode automatically intercepts standard Ctrl+C terminal signals for graceful shutdown.

---

### 💻 CLI Subcommand Mode

For automation, cron jobs, and scripting, you can trigger specific subcommands directly.

#### 1. Running the Pipeline (`run-pipeline`)
This is the primary tool that collects data, calculates indicators, generates ML features, runs Monte Carlo simulations, and saves outputs.

```bash
cargo run -- run-pipeline -c bitcoin,ethereum -v usd -d 90 --prep-ml
```

#### 2. Ping Services (`ping`)
Verifies active HTTP connections and response status from both CoinGecko and Yahoo Finance.

```bash
cargo run -- ping
```

#### 3. List Supported Coins (`list-coins`)
Fetches and lists the top 50 cryptocurrencies sorted by market capitalization.

```bash
cargo run -- list-coins
```

#### 4. Get Trending Coins (`trending`)
Lists coins currently trending on CoinGecko.

```bash
cargo run -- trending
```

#### 5. Check Coin ID (`check-coin`)
Validates a coin ID and checks for spelling suggestions. Accepts the coin ID as a positional argument.

```bash
cargo run -- check-coin btc
```

#### 6. Get Raw Candlesticks (`ohlcv`)
Retrieves raw candlestick data for a coin.

```bash
cargo run -- ohlcv -c bitcoin -v usd --days 30 --format csv
```

#### 7. Run Strategy Backtest (`backtest`)
Runs a standalone strategy backtest on a coin with configurable fees, slippage, and rebalancing frequency. Supports built-in strategies (`rsi`, `macd`, `bollinger`, `all`) or a custom JSON strategy file via `--strategy`.

```bash
cargo run -- backtest -c bitcoin -v usd -d 90 --strategy rsi --fee 0.001 --slippage 0.0005 --rebalance-frequency daily
```

---

## 4. CLI Arguments Reference

### Global Flags

- `--output-dir <dir>`: Base output directory (default: `cdg_files`, overridable via `CDG_OUTPUT_DIR` environment variable).
- `--db-path <path>`: SQLite database cache path (default: `{output_dir}/cache.db`).
- `-o`, `--output-prefix <prefix>`: Prefix for generated pipeline outputs (default: `{output_dir}/output`).
- `--raw-format <format>`: Raw OHLCV export format: `json` or `csv` (default: `json`, overridable via `CDG_RAW_FORMAT` environment variable).
- `--cache-ttl <seconds>`: Cache TTL in seconds (default: `300` / 5 minutes).

### Subcommand `run-pipeline` Options

| Option / Flag | Type | Description | Default |
| :--- | :--- | :--- | :--- |
| `-c`, `--coin` | `String` | Comma-separated list of CoinGecko coin IDs (e.g. `bitcoin,ethereum`) | `bitcoin` |
| `-v`, `--currency` | `String` | Comma-separated list of fiat currencies (e.g. `usd,eur`) | `usd` |
| `-d`, `--days` | `u32` | Timeframe in days for historical data retrieval | `90` |
| `--prep-ml` | `bool` | Enables Standard Z-Score and MinMax scaling feature columns | `false` |
| `--light` | `bool` | Enables Lightweight Mode (forces coin=bitcoin, days=30, skips benchmarks) | `false` |
| `--drop-weekends` | `bool` | Drops Saturday/Sunday data rows instead of forward-filling stocks | `false` |
| `--seed` | `u64` | Seed value for Monte Carlo simulation RNG | `None` |
| `--concurrency` | `usize` | CoinGecko query concurrency limit (default: 1 for demo/free keys, 3 for pro keys, overridable via `COINGECKO_CONCURRENCY`) | `None` |
| `--annualization-factor` | `f64` | Override annualization factor for returns/volatility calculations (e.g. 252 or 365). Defaults to 252 when `--drop-weekends` is set, otherwise 365. | `None` |
| `--backtest` | `bool` | Run strategy and portfolio backtesting at the end of the pipeline | `false` |
| `--strategy` | `String` | Built-in strategy (`rsi`, `macd`, `bollinger`, or `all`) or path to a custom strategy JSON file. Overridable via `CDG_BACKTEST_STRATEGY`. | `rsi` |
| `--fee` | `f64` | Transaction fee as decimal | `0.001` |
| `--slippage` | `f64` | Slippage as decimal | `0.0005` |
| `--rebalance-frequency` | `String` | Portfolio rebalancing frequency: `daily`, `weekly`, or `monthly` | `daily` |

### Subcommand `backtest` Options

| Option / Flag | Type | Description | Default |
| :--- | :--- | :--- | :--- |
| `-c`, `--coin` | `String` | Coin ID to backtest | `bitcoin` |
| `-v`, `--currency` | `String` | Vs fiat currency | `usd` |
| `-d`, `--days` | `u32` | Timeframe in days | `90` |
| `-s`, `--strategy` | `String` | Built-in strategy (`rsi`, `macd`, `bollinger`, or `all`) or path to a custom strategy JSON file. Overridable via `CDG_BACKTEST_STRATEGY`. | `rsi` |
| `--fee` | `f64` | Transaction fee as decimal | `0.001` |
| `--slippage` | `f64` | Slippage as decimal | `0.0005` |
| `--rebalance-frequency` | `String` | Portfolio rebalancing frequency: `daily`, `weekly`, or `monthly` | `daily` |

### Subcommand `ohlcv` Options

| Option / Flag | Type | Description | Default |
| :--- | :--- | :--- | :--- |
| `-c`, `--coin` | `String` | Coin ID to retrieve | `bitcoin` |
| `-v`, `--currency` | `String` | Vs currency to retrieve | `usd` |
| `-d`, `--days` | `u32` | Timeframe in days | `90` |
| `-f`, `--format` | `String` | Output format: `stdout`, `csv`, or `json` | `stdout` |
