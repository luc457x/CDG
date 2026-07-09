# Backlog (BACKLOG.md)

Unvalidated ideas. Out of scope.

## Use

- Add idea/feature with short description.
- If approved, spec in [SPEC.md](./SPEC.md) + tasks in [TASKS.md](./TASKS.md).
- Never implement directly from backlog.

## Items

- **Configurable Risk-Free Ticker**: Allow configuring the risk-free asset ticker (currently hardcoded as `^TNX` for the US 10-Year Treasury yield) via CLI flags and environment variables.
- **`--light` Generalist Refactor (P8)**: Full `--light` mode — forces `concurrency=1`, skips Markowitz/plots/benchmarks, single-coin focus. Useful for CI, quick sanity checks, low-resource envs. Currently `--light` only overrides `days=30`.
- **Unified Multi-Source Ingestion (P9/B1)**: Auto-detect CoinGecko ↔ Yahoo Finance from ticker format with `resolve://` caching. Eliminates wrong-source footgun and redundant fetches.
- **CoinGecko ML Extras (P9/B2)**: Flag-gated hourly `market_chart`, `coins/markets`, `global`/`defi` endpoints as additional ML signal inputs (granularity + DeFi context).
- **Yahoo Macro Context (P9/B3)**: Use Yahoo Finance not as asset source, but as macro economy signal provider — fetch indices, yields, sector ETFs, and economic indicators to give CDG a local/global economic backdrop for portfolio decisions.
- **Structured Logging (beta)**: `tracing`/`env_logger` integration — `--verbose`/`--quiet` flags, persistent `cdg.log` writer. Required for unattended/CI runs.
- **Session Config Serialization**: Serialize full `PipelineConfig` to `cdg_session.json` after each run for exact reproducibility. Critical for ML experiment tracking as flag count grows.
- **Configurable Benchmark Ticker List**: Replace hardcoded `["^GSPC", "^DJI", "^IXIC", "^HSI", "^BVSP", "^TNX"]` with user-defined benchmark set via config/CLI.
- **God-Function Splitting (`analysis.rs`/`backtest.rs`)**: Break up `compute_returns_and_indicators` and `run_backtest_for_asset` into smaller focused fns. Maintainability refactor — no behavior change. Schedule before ML complexity compounds.
- **`sanitize_name` Whitelist Rewrite**: Replace blacklist with `[a-z0-9_-]` whitelist. Security hygiene — blacklists always have gaps. Small effort, high correctness gain.
- **Atomic Writes (`export.rs`)**: Replace `File::create` with tempfile+rename pattern. Prevents corrupt output files on crash/OOM kill. Critical for CI and unattended runs.
- **Cache Hardening (`cache.rs`)**: Add max-size guard to prevent unbounded disk growth + run `check_cache_hits` concurrently. Merge both into single refactor pass.
- **Plot Edge Case Fixes (`plot.rs`)**: `y_max == y_min` padding guard (flat-line/stablecoin crash) + single-series non-black color. One fix pass.
- **[HIGH] Replace `Xorshift` with `rand` crate**: Swap custom RNG for `rand` with seeded reproducibility. First post-alpha implementation — required for statistical validity and ML experiment repeatability.
- **Remove `#[allow(clippy::type_complexity)]` (MACD/Bollinger)**: Replace suppressed complex return types with named structs. Eliminates clippy debt and improves ML feature extraction readability.
- **`prediction_r2` as Strategy vs B&H (G7)**: Currently computed as actual vs strategy returns. Plan Option A: compare strategy equity curve vs buy-and-hold baseline. More meaningful signal quality metric.
- **Deterministic `BacktestReport` ordering (G8)**: `BacktestReport.metrics` uses `HashMap` — iteration order is non-deterministic. Replace with `IndexMap` (insertion-ordered) or sorted `BTreeMap` for stable JSON/CSV output across runs.
- **Remove `.clone()` in `align_datasets` join (G10)**: `analysis.rs` clones the full `other_df` DataFrame before `.lazy()`. Unnecessary allocation on every join — pass by reference or use `lazy()` directly.
- **Vectorize `drop_weekends` with polars (G11)**: `drop_weekends` uses a Rust loop with per-row string date parsing (~10k allocs on large datasets). Replace with polars `str.strptime` + `dt.weekday()` for vectorized filtering.
- **Document `unwrap_or(0.0)` in orderbook metrics (G14)**: `parse_coingecko_tickers` / `calculate_orderbook_metrics` use `unwrap_or(0.0)` for missing bid/ask/spread fields. Document intent or replace with `Option` propagation to prevent silent null poisoning in display data.
- **Convert inequality backtest assertions to exact (G15)**: Some equity tests (`test_macd_strategy_equity_exact`, `test_bollinger_*`) assert `>` / `== 1` instead of exact float values. Convert to deterministic exact assertions where feasible.
- **`with_context` on `create_dir_all` in `export.rs` (G16)**: `create_dir_all` failures propagate without path context. Add `.with_context(|| format!("failed to create dir: {}", path))` for debuggable error messages.
