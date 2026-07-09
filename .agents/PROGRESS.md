# Progress (PROGRESS.md)

## Status

- State: Implementing Alpha plan; Phases 01–07 implemented.
- Last: Replaced std::process::exit(0) handler with CancellationToken-driven graceful shutdown on Ctrl+C.

## Log

Old sessions: [PROGRESS_ARCHIVE.md](./PROGRESS_ARCHIVE.md).

### Session 39: Implement Graceful Ctrl+C Shutdown

- Date: 2026-07-08
- Agent: Antigravity
- Goal: Implement alpha phase 07 — replace Ctrl+C std::process::exit with CancellationToken graceful shutdown.
- Constraints: None.
- Done:
  - **Phase 07**: Added `tokio-util` dependency in [Cargo.toml](file:///C:/Users/lucas/Code/CDG/Cargo.toml) to support `CancellationToken`.
  - Registered Ctrl+C listener in [pipeline.rs](file:///C:/Users/lucas/Code/CDG/src/pipeline.rs) to trigger cancellation token on Ctrl+C during active runs.
  - Implemented `AbortOnDrop` task guard in [pipeline.rs](file:///C:/Users/lucas/Code/CDG/src/pipeline.rs) to prevent listener leaks.
  - Checked `cancel_token.is_cancelled()` inside spawned JoinSet tasks and sequential loops in `run_pipeline_flow` and `run_standalone_backtest` to exit early when cancelled.
  - Used `tokio::select!` in the `JoinSet` polling loop of `run_pipeline_flow` to immediately detect cancellation and shutdown/abort remaining tasks.
  - Removed global `std::process::exit` Ctrl+C spawn from [main.rs](file:///C:/Users/lucas/Code/CDG/src/main.rs), wrapping the command execution inside an `async` block to cleanly return and handle cancellation errors.
- Blocked: None.
- Risk: None.
- Artifact: [Cargo.toml](file:///C:/Users/lucas/Code/CDG/Cargo.toml), [src/pipeline.rs](file:///C:/Users/lucas/Code/CDG/src/pipeline.rs), [src/main.rs](file:///C:/Users/lucas/Code/CDG/src/main.rs).
- Verification: `cargo test` runs cleanly (83 passed).
- Pending: Phase 08 (pipeline E2E test).

### Session 38: Fix Backtest Engine + Add Tests

- Date: 2026-07-08
- Agent: Antigravity
- Goal: Implement alpha phases 04, 05, 06 — fix neutral-exit fee/P&L bug, fix metrics math, add deterministic equity/portfolio tests.
- Constraints: None.
- Done:
  - **Phase 04**: Rewrote equity loop in [backtest.rs:597-738](file:///C:/Users/lucas/Code/CDG/src/backtest.rs#L597-L738) to use explicit `prev_position: i32` (-1/0/1). Return now applied with `prev_position` sign before fee; fee charged on transition. Fixes one-bar lag on neutral exits.
  - **Phase 05**: Fixed `calculate_r2` constant-series guard (ss_tot=0 → 1.0 if perfect, else 0.0) at [backtest.rs:292](file:///C:/Users/lucas/Code/CDG/src/backtest.rs#L292-L294). Fixed `calculate_max_drawdown` negative-peak guard (`!= 0.0`) at [backtest.rs:327](file:///C:/Users/lucas/Code/CDG/src/backtest.rs#L327). Computed `prediction_r2` from strategy vs actual returns (no more unconditional `0.0`). Fixed portfolio/treasury placeholder `prediction_accuracy/active_win_rate: 1.0` → `0.0` in [backtest.rs](file:///C:/Users/lucas/Code/CDG/src/backtest.rs) and [pipeline.rs](file:///C:/Users/lucas/Code/CDG/src/pipeline.rs).
  - **Phase 06**: Added 13 new `#[test]` functions covering: neutral-exit exact equity, no-fee-when-unchanged, R² constant-series perfect/wrong, max-drawdown negative peak, RSI flat/long exact values, single-bar error, MACD exact, Bollinger below-band, custom 2-rule JSON, all-None indicator flat, portfolio weekly-vs-monthly fees, weight renormalization, mismatched/empty assets errors.
- Blocked: None.
- Risk: prev_position semantic change alters existing strategy return magnitudes (expected — bug fix).
- Artifact: [backtest.rs](file:///C:/Users/lucas/Code/CDG/src/backtest.rs), [pipeline.rs](file:///C:/Users/lucas/Code/CDG/src/pipeline.rs).
- Verification: `cargo test backtest::` — 27 tests passed, 0 failed.
- Pending: Phase 07 (exit→cancellation), Phase 08 (pipeline E2E test).

### Session 37: Fix Parsers and Indicators

- Date: 2026-07-08
- Agent: Antigravity
- Goal: Fix aggregation & panic in CoinGecko/Yahoo parsers; propagate nulls/NaNs in indicators; add golden-value tests.
- Constraints: None.
- Done:
  - Fixed market chart OOB panic when prices outnumber volumes in [analysis.rs:27-32](file:///C:/Users/lucas/Code/CDG/src/analysis.rs#L27-L32).
  - Modified [analysis.rs](file:///C:/Users/lucas/Code/CDG/src/analysis.rs) to use `last` close of day and `mean` volume in market chart aggregation, and `first` open / `last` close in OHLC aggregation.
  - Added warning log skipped malformed OHLC rows in [analysis.rs:73-75](file:///C:/Users/lucas/Code/CDG/src/analysis.rs#L73-L75).
  - Preserved date continuity on null Yahoo prices in [analysis.rs:271-282](file:///C:/Users/lucas/Code/CDG/src/analysis.rs#L271-L282).
  - Gated advanced indicators (ATR, Stochastic, ADX, OBV) against missing OHLC/volume nulls, propagating `None`/`NaN` correctly and printing warnings to stderr in [analysis.rs:728-764](file:///C:/Users/lucas/Code/CDG/src/analysis.rs#L728-L764).
  - Filtered out `NaN`/`infinite` inputs in `prep_ml` in [analysis.rs:787-790](file:///C:/Users/lucas/Code/CDG/src/analysis.rs#L787-790).
  - Added 15 new test functions covering parser bug fixes, null propagation, indicator edge cases, zero variance, and golden value checks in [analysis.rs:1427-1744](file:///C:/Users/lucas/Code/CDG/src/analysis.rs#L1427-L1744).
- Blocked: None.
- Risk: None.
- Artifact: [analysis.rs](file:///C:/Users/lucas/Code/CDG/src/analysis.rs).
- Verification: `cargo test` — 66 tests passed.
- Pending: None.

### Session 36: Align Alpha Plan with code_review.md

- Date: 2026-07-08
- Agent: Antigravity
- Goal: Add the 4 uncovered `code_review.md` items as new alpha phases and fix alignment-audit inconsistencies in `.agents/etc/alpha_plan/`.
- Constraints: Plan claimed `.agents/etc/alpha_plan/` was write-protected; actual environment granted write access, so no workaround needed.
- Done:
  - Created `26.fix_optimization_covariance.md` (Critical covariance annualization + Monte Carlo hardening, source `code_review.md` #2).
  - Created `27.fix_path_injection.md` (wire existing `sanitize_name`/`validate_safe_path`, source #3).
  - Created `28.fix_coingecko_sleep.md` (remove blanket 3s cache-miss sleep, source #4).
  - Created `29.fix_menu_concurrency_annualization.md` (thread menu options, source #6).
  - `README.md`: added `code_analysis_2026-07-04.md` + `code_review.md` to Sources; appended `26 → 27 → 28 → 29` execution order; bumped pre-alpha polish range `09–25` → `09–29` with Critical note.
  - `25.add_pipeline_currency_assert.md`: added parallel `currency_dfs[0]` (`main.rs:495`) safety-rail note.
  - `19.add_cache_boundary_doc.md`: added same-UTC-day cache-key unit test task + validation.
  - `10.fix_annualization_inconsistency.md`: added out-of-scope Note for `calculate_sharpe` hardcoded `365.0` (`code_review.md` #7).
  - `backlog.md`: parked PHASE 8 alignment hardening (`align_datasets` clone, `drop_weekends` vs config, `parse_coingecko_tickers` `unwrap_or(0.0)`).
- Blocked: None.
- Risk: Plan's stated write-protection did not apply; doc edits only, low regression risk.
- Artifact: `.agents/etc/alpha_plan/26.fix_optimization_covariance.md`, `27.fix_path_injection.md`, `28.fix_coingecko_sleep.md`, `29.fix_menu_concurrency_annualization.md`, `README.md`, `25.add_pipeline_currency_assert.md`, `19.add_cache_boundary_doc.md`, `10.fix_annualization_inconsistency.md`, `backlog.md`.
- Verification: Confirmed all 4 new phase files exist on disk; re-read edited files to confirm content applied.
- Pending: PHASE 7 (structured logging) and PHASE 8 (alignment hardening) remain deferred/backlogged; code_review items #1 (`main.rs:495` panic) and #5 (`analysis.rs` duplicated indicators) left to backlog per plan.
