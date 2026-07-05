# Custom Backtesting Strategies

[🏠 Home](../README.md) • [📖 Overview](README.md) • [🏗️ Architecture](architecture.md) • [💻 Setup](installation_usage.md) • [🔌 API & Cache](api_cache.md) • [📊 Processing & Optimization](analysis_optimization.md) • [⚙️ Custom Strategies](custom_strategies.md) • [🚀 Deployment](deployment.md)

---

CDG supports defining custom technical backtesting strategies programmatically via structured JSON files, as well as built-in strategies (`rsi`, `macd`, `bollinger`, `all`). You can pass a custom strategy JSON file path to the `--strategy` CLI argument or the `CDG_BACKTEST_STRATEGY` environment variable.

When `--backtest` is enabled on `run-pipeline`, or when using the standalone `backtest` subcommand, CDG evaluates strategies on each target asset, computes transaction costs (`--fee`, `--slippage`), and optionally rebalances optimized portfolios at `daily`, `weekly`, or `monthly` frequencies. Results include equity curves, buy-and-hold comparisons, US Treasury 10Y benchmarks, and consolidated CSV/JSON reports.

---

## Strategy Schema Configuration

A custom strategy configuration consists of the following fields:

| Field | Type | Required | Description |
|---|---|---|---|
| `name` | String | Yes | Unique name for the strategy (used in summary output and exported CSV/JSON reports). |
| `buy_condition` | Condition | Yes | Logical rules that trigger a **Long (Buy)** signal (position signal = `2`). |
| `sell_condition` | Condition | Yes | Logical rules that trigger a **Short (Sell)** signal (position signal = `0`). |
| `neutral_condition` | Condition | No | Logical rules that trigger a **Neutral (Cash)** signal (position signal = `1`). If omitted or `null`, the backtester holds the previous position when neither buy nor sell conditions are met. |
| `confidence` | ConfidenceConfig | No | Configures position sizing / confidence factor (`conf_t`, bounded between `0.1` and `1.0`) to scale returns. Defaults to a constant `1.0` if omitted. |

---

## Condition Structure

Conditions represent logical logic trees. A condition can be either a **Logical operator** (combining nested sub-rules) or a **Comparison rule** (evaluating column values).

### 1. Logical Operator Rules
Logical rules combine other conditions using boolean algebra:
- **`AND`**: All nested rules in the `rules` array must evaluate to `true`.
- **`OR`**: At least one nested rule in the `rules` array must evaluate to `true`.
- **`NOT`**: Inverts the nested rule (requires exactly one rule in the `rules` array).

Example of an `AND` condition:
```json
{
  "operator": "AND",
  "rules": [
    { "column": "rsi_14", "operator": "<", "value": 30.0 },
    { "column": "close", "operator": "<", "value": "bollinger_lower" }
  ]
}
```

### 2. Comparison Rules
Comparison rules compare a DataFrame column's value against a static numeric value or another column:
- **`column`**: The column name (e.g. `"rsi_14"`). Names are resolved dynamically; if the exact name does not exist, the backtester automatically looks for a coin-prefixed version (e.g. `bitcoin_usd_rsi_14`).
- **`shift`** (optional, default: `0`): Historical offset/lookback. For example, `shift: 1` compares the indicator's value from the *previous* day.
- **`operator`**: Comparison operator. Mathematical comparison signs are fully supported and recommended:
  - **`<`** (or `LT`, `lt`): Less than.
  - **`>`** (or `GT`, `gt`): Greater than.
  - **`==`** (or `EQ`, `eq`): Equal to.
  - **`<=`** (or `LTE`, `lte`): Less than or equal to.
  - **`>=`** (or `GTE`, `gte`): Greater than or equal to.
- **`value`**: The target value for comparison. Can be:
  - A numeric constant (e.g. `30.0`).
  - Another column reference object: `{"column": "close", "shift": 1}`.

Example comparing to a shifted column value (e.g., close price is higher than yesterday's close):
```json
{
  "column": "close",
  "operator": ">",
  "value": {
    "column": "close",
    "shift": 1
  }
}
```

---

## Confidence and Position Sizing

The `confidence` config determines how the strategy scales position size (`conf_t` from `0.1` to `1.0`):

### 1. Constant Sizing
Uses a static sizing multiplier.
```json
{
  "type": "Constant",
  "value": 1.0
}
```

### 2. Linear Scaling Sizing
Scales the position dynamically based on the absolute value of a pre-calculated DataFrame column (e.g. MACD histogram or ATR):
- **`column`**: The column to scale against.
- **`shift`** (optional, default: `0`): Offset for the scaling indicator.
- **`min`** / **`max`**: Bounds to clamp the final multiplier value.
- **`multiplier`** (optional, default: `1.0`): Flat factor multiplied by the column value before clamping.

Formula: `conf_t = clamp(abs(col_value) * multiplier, min, max)`

```json
{
  "type": "LinearScale",
  "column": "macd_histogram",
  "multiplier": 2.5,
  "min": 0.1,
  "max": 1.0
}
```

---

## Built-in Strategies

If `--strategy` is set to a built-in name instead of a `.json` path, CDG uses pre-defined logic:

| Strategy | Buy Signal | Sell Signal | Notes |
| :--- | :--- | :--- | :--- |
| `rsi` | RSI < 30 | RSI > 70 | Confidence scales with distance from thresholds. |
| `macd` | MACD Line > Signal Line | MACD Line < Signal Line | Confidence scales by histogram volatility. |
| `bollinger` | Price < Lower Band | Price > Upper Band | Confidence scales by distance from band. |
| `all` | All three strategies above executed sequentially. | | |

---

## Backtest Execution Details

When running `run-pipeline --backtest` or the standalone `backtest` subcommand:

- **Transaction Costs**: Every trade incurs `fee + slippage` deducted from equity.
- **Rebalancing Frequency**: Optimized portfolio backtests can rebalance `daily`, `weekly`, or `monthly`.
- **US Treasury Benchmark**: If Yahoo Finance `^TNX` data is present, CDG computes a 10-Year Treasury buy-and-hold benchmark for risk-adjusted comparison.
- **Reports**: 
  - Console ASCII table with strategy return, buy-and-hold return, Sharpe ratio, max drawdown, win rate, and rating.
  - CSV report: `{run_dir}/backtests/backtest_report.csv`
  - JSON report: `{run_dir}/backtests/backtest_report.json`
- **Plots**: Per-asset equity curve PNGs and optimized portfolio rebalanced equity PNGs are saved under `{run_dir}/backtests/`.

---

## Examples

### Example 1: Single Strategy JSON (`custom_rsi.json`)
A simple RSI mean-reversion strategy:
```json
{
  "name": "custom_rsi",
  "buy_condition": {
    "column": "rsi_14",
    "operator": "<",
    "value": 30.0
  },
  "sell_condition": {
    "column": "rsi_14",
    "operator": ">",
    "value": 70.0
  },
  "neutral_condition": null,
  "confidence": {
    "type": "Constant",
    "value": 1.0
  }
}
```

### Example 2: Multi-Strategy List JSON (`multi_rsi.json`)
You can define multiple strategies in a JSON array. Both will be executed sequentially during backtesting:
```json
[
  {
    "name": "rsi_oversold",
    "buy_condition": { "column": "rsi_14", "operator": "<", "value": 30.0 },
    "sell_condition": { "column": "rsi_14", "operator": ">", "value": 70.0 }
  },
  {
    "name": "rsi_momentum",
    "buy_condition": { "column": "rsi_14", "operator": ">", "value": 50.0 },
    "sell_condition": { "column": "rsi_14", "operator": "<", "value": 50.0 }
  }
]
```

### Example 3: Multi-Strategy Key-Value Map JSON (`multi_map.json`)
Alternatively, structure strategies as key-value pairs. If the `"name"` field inside a strategy is empty, the backtester automatically falls back to using the map key as the strategy name:
```json
{
  "aggressive_rsi": {
    "name": "",
    "buy_condition": { "column": "rsi_14", "operator": "<", "value": 35.0 },
    "sell_condition": { "column": "rsi_14", "operator": ">", "value": 65.0 }
  },
  "conservative_rsi": {
    "name": "conservative_rsi_explicit",
    "buy_condition": { "column": "rsi_14", "operator": "<", "value": 25.0 },
    "sell_condition": { "column": "rsi_14", "operator": ">", "value": 75.0 }
  }
}
```

---

## CLI & Environment Variable Integration

To run backtests using your custom strategy configuration file:

### CLI Subcommands
```powershell
# Run custom strategy backtest on Bitcoin
cargo run -- backtest --coin bitcoin --currency usd --days 90 --strategy /cdg_files/custom.json

# Run custom strategies as part of the full data processing pipeline
cargo run -- run-pipeline --coin bitcoin --currency usd --days 90 --backtest --strategy /cdg_files/custom.json
```

### Environment Variable
Clap automatically loads the strategy from the `CDG_BACKTEST_STRATEGY` environment variable if defined:
```powershell
# Set env variable
$env:CDG_BACKTEST_STRATEGY="cdg_files/custom_rsi.json"

# Run backtest (reads strategy from env automatically)
cargo run -- backtest --coin bitcoin --currency usd --days 90
```
