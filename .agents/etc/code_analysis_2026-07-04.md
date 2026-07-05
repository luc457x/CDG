# CDG Code Analysis

**Date:** 2026-07-04  
**Scope:** Full repository static analysis — no edits performed  
**Lines of Code:** ~5,289 Rust across 11 source modules  

---

## Architecture Overview

| Layer | Modules | Responsibility |
|-------|---------|---------------|
| **CLI** | `main.rs`, `ui.rs` | `clap` subcommands + `dialoguer` interactive TUI |
| **API** | `api/coingecko.rs`, `api/yahoo.rs` | HTTP clients for CoinGecko + Yahoo Finance |
| **Cache** | `cache.rs` | Async SQLite cache via `sqlx` + `tokio` with WAL mode |
| **Analysis** | `analysis.rs` | Polars-based parsing, alignment, technical indicators, ML prep |
| **Optimization** | `optimization.rs` | Monte Carlo Markowitz with xorshift RNG, deterministic seed support |
| **Backtest** | `backtest.rs` | Strategy engine (RSI/MACD/Bollinger), custom JSON configs, buy-and-hold, rebalancing |
| **Pipeline** | `pipeline.rs` | Orchestrates fetch → align → indicators → ML prep → export → plot → optimize → backtest |
| **Export** | `export.rs` | CSV + Parquet writers |
| **Plot** | `plot.rs` | PNG chart generation via `plotters` |
| **Utils** | `utils.rs` | Path safety, name sanitization |

---

## Strengths

1. **Clean module separation** — API, cache, analysis, optimization, backtest, plot, export are properly isolated with clear boundaries.
2. **Path safety** — `utils::validate_safe_path` proactively rejects `..` traversal before filesystem operations.
3. **Determinism respected** — xorshift RNG with `--seed` flag makes Monte Carlo reproducible, matching STDD requirements.
4. **Good async patterns** — `Arc<Cache>`, semaphore-based concurrency control, `JoinSet` for parallel fetches.
5. **Input validation** — env var-based CLI defaults, format checks (`json/csv`), safe path enforcement.
6. **Reasonable test coverage** for critical paths — optimization determinism, pipeline smoke, path safety, export round-trips.

---

## Issues

### 1. [HIGH] Two god-functions in `pipeline.rs` (~770 LOC + ~280 LOC)

`run_pipeline_flow` and `run_standalone_backtest` share ~200 lines of duplicated backtest/report logic (US Treasury benchmark, CSV/JSON export, backtest table formatting).

**Impact:** High maintenance cost, bug fixes must be applied twice.

**Recommendation:** Extract shared logic into `backtest::run_backtest_report(df, cols, metrics, report_dir, ...)`.

---

### 2. [HIGH] Zero unit tests in `backtest.rs` (~1,411 LOC)

Strategy math, custom JSON config parsing, Sharpe/max-drawdown calculations are unvalidated.

**Impact:** Financial correctness unverified. Subtle bugs in RSI/MACD/Bollinger logic or edge-case handling of `None` indicators will fail silently in production.

**Recommendation:** Add unit tests for each strategy signal generation, custom config deserialization, and edge cases (insufficient data, all-NaN columns).

---

### 3. [HIGH] Zero unit tests in `analysis.rs` (~1,161 LOC)

Indicator math (RSI, MACD, Bollinger, ATR, Stochastic, ADX, OBV) is complex and mostly untested.

**Impact:** Incorrect technical indicators propagate to backtest and optimization results.

**Recommendation:** Add golden-value tests using known price series with hand-calculated indicator outputs.

---

### 4. [MEDIUM] Ctrl+C handler abandons resources

`pipeline.rs:157-161` — `tokio::spawn` + `std::process::exit(0)` on Ctrl+C.

**Impact:** Background `JoinSet` tasks, file handles, and WAL locks are abandoned without cleanup. Partial output files may be left in inconsistent state.

**Recommendation:** Use a cancellation token (`CancellationToken`) propagated to all async tasks, then wait for graceful shutdown before exiting.

---

### 5. [MEDIUM] Inconsistent error semantics for invalid coins

`pipeline.rs:186-200` — 404 errors silently skip invalid coins but other errors re-add the coin to the pipeline.

**Impact:** User gets partial pipeline without knowing which coins were dropped.

**Recommendation:** Uniform behavior — either always skip with a warning, or always fail hard. Document the choice.

---

### 6. [MEDIUM] Strategy `.json` config files loaded without existence check

`pipeline.rs:489`, `pipeline.rs:993` — `backtest::load_custom_strategies(strategy)` reads from disk but no prior `fs::metadata` check.

**Impact:** A typo in the strategy path could panic or return a confusing error.

**Recommendation:** Validate file existence early and return a clear error message.

---

### 7. [LOW] `sanitize_name` is a fragile defense-in-depth

`utils.rs:13-16` — only replaces `^` and `-`. Works because `validate_safe_path` catches traversal, but adding new unsafe characters later could bypass it.

**Impact:** Low — combined with path validation it's currently safe.

**Recommendation:** Consider a whitelist approach (`[a-z0-9_-]`) instead of blacklisting specific characters.

---

### 8. [LOW] Unnecessary clone pressure on large DataFrames

`pipeline.rs:365, 372` — `final_df` is mutated in-place but passed as `&mut` only for export. No ownership transfer means extra clone pressure.

**Impact:** Memory overhead on large datasets.

**Recommendation:** Pass `final_df` by value (or use `Arc<DataFrame>`) where mutation is no longer needed after export.

---

## GCP / Vertex Compatibility Notes

- SQLite cache with WAL mode is Cloud Run-friendly (local disk, fast startup).
- Lightweight mode explicitly targets GCP free-tier — good alignment.
- Parquet/CSV export exists but BigQuery compatibility is unimplemented (no native BigQuery auth/load path).
- No environment-based config for secrets loading (`.env` support absent — relies solely on `clap` env args).

---

## Summary

**Verdict:** Solid foundational code with clean architecture and good async patterns. The primary risks are maintainability (god-functions) and correctness (untested math-heavy modules). No security-critical issues found beyond the minor sanitization observation, which is currently mitigated by `validate_safe_path`. Safe to refactor following STDD.
