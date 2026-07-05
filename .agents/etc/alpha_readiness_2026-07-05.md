# CDG Alpha-Readiness Deep Review

**Date:** 2026-07-05  
**Scope:** Full codebase audit — what must be fixed, improved, or validated BEFORE an alpha release  
**Methodology:** Read every source file, traced control flow, inspected error paths, math logic, async safety, CLI/UX contracts, and test counts. No edits performed.  

---

## 1. Alpha-Blocking Issues (P0 — Release cannot proceed)

### 1.1 Zero unit tests for `analysis.rs` indicator math
**What is missing:** `analysis.rs` is ~1,161 LOC. It contains hand-rolled implementations of RSI, MACD, Bollinger Bands, ATR, Stochastic Oscillator, ADX, and OBV. There are `#[cfg(test)]` blocks but **every test only checks column existence** (`assert!(res.column("bitcoin_usd_rsi_14").is_ok())`). None verify numerical correctness against known inputs/outputs.

**Why it blocks alpha:** If any indicator formula has an off-by-one, wrong smoothing, or sign error, every downstream backtest result, portfolio weight, and ML feature is silently corrupted. An alpha user will find bugs immediately if they compare CDG outputs to TradingView or Excel.

**What you need to do:**
- For each indicator, write a golden-value test using a small price series where you manually compute the expected result (or use a Python script with `pandas_ta` / `ta` library to generate reference values).
- At minimum: 3 test vectors per indicator (uptrend, downtrend, flat/sideways) covering edge cases like `period > data length`, `NaN` propagation, and `close == open` boundary.
- Minimum: 15–20 new unit tests in `analysis.rs`.

### 1.2 Zero unit tests for `backtest.rs` strategy signal generation + portfolio rebalancing math
**What is missing:** `backtest.rs` is ~1,411 LOC. It has tests for `calculate_mda`, `calculate_r2`, `calculate_sharpe`, `calculate_max_drawdown`, and a couple of integration smoke tests. But there are **no tests for the actual signal generation loop** (`run_backtest_for_asset`) validating that RSI < 30 triggers a buy, MACD cross triggers correctly, Bollinger touch triggers correctly, or that the custom strategy JSON config evaluates conditions accurately.

**Why it blocks alpha:** Strategy backtesting is the primary value proposition. If the signal logic is wrong (e.g., using `prev_t` but applying fee to the wrong bar, or `conf_t` scaling is wrong), alpha users will lose trust.

**What you need to do:**
- Write tests that feed deterministic price/indicator data into `run_backtest_for_asset` and assert exact equity curve values at each bar.
- Test all three built-in strategies (RSI, MACD, Bollinger) with known prices.
- Test edge cases: price gaps, zero volume days, all `None` indicator columns, single-bar DataFrame.
- Test `backtest_portfolio` with weekly/monthly rebalancing asserting exact fee deductions and weight drift between rebalances.
- Minimum: 10–15 new unit tests.

### 1.3 Ctrl+C kills the process with `std::process::exit(0)`, abandoning async tasks
**What is missing:** `pipeline.rs:157-161` spawns a Ctrl+C handler that calls `std::process::exit(0)`. This bypasses Drop destructors for `JoinSet` tasks, leaves SQLite WAL handles open, and can corrupt the output directory if a file write is mid-flight.

**Why it blocks alpha:** On Cloud Run or any container, SIGTERM is common. A tool that leaves half-written CSVs and locked DB files is not alpha-ready.

**What you need to do:**
- Replace `std::process::exit(0)` with a `CancellationToken` from `tokio_util` spread across all async tasks.
- On cancel, wait for `JoinSet` to drain, close file handles, checkpoint SQLite, then return `Err` from `main()` for a clean exit code (or `Ok(())` if user-initiated).

### 1.4 `analysis.rs`: returns are computed on filtered slice but OHLC/ATR/Stoch use `0.0` for nulls without flagging data quality
**What is missing:** In `compute_returns_and_indicators`, null prices are dropped, returns are computed on the filtered slice, then scattered back. But for OHLC-based indicators (ATR, Stochastic, ADX, OBV), missing high/low values are replaced with `0.0` (`highs_raw[i].unwrap_or(0.0)`). This produces garbage indicator values silently.

**Why it blocks alpha:** A dataset with a single missing OHLC bar will cause ATR to spike to infinity, ADX to collapse to 0, and OBV to jump. These silent corruptions will propagate to backtest and portfolio weight results.

**What you need to do:**
- If `high` or `low` is `None` at index `i`, the corresponding ATR/Stoch/ADX/OBV output at `i` must be `None` (or the entire advanced indicator block must be gated behind a data-quality check).
- Add a `data_quality_warnings: Vec<String>` return or logger so the user knows which dates had missing OHLC and were filled with `0.0`.

### 1.5 `backtest.rs`: strategy returns for "neutral" signal compute `0.0` but equity track uses `eq_base` unchanged — fee logic is inconsistent
**What is missing:** In `run_backtest_for_asset` (lines 708-723):
```rust
let mut eq_base = equity[prev_t];
if sig_t != current_position {
    eq_base *= 1.0 - (fee + slippage);
    total_trades += 1;
}
let r_strat = if sig_t == 2 { r_t * conf_t } else if sig_t == 0 { -r_t * conf_t } else { 0.0 };
equity[t] = eq_base * (1.0 + r_strat);
```
If `sig_t` is neutral (1) but `current_position` was long (2), a fee IS charged (`sig_t != current_position`), but the strategy return is `0.0`. This is correct only if the position was reversed. But for a custom strategy that enters neutral from long, the user expects to exit the position — yet the fee is applied but no P&L is locked in from the previous day's move.

**Why it blocks alpha:** Portfolio backtest returns will be wrong by one bar for every neutral exit, which compounds over 90 days.

**What you need to do:**
- Rewrite the equity loop to track `position: -1 | 0 | 1` (short, neutral, long) rather than `current_position` conflating entry cost with P&L.
- Charge fee on the transition, then apply the day's return to the position held at the START of the bar.

### 1.6 `backtest.rs`: `prediction_r2` is hardcoded to `0.0` for every backtest result
**What is missing:** `BacktestMetrics` has `prediction_r2: 0.0` initialized in `run_backtest_for_asset` line 808 and `backtest_portfolio` line 1105. It is never computed. The CSV/JSON reports export `0.0` unconditionally.

**Why it blocks alpha:** A metrics table showing `R² = 0.00` for every strategy, every asset, is misleading and undermines credibility.

**What you need to do:**
- Either implement `calculate_r2` for strategy returns vs buy-and-hold returns, or remove the column from the report schema and the struct until it's actually calculated.

---

## 2. High-Severity Issues (P1 — Must fix before wider alpha distribution)

### 2.1 `pipeline.rs` god-function: 770 LOC + 280 LOC duplicated backtest/report block
`run_pipeline_flow` and `run_standalone_backtest` each contain ~200 lines of copy-pasted logic for US Treasury benchmark calculation, CSV/JSON backtest report export, and table printing.

**What to do:**
- Extract `backtest::generate_backtest_report(metrics, run_dir)` that writes CSV + JSON + formatted table.
- Extract `backtest::append_treasury_benchmark(df, cols, ann_factor, metrics)` for the `^TNX` logic.
- Target: reduce `pipeline.rs` to <400 LOC.

### 2.2 No integration tests for the full `run_pipeline_flow` path against mocked API responses
**What is missing:** All tests in `tests/pipeline_tests.rs` build DataFrames by hand and call `analysis`, `optimization`, or `backtest` functions directly. No test ever calls `run_pipeline_flow` end-to-end.

**Why it matters:** A change in `cache.rs`, `api/coingecko.rs`, or `pipeline.rs` join logic could break the entire flow without any test catching it.

**What you need to do:**
- Add a mocked HTTP layer (`wiremock` or `mockito`) that returns canned JSON for CoinGecko + Yahoo.
- Call `run_pipeline_flow` with the mock server URL and assert that `data.csv`, `data.parquet`, `portfolio_weights.csv`, and at least one PNG are produced.
- This is the single most important test you can add for alpha stability.

### 2.3 `api/coingecko.rs`: `get_coins_list` is called on every `check_coin_id` — O(n) linear scan over the entire coin list
**What is missing:** `check_coin_id` fetches the full `/coins/list` endpoint (~3 MB, ~10,000 coins) every time the user runs `check-coin` or when the interactive menu validates input. No local cache or pagination.

**Why it matters:** On the free CoinGecko tier (rate limit ~10-30 req/min), this alone can trigger a 429 on repeated use. In the interactive menu, if the user checks 3 coins in a row, they will be rate-limited.

**What you need to do:**
- Cache the coins list in the SQLite `api_cache` table with a 24-hour TTL (it changes rarely).
- Or load it once at startup and pass it through.

### 2.4 `api/coingecko.rs`: retry logic only handles 429, not 5xx or network timeouts in `get_request`
**What is missing:** The retry loop has `max_attempts = 4` but the error path for `?` on `request_builder.send().await?` (network failure) does NOT increment `attempts` — it returns immediately. Only 429 is retried. 503, timeouts, and connection resets are fatal.

**What to do:**
- Wrap `send().await` in a match and apply the same exponential backoff for `reqwest::Error` (if it's a timeout/connect error) and for 5xx status codes.

### 2.5 `optimization.rs`: Monte Carlo uses 10,000 simulations with no early-stopping or convergence check
**What is missing:** `run_monte_carlo` always runs exactly `num_simulations` iterations. For the default of 10,000 this is fine, but there is no check that the frontier has converged.

**Why it matters:** For alpha, 10,000 is acceptable. But the struct `OptimizationResult` stores `simulated_points: Vec<(f64, f64, f64)>` which is ~240 KB for 10,000 points. If someone runs this with `--seed` multiple times and keeps the results in memory (e.g., the UI), memory grows unbounded.

**What to do:**
- Add a configurable max-simulation cap (e.g., 50,000) to prevent OOM if the API is extended.
- Document that each simulation stores 3 f64s (~24 bytes) so 10k = ~240 KB per run.

---

## 3. Medium-Severity Issues (P2 — Should fix before beta)

### 3.1 `analysis.rs`: `align_datasets` forces forward-fill on ALL non-date columns, including `_volume`
**What is missing:** Volume columns are forward-filled in the default path (`drop_weekends = false`). For crypto + stock alignment, a weekend stock volume forward-fill means the crypto volume column logic is correct (it was already present), but if a stock column like `^GSPC_volume` exists, it gets forward-filled too, which is misleading.

**What to do:**
- Either exclude `_volume` columns from forward-fill (fill with 0 like the current `drop_weekends` path does), or document that forward-filled volume is intentional for "last known traded volume" semantics.

### 3.2 `cache.rs`: SQLite cache key is the full URL including query params — cache hit rate is fragile
**What is missing:** URLs are cached exactly as built by `reqwest::Url::parse_with_params`. If timestamps shift by 1 millisecond due to client clock drift, the cache key changes even though the data is identical. The README mentions timestamp rounding to daily boundaries, but `pipeline.rs:126` does `(now / 86400) * 86400` only for `from_timestamp` / `to_timestamp`. The `market_chart/range` endpoint uses these rounded values, but `get_coin_ohlc` uses `days_str` (integer days), not timestamps, so its URL is stable. The cache works, but the TTL logic doesn't account for intraday data freshness.

**What to do:**
- For alpha, keep it simple. But add a log line when a cache miss occurs so users understand why a fresh API call was made.

### 3.3 No structured logging — only `println!` and `eprintln!`
**What is missing:** The tool uses `println!` for everything from progress updates to error diagnostics. There is no log level, no `--verbose` flag, no log file.

**Why it matters:** When an alpha user reports a bug, you cannot ask them for "the debug log." You must ask them to rerun with output redirected, which is friction.

**What to do:**
- Add `tracing` or `env_logger` with `--verbose` / `--quiet` CLI flags. At minimum, write a structured `cdg.log` file in the run directory.

### 3.4 `ui.rs`: interactive menu does not write to a log or save session state
**What is missing:** If the user runs the interactive menu, runs a pipeline, and the tool panics or is killed, there is no record of what they selected.

**What to do:**
- Log the `PipelineConfig` used for each run to `{run_dir}/config.json` so the run is reproducible.

### 3.5 `pipeline.rs`: hardcoded benchmark ticker list may fail silently
**What is missing:** The list `["^GSPC", "^DJI", "^IXIC", "^HSI", "^BVSP", "^TNX"]` is fetched unconditionally (if not lightweight). If Yahoo returns 404 for any ticker (regional restrictions, discontinued symbols), the error is printed with `pb.println` but execution continues. This means the aligned DataFrame may not contain `^TNX`, and later code checks `if final_df.column("^TNX").is_ok()` — but if `^GSPC` was the one missing, the user has no benchmark data at all.

**What to do:**
- Define benchmark tickers as a configurable constant or CLI flag.
- If a benchmark fetch fails, degrade gracefully: skip that ticker but ensure at least one benchmark is present if possible.

### 3.6 `export.rs`: test writes to `tests/temp_test.csv` and `tests/temp_test.parquet` without cleanup
**What is missing:** If the test is interrupted (Ctrl+C), the temp files remain. They are in `tests/` which is tracked by git in some projects.

**What to do:**
- Use `tempfile::NamedTempFile` instead of hardcoded paths.
- Or add a cleanup guard.

---

## 4. Low-Severity / Style Issues (P3 — Fix before beta or public launch)

### 4.1 `analysis.rs` and `backtest.rs` contain large functions with high cyclomatic complexity
- `compute_returns_and_indicators` (~120 LOC) does parsing, indicator computation, OHLC gating, and column assembly.
- `run_backtest_for_asset` (~240 LOC) does data loading, strategy evaluation, equity tracking, Sharpe/drawdown calculation, confusion matrix, and metrics assembly.
- **Recommendation:** Split into `fn load_price_series`, `fn evaluate_signal`, `fn track_equity`, `fn compute_metrics` to make each unit testable in isolation.

### 4.2 `pipeline.rs` footgun: `currency_cols[0]` assumed to exist in lightweight mode
**What is missing:** Line 352: `final_df = analysis::compute_returns_and_indicators(&final_df, &currency_cols[0])?;` — if `currency_cols` is empty (should be impossible, but not asserted), this panics.

**What to do:**
- Add `assert!(!currency_cols.is_empty())` after the fetch loop.

### 4.3 `backtest.rs`: `BacktestMetrics` uses `f64` for everything, no domain constraints
- `prediction_accuracy` can exceed `1.0` if the confusion matrix counts are wrong.
- `strategy_return` can be `NaN` if equity starts at `0.0` (it starts at `10000.0` so this is unlikely).
- **Recommendation:** Add `#[must_use]` and `#[derive(Debug)]` with debug assertions in tests.

### 4.4 `optimization.rs`: xorshift RNG has known weak bits in low positions
**What is missing:** The custom `Xorshift` implementation uses `x ^= x << 13; x ^= x >> 7; x ^= x << 17;` which is fine for non-cryptographic use, but `next_f64` maps `u64` to `f64` via `/ u64::MAX`, producing only ~53 bits of precision due to IEEE 754 double mantissa. For 10,000 portfolio simulations, this is adequate, but for alpha, consider using `rand::rngs::StdRng` with `rand::distributions::Uniform` for better statistical properties.

**What to do:**
- Document that the RNG is deterministic and not cryptographically secure.
- Consider adding `rand_xorshift` as an optional dependency and falling back to the custom impl only if `rand` is not desired.

### 4.5 `README.md` claims `--seed default (1337)` but `pipeline.rs` default is `None` which resolves to xorshift seed `1` inside `optimization.rs`
**What is missing:** The README says `default: 1337`. The CLI default is `None`. The optimization module treats `None` as "no seed" and uses `Xorshift::new(1)` because of the `if seed == 0 { 1 } else { seed }` guard. This is misleading.

**What to do:**
- Either change the README to say default is random, or change the default to `Some(1337)` in the CLI definition.

### 4.6 `Cargo.toml` profile settings suppress all debug info
```toml
[profile.dev]
codegen-units = 1
debug = false

[profile.release]
codegen-units = 1
debug = false
```
**Why it matters:** Alpha users and contributors will be unable to get useful backtraces from crash reports. Setting `debug = false` in dev makes `cargo test` output less useful and makes `RUST_BACKTRACE=1` produce shallow traces.

**What to do:**
- Set `debug = 1` in dev for alpha. Keep `debug = false` in release for binary size, but be aware this makes customer crash reports useless.

---

## 5. Alpha Release Checklist (concrete, actionable)

Before you tag `v0.1.0-alpha`, you MUST have:

| # | Item | Owner | Blocking? |
|---|------|-------|-----------|
| 1 | Golden-value tests for all 8 technical indicators (`analysis.rs`) | Math validation | YES |
| 2 | Deterministic equity-curve tests for RSI/MACD/Bollinger/custom strategy (`backtest.rs`) | Math validation | YES |
| 3 | Fix neutral-signal fee/P&L equity bug in `run_backtest_for_asset` | Bug fix | YES |
| 4 | Replace `std::process::exit(0)` with cancellation token on Ctrl+C | Reliability | YES |
| 5 | Fix null OHLC fill-to-zero producing garbage ATR/Stoch/ADX | Data quality | YES |
| 6 | Remove hardcoded `prediction_r2: 0.0` or implement it | Correctness | YES |
| 7 | Integration test with mocked HTTP responses for `run_pipeline_flow` | E2E stability | YES |
| 8 | Cache the `/coins/list` response for `check_coin_id` | Performance | YES |
| 9 | Add structured logging (`tracing` or `env_logger`) with `--verbose` | Debuggability | NO |
| 10 | Write `{run_dir}/config.json` with the `PipelineConfig` used | Reproducibility | NO |
| 11 | Fix README discrepancy on default Monte Carlo seed | Documentation | NO |
| 12 | Change `Cargo.toml` dev profile to `debug = 1` | Debuggability | NO |
| 13 | Use `tempfile` crate in `export.rs` tests | Hygiene | NO |
| 14 | Add `--version` and `--about` metadata review (check `clap` derive) | Polish | NO |
| 15 | Verify binary runs on clean Windows/macOS/Linux without `dotenv` dependency (document env vars instead) | Portability | NO |

---

## 6. Summary

**Current state:** The architecture is solid. Module boundaries are clean, async patterns are correct, and the CLI contract is well-designed. The core mathematical implementations (indicators, Sharpe, drawdown, Monte Carlo) are plausible but **entirely unvalidated against known-good reference values**.

**Alpha risk:** Not security or data-leakage risk. The risk is **correctness risk**: an alpha user runs `cargo run -- run-pipeline -c bitcoin -d 90`, gets numbers that look authoritative, and makes decisions based on them. If RSI is calculated with wrong smoothing, or the equity curve charges fees on the wrong bar, the tool is actively harmful to its primary use case.

**To reach alpha:** You need the 7 YES-blocking items above. Items 8-15 are quality-of-life improvements that separate "it runs" from "it is trustworthy."

**Estimated effort to alpha-ready:** 2-3 focused days of work if you write tests first (TDD) for the indicator and backtest math. The cancellation token and data-quality fixes are a few hours each. The mocked integration test is the largest single item (~half day).
