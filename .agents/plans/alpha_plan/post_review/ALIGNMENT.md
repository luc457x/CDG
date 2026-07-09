# Alpha Plan Alignment — Phase Status

**Date:** 2026-07-09  
**Method:** Static audit of `src/` + test execution (`cargo test -p cdg` — 73 unit + 20 integration tests pass)

| Phase | Status | Notes |
|-------|--------|-------|
| 01 | **IMPLEMENTED** | Panic, mean→last, first/last OHLC, malformed log, yahoo null continuity all present with tests |
| 02 | **PARTIAL** | Null propagation + `prep_ml` guard done. `data_quality_warnings: Vec<String>` field/test missing |
| 03 | **PARTIAL** | SMA/EMA/RSI/Bollinger/ATR/Stoch/OBV golden tests present. MACD & ADX golden tests missing. Upgraded `test_returns_and_indicators_computation` assertions (RSI bounded, Bollinger upper>lower, etc.) missing |
| 04 | **IMPLEMENTED** | Neutral bug fixed, `position: i32` loop, deterministic test present. Fee/return order differs from doc wording but semantics correct |
| 05 | **PARTIAL** | `calculate_r2` + `calculate_max_drawdown` fixed. `prediction_r2` computed (actual vs strategy, not vs B&H). Portfolio/treasury placeholders still hardcoded `0.0`/`"n/a"` |
| 06 | **IMPLEMENTED** | 10+ strategy + portfolio tests present. Coverage breadth good; some tests assert inequalities rather than exact equity values |
| 07 | **IMPLEMENTED** | `std::process::exit(0)` removed, `CancellationToken` propagated, clean `main` return |
| 08 | **IMPLEMENTED** | `wiremock` e2e tests cover happy path, 404 coin, missing ^TNX, cache hits |
| 09 | **PARTIAL** | `generate_backtest_report` + `append_treasury_benchmark` extracted. Success message still unconditional at `pipeline.rs:865`; `run_pipeline_flow` not <400 LOC |
| 10 | **PARTIAL** | `--drop-weekends` threaded to standalone. No `annualization_factor: Option<f64>` param in `run_standalone_backtest` |
| 11 | **IMPLEMENTED** | `/coins/list` cached 24h SQLite, reused by `check_coin_id` |
| 12 | **IMPLEMENTED** | 429/5xx/network retry with backoff, 30s timeout, max 4 attempts |
| 13 | **IMPLEMENTED** | `CoinResolution` enum, list lowercased once |
| 14 | **IMPLEMENTED** | `plots` toggle + env + gates |
| 15 | **IMPLEMENTED** | `optimize` toggle + env + gates |
| 16 | **IMPLEMENTED** | `warn_light_conflicts` + test |
| 17 | **IMPLEMENTED** | T1–T6 present |
| 18 | **IMPLEMENTED** | T12a/T12b present |
| 19 | **IMPLEMENTED** | `doc/api_cache.md`, cache miss logs |
| 20 | **IMPLEMENTED** | Candlestick PNG + ASCII stdout |
| 21 | **IMPLEMENTED** | `debug = 1` in dev profile |
| 22 | **IMPLEMENTED** | README seed matches CLI default `1337` |
| 23 | **IMPLEMENTED** | Internal methods private, negative TTL guard, concurrent `check_cache_hits`, 10MB body cap |
| 24 | **IMPLEMENTED** | `&DataFrame` signatures, `tempfile`, `validate_safe_path` |
| 25 | **IMPLEMENTED** | `currency_cols` empty guard |
| 26 | **IMPLEMENTED** | Unified covariance annualization, `max_simulations` cap, explicit RF rate, min-weight documented, progress bar fixed |
| 27 | **IMPLEMENTED** | `sanitize_name` + `validate_safe_path` wired before path building |
| 28 | **IMPLEMENTED** | Unconditional 3s sleep removed |
| 29 | **IMPLEMENTED** | Menu prompts concurrency + annualization_factor |

**Aggregate:** 25/29 fully implemented, 4 partial (02, 03, 05, 09, 10). 0 not implemented.
