# CDG — Alpha-Readiness Phased Plan

**Date:** 2026-07-07
**Primary focus:** `alpha_readiness_2026-07-05.md` (P0–P3 + checklist)
**Sources reconciled:** `base_plan.md` (P1–P12), `code_analysis_2026-07-04.md`, `line_by_line_review.md` (Phases 1–11)
**Method:** STDD. Spec/contracts first, red tests, minimal green, refactor. All `file:line` verified against current `dev`.

This plan reorganizes every finding into execution phases ordered by alpha-blocking severity. P0 = release blocker. Each phase lists: scope, source findings, concrete tasks (with `file:line`), and validation.

---

## Alpha-blocking gate (must all pass before `v0.1.0-alpha`)

### PHASE 0 — P0: Indicator math correctness (`analysis.rs`)
**Why blocks:** RSI/MACD/Bollinger/ATR/Stoch/ADX/OBV are unvalidated and the null-OHLC path is corrupting.

**Findings reconciled:**
- alpha 1.1 (zero golden-value tests), 1.4 (null OHLC → `0.0` corruption)
- base_plan P4 (math tests + annualization — math tests only here; annualization fix in Phase H4)
- line_by_line 4a (market_chart panic `total_volumes[i]`; OHLC aggregation `mean` vs first/last; yahoo null breaks date continuity), 4b/4c (indicators correct but untested), 4d (critical `unwrap_or(0.0)` on high/low/volume), 4e (weak existence-only tests)

**Tasks:**
1. **Golden-value tests** for `calculate_sma`, `calculate_ema`, `calculate_rsi`, `calculate_macd`, `calculate_bollinger_bands`, `calculate_atr`, `calculate_stochastic`, `calculate_adx`, `calculate_obv` (`analysis.rs`).
   - 3 vectors each: uptrend, downtrend, flat. Edge cases: `period > len` → all `None`; `close == open` boundary; `NaN`/out-of-range propagation.
   - Reference values via hand computation or `pandas_ta`/`ta` script. Target 15–20 new tests.
2. **Fix 1.4 / 4d corruption:** in `compute_returns_and_indicators` (~`analysis.rs:683-797`), if `high`/`low`/`volume` is `None` at index `i`, emit `None` for ATR/Stoch/ADX/OBV at `i` (do not `unwrap_or(0.0)`). Add `data_quality_warnings: Vec<String>` return + logger listing dates with missing OHLC.
3. **Fix 4a `parse_coingecko_market_chart`** panic: `total_volumes[i]` → `total_volumes.get(i).map(|v| v.1).unwrap_or(0.0)` (line ~323).
4. **Fix 4a `parse_coingecko_ohlc`** aggregation: open = first, close = last of day (not `mean`); log/warn on malformed `item.len() < 5` instead of silent drop; round timestamp (no truncation).
5. **Fix 4a `parse_yahoo_json`:** null price rows must not silently break date continuity — forward-fill or return explicit gap strategy; align with `align_datasets`.
6. **Fix 4d `prep_ml`:** guard `inf`/`nan` inputs; document constant-variance `std=1.0` fallback.

**Validation:** `cargo test -p cdg analysis::` all green; golden vectors match reference; smear test with one missing OHLC bar → ATR/Stoch/ADX/OBV `None` + warning, not garbage.

---

### PHASE 1 — P0: Backtest signal + equity correctness (`backtest.rs`)
**Why blocks:** Signal logic and equity math are the core value prop and are wrong/undefined.

**Findings reconciled:**
- alpha 1.2 (no signal/equity tests), 1.5 (neutral-signal fee/P&L bug), 1.6 (`prediction_r2` hardcoded `0.0`)
- base_plan P4 (Sharpe/drawdown tests)
- line_by_line 6b (`calculate_r2` constant-series returns 0.0 bug; `calculate_max_drawdown` `peak>0.0` guard), 6c (neutral-fee bug; portfolio `unwrap_or(0.0)`; division-by-zero on zero equity; test data decimal-vs-percent mismatch), 6d (placeholder metrics; weak assertions)

**Tasks:**
1. **Deterministic equity-curve tests** for `run_backtest_for_asset` feeding known price/indicator data:
   - RSI < 30 → buy, MACD cross, Bollinger touch, custom JSON config condition eval.
   - Exact equity value per bar; edge cases: price gaps, zero-volume day, all-`None` indicator cols, single-bar DataFrame.
   - `backtest_portfolio` weekly/monthly rebalance: exact fee deductions + weight drift between rebalances.
   - Target 10–15 new tests.
2. **Fix 1.5 / 6c neutral bug:** rewrite equity loop to track `position: -1|0|1`. Charge fee on transition; apply day's return to position held at bar START. Fixes one-bar P&L loss on neutral exits.
3. **Fix 1.6 / 6a:** `prediction_r2` — implement `calculate_r2` for strategy vs buy-&-hold returns, OR remove the field + report column until computed. Decide + apply consistently.
4. **Fix 6b `calculate_r2`:** `ss_tot == 0.0` → return `1.0` if `ss_res == 0.0` else `0.0` (perfect constant prediction). Add test.
5. **Fix 6b `calculate_max_drawdown`:** guard `peak != 0.0` not `peak > 0.0` (negative-peak equity). Add test.
6. **Fix 6c `backtest_portfolio`:** replace `unwrap_or(0.0)` null poisoning; guard `prev_e == 0.0` division-by-zero in Sharpe; align test data to percentage returns (matches `compute_returns_and_indicators`).
7. **Fix 6d placeholders:** drop meaningless `prediction_accuracy: 1.0` / `active_win_rate: 1.0` / `prediction_rating: "N/A"` from portfolio + treasury `BacktestMetrics` (or separate struct). Add `BacktestMetrics` doc on valid rating values.

**Validation:** `cargo test -p cdg backtest::` green; equity curves match hand-computed; treasury/portfolio reports contain no fake prediction fields.

---

### PHASE 2 — P0: Graceful shutdown (CancellationToken)
**Why blocks:** SIGTERM/Ctrl+C abandons tasks, leaves half-written CSVs + locked WAL DB.

**Findings reconciled:**
- alpha 1.3, checklist #4
- code_analysis #4, line_by_line 11 (`main.rs:154-161` `std::process::exit(0)`)

**Tasks:**
1. Add `tokio_util::sync::CancellationToken`. Propagate to all `JoinSet` tasks in `pipeline.rs` fetch loop and `run_ohlcv_flow`/`run_standalone_backtest`.
2. On cancel: stop spawning, `join_set.shutdown().await` drain, flush/close file handles, checkpoint SQLite, then return `Err` from `main()` (clean non-zero) or `Ok(())` if user-initiated. Remove `std::process::exit(0)`.
3. Replace `pipeline.rs:157-161` spawn.

**Validation:** `cargo run -- run-pipeline -c bitcoin -d 30`, Ctrl+C mid-fetch → no leftover half-written file, DB not locked, exit code reflects cancellation.

---

### PHASE 3 — P0: End-to-end integration test (mocked HTTP)
**Why blocks:** No test exercises `run_pipeline_flow` end-to-end; a cache/api/join change breaks flow silently.

**Findings reconciled:**
- alpha 2.2, checklist #7

**Tasks:**
1. Add `wiremock`/`mockito` HTTP layer returning canned CoinGecko + Yahoo JSON.
2. Call `run_pipeline_flow` with mock server URL; assert `data.csv`, `data.parquet`, `portfolio_weights.csv`, and ≥1 PNG produced.
3. Cover happy path + one 404 coin (assert graceful skip) + missing `^TNX` (assert benchmark degradation).

**Validation:** `cargo test --test pipeline_tests` integration case green; outputs present.

---

## P1 — high severity (before wider alpha distribution)

### PHASE 4 — P1: Pipeline god-function refactor + shared report
**Findings:** alpha 2.1, code_analysis #1, line_by_line 10c (copy-paste backtest/report block; unstable "success" message; HashMap overwrite; ignored JSON write).

**Tasks:**
1. Extract `backtest::generate_backtest_report(metrics, run_dir)` → writes CSV + JSON + formatted table (single source, used by `run_pipeline_flow` + `run_standalone_backtest`).
2. Extract `backtest::append_treasury_benchmark(df, cols, ann_factor, metrics)`.
3. Fix `pipeline.rs:714-767`: HashMap key collision warning; surface JSON write failure; final message reflects actual status (not unconditional success).
4. Reduce `pipeline.rs` to <400 LOC.

**Validation:** `cargo build`; both call sites produce identical report bytes (golden compare).

---

### PHASE 5 — P1: API client reliability + caching
**Findings:** alpha 2.3 (coins/list cache), 2.4 (retry 5xx/timeout), code_analysis #5/#6, line_by_line 8a (`get_coins_list` 5-min TTL; `check_coin_id` refetch every call; retry only 429; no timeout; O(n*m) substring; inverted `Ok(None)` semantics), 8b (yahoo better retry but no jitter/timeout; no percent-encoding in URL).

**Tasks:**
1. Cache `/coins/list` in SQLite `api_cache` with 24h TTL (or in-memory for client lifetime). `check_coin_id` reuses cache instead of refetching.
2. `get_request` (coingecko): retry 429, 5xx, and `reqwest::Error` (timeout/connect) with exponential backoff + `Retry-After` honor; add client `timeout`.
3. yahoo: add jitter to backoff; `reqwest::Url` builder with percent-encoding for ticker; add timeout.
4. Improve `check_coin_id` return type (`Exact`/`Ambiguous`/`NotFound`) — non-breaking if callers updated in `main.rs` + `ui.rs`.
5. Pre-lowercase coin list once (avoid per-iteration alloc).

**Validation:** unit tests for retry/backoff; cache-hit test for repeated `check_coin_id`; no 429 under 3 sequential checks.

---

### PHASE 6 — P1: Optimization hardening
**Findings:** alpha 2.5 (MC no convergence cap), line_by_line 5 (`^TNX` silent 0.0 risk-free; `+0.01` min-weight math unclear; progress bar not real-time; weak tests).

**Tasks:**
1. Add configurable `max_simulations` cap (default 10,000, hard cap 50,000). Document per-sim memory (~24 B × 10k = 240 KB).
2. Make risk-free rate explicit param (default `^TNX`, fallback documented + logged) instead of silent `0.0`.
3. Replace custom `Xorshift` min-weight hack with documented Dirichlet/reject-resample OR document the bias; fix progress bar update under Rayon.
4. Add optimization formula tests (weights sum to 1, min-vol ≤ max-sharpe vol, annualization factor).

**Validation:** `cargo test optimization::`; cap respected; determinism with `--seed`.

---

## base_plan P1–P12 (feature/contract work, layer under alpha gate)

### PHASE H1 — P1 `--light` conflict warning (`base_plan` P1)
- Add `warn_light_conflicts(config)` in `pipeline.rs`; print via `eprintln!` after `--light` override, before timestamp calc. Detect multi-coin + `days != 30`. Fix misleading `main.rs:52` docstring. Unit test helper.

### PHASE H2 — P2 cache boundary (verify + doc) (`base_plan` P2)
- Test: two `now` same UTC day → identical `from`/`to` params. Document boundary rounding vs TTL in `doc/api_cache.md`. No rounding change.

### PHASE H3 — P3 weekend alignment tests (`base_plan` P3)
- T1–T6 in `analysis.rs`: `drop_weekends` true/false, `_volume` Zero-fill, crypto+stock join, unparseable date mask, `parse_coingecko_market_chart` weekend rows.

### PHASE H4 — P4 annualization fix (`base_plan` P4)
- `run_standalone_backtest` hardcodes `365.0` at `pipeline.rs:1016`; add `annualization_factor: Option<f64>`/`drop_weekends` param; compute `ann` like `RunPipeline` path; add `--drop-weekends` to `Backtest` subcommand; thread through `main.rs`.

### PHASE H5 — P5 optimization toggle (`base_plan` P5)
- `optimize: bool` in `PipelineConfig` (default true); `--no-optimize` + `CDG_OPTIMIZE`; gate `pipeline.rs:426` as `config.optimize && currency_cols.len() >= 2`; skip message.

### PHASE H6 — P7 plots toggle + tests (`base_plan` P7)
- `plots: bool` (default true); `--no-plots` + `CDG_PLOTS`; gate `pipeline.rs:376` → `config.plots && !light`; thread into `run_standalone_backtest`. `plot.rs` tests T1–T8 rendering to `temp_dir()`, assert PNG signature `[137,80,78,71,13,10,26,10]`.

### PHASE H7 — P11 candlestick PNG + stdout ASCII (`base_plan` P11)
- `plot_candlestick(...)` via plotters `Candlestick` (rise green/fall red). `print_candlestick_stdout(df, col, max_width)` downsample + ANSI. Add `candle_stdout` to `PipelineConfig`; per-coin `{col}_candlestick.png` + optional stdout. `--candle-stdout` in `RunPipeline`. Tests: valid PNG; non-empty glyphs; no panic single-row/flat; missing OHLC → `Err`.

### PHASE H8 — P12 weekend fill semantics (tests only) (`base_plan` P12)
- T12a/T12b: crypto LEFT JOIN stock-like (Fri close=100/vol=1000, null Sat/Sun) → assert Sat/Sun price==100 AND volume==0; `{ticker}_volume` Zero branch (P9/B1 dependency note).

**H-phase validation:** `cargo test` + `cargo clippy` clean; manual checks from `base_plan` Validation section (P1 warnings, P4 252 vs 365, P5/P7 toggles, P11 PNG+ASCII).

---

## P2 — medium (before beta)

### PHASE 7 — Structured logging + reproducibility
**Findings:** alpha 3.3 (no logging), 3.4 (UI no session state), 3.2 (cache miss log), 3.5 (benchmark fail silent), checklist #9/#10.
- Add `tracing`/`env_logger` with `--verbose`/`--quiet`; write `cdg.log` in run dir.
- Log `PipelineConfig` to `{run_dir}/config.json` (reproducible runs).
- Benchmark tickers configurable (const or CLI flag); degrade gracefully, ensure ≥1 benchmark present; log cache misses.

### PHASE 8 — Data-quality & alignment hardening (line_by_line leftovers)
- `align_datasets` (`analysis.rs:292-350`): remove unnecessary `.clone()`; vectorize `drop_weekends` date parse (`str.strptime`+`dt.weekday`); fix `pipeline.rs:309/277` hardcoded `drop_weekends=false` inconsistency (use config).
- `parse_coingecko_tickers` / `calculate_orderbook_metrics`: propagate nulls instead of `unwrap_or(0.0)`.

---

## P3 — low / style (before public launch)

### PHASE 9 — Hygiene & docs
**Findings:** alpha 4.1 (large fn complexity → split `compute_returns_and_indicators`, `run_backtest_for_asset`), 4.2 (`currency_cols[0]` assert), 4.3 (`BacktestMetrics` domain constraints/`#[must_use]`), 4.4 (RNG doc), 4.5 (README seed 1337 vs None→1), 4.6 (`Cargo.toml debug=false`), checklist #11–#15; code_analysis #2/#3/#7/#8.
- `Cargo.toml` `[profile.dev] debug = 1`.
- Fix README seed discrepancy (default `Some(1337)` or doc "random").
- `export.rs` + `cache.rs` + `backtest.rs` tests → `tempfile::TempDir`/RAII (no `tests/` pollution).
- `sanitize_name` whitelist (`[a-z0-9_-]`), length guard, `Result` return; `validate_safe_path` reject empty.
- `export.rs` drop `&mut DataFrame`; atomic write (temp+rename); path validation.
- `ui.rs` clear via ANSI; extract triplicated API-key detection; validate numeric inputs; fix "suggestions = Error" UX; `PipelineConfig` owns `String` (drop `'a`).
- `main.rs` extract `init_clients` + `resolve_paths`; remove duplicated test logic.
- `plot.rs` `get_distinct_color` non-black single series; fix `y_max==y_min` padding; test `get_distinct_color`/`hsl_to_rgb`/`get_log_return_stats`.
- `cache.rs` make `get_internal`/`insert_internal` private; guard negative TTL; concurrent `check_cache_hits`.

---

## Execution order (gating)
```
PHASE 0 → PHASE 1 → PHASE 2 → PHASE 3   (P0 gate: tag v0.1.0-alpha)
PHASE 4 → PHASE 5 → PHASE 6             (P1)
PHASE H1..H8                            (base_plan P1–P12; can run parallel to P1 after P0)
PHASE 7 → PHASE 8                       (P2)
PHASE 9                                 (P3)
```

## Cross-cutting rules
- Every math/bug fix ships with a red test first (STDD).
- No `unwrap_or(0.0)` on financial nulls — propagate `None` or error + warn.
- No `std::process::exit` while async tasks live.
- Keep daily cache pin (P2 decision); finer granularity → Backlog.
- Deferred (not this plan): base_plan P8 (`--light` generalist refactor), P9 (unified multi-source ingestion B1–B3), P10 (ML ADR).
