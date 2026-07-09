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
