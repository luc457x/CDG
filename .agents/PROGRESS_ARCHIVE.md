# Progress Log History (PROGRESS_ARCHIVE.md)

## Archived Sessions

### Session 44: Complete Alpha Phases 19-25

- Date: 2026-07-09
- Agent: Antigravity
- Goal: Implement alpha phases 19 to 25.
- Constraints: None.
- Done:
  - **Phase 19**: Documented daily boundary rounding, canonical URL cache key, eviction rule, and endpoint TTLs in [doc/api_cache.md](file:///C:/Users/lucas/Code/CDG/doc/api_cache.md) and linked from [README.md](file:///C:/Users/lucas/Code/CDG/README.md). Logged cache misses in [src/api/coingecko.rs](file:///C:/Users/lucas/Code/CDG/src/api/coingecko.rs). Added test `test_coingecko_market_chart_range_same_day_alignment` in [tests/api_tests.rs](file:///C:/Users/lucas/Code/CDG/tests/api_tests.rs).
  - **Phase 20**: Added `plot_candlestick` (PNG) and `print_candlestick_stdout` (ASCII) in [src/plot.rs](file:///C:/Users/lucas/Code/CDG/src/plot.rs). Added `--candle-stdout` flag to `RunPipeline` and `Ohlcv` commands in [src/main.rs](file:///C:/Users/lucas/Code/CDG/src/main.rs), wired through [src/pipeline.rs](file:///C:/Users/lucas/Code/CDG/src/pipeline.rs) and [src/ui.rs](file:///C:/Users/lucas/Code/CDG/src/ui.rs). Added tests `test_plot_candlestick_and_stdout` and `test_print_candlestick_stdout_flat_and_single_row` in [tests/pipeline_tests.rs](file:///C:/Users/lucas/Code/CDG/tests/pipeline_tests.rs).
  - **Phase 21**: Added comment and set `debug = 1` for dev profile in [Cargo.toml](file:///C:/Users/lucas/Code/CDG/Cargo.toml) for backtraces.
  - **Phase 22**: Updated seed default documentation to `1337` in [README.md](file:///C:/Users/lucas/Code/CDG/README.md).
  - **Phase 23**: Hardened [src/cache.rs](file:///C:/Users/lucas/Code/CDG/src/cache.rs): internal helper methods made private, negative TTL guard, 10MB response limit, concurrent hits checking via `futures::future::join_all`. Added `futures = "0.3"` to [Cargo.toml](file:///C:/Users/lucas/Code/CDG/Cargo.toml). Added test `test_cache_negative_ttl` and `test_cache_max_body_bytes` in `src/cache.rs`.
  - **Phase 24**: Changed signatures to `&DataFrame` in [src/export.rs](file:///C:/Users/lucas/Code/CDG/src/export.rs) and callers in [src/pipeline.rs](file:///C:/Users/lucas/Code/CDG/src/pipeline.rs). Added path safety validation using `utils::validate_safe_path`. Added anyhow context to parent directory creation. Isolated tests using `tempfile::tempdir`.
  - **Phase 25**: Implemented empty dataset safety rails in [src/pipeline.rs](file:///C:/Users/lucas/Code/CDG/src/pipeline.rs) to return clean errors on fetch failures instead of out-of-bounds panics. Added test `test_pipeline_flow_all_coins_fail` in [tests/pipeline_tests.rs](file:///C:/Users/lucas/Code/CDG/tests/pipeline_tests.rs).
- Blocked: None.
- Risk: None.
- Artifact: [src/cache.rs](file:///C:/Users/lucas/Code/CDG/src/cache.rs), [src/export.rs](file:///C:/Users/lucas/Code/CDG/src/export.rs), [src/pipeline.rs](file:///C:/Users/lucas/Code/CDG/src/pipeline.rs), [src/plot.rs](file:///C:/Users/lucas/Code/CDG/src/plot.rs), [src/main.rs](file:///C:/Users/lucas/Code/CDG/src/main.rs), [src/ui.rs](file:///C:/Users/lucas/Code/CDG/src/ui.rs), [Cargo.toml](file:///C:/Users/lucas/Code/CDG/Cargo.toml), [README.md](file:///C:/Users/lucas/Code/CDG/README.md), [doc/api_cache.md](file:///C:/Users/lucas/Code/CDG/doc/api_cache.md), [tests/api_tests.rs](file:///C:/Users/lucas/Code/CDG/tests/api_tests.rs), [tests/pipeline_tests.rs](file:///C:/Users/lucas/Code/CDG/tests/pipeline_tests.rs).
- Verification: `cargo test` — 108 tests passed.
- Pending: None.

### Session 43: Implement --light Warnings & Weekend Tests

- Date: 2026-07-09
- Agent: Antigravity
- Goal: Implement alpha phases 16-18.
- Constraints: None.
- Done:
  - **Phase 16**: Added `pub fn warn_light_conflicts` in [src/pipeline.rs](file:///C:/Users/lucas/Code/CDG/src/pipeline.rs) to check `--light` conflicts and print warnings via `eprintln!` in `run_pipeline_flow`. Removed misleading "forces coin=bitcoin" from `--light` docstring in [src/main.rs](file:///C:/Users/lucas/Code/CDG/src/main.rs). Added `test_warn_light_conflicts` in [tests/pipeline_tests.rs](file:///C:/Users/lucas/Code/CDG/tests/pipeline_tests.rs).
  - **Phase 17**: Added tests `test_weekend_alignment_t1` through `test_weekend_alignment_t6` in [src/analysis.rs](file:///C:/Users/lucas/Code/CDG/src/analysis.rs) validating weekend alignment and drop logic.
  - **Phase 18**: Added tests `test_weekend_fill_t12a` and `test_weekend_fill_t12b` in [src/analysis.rs](file:///C:/Users/lucas/Code/CDG/src/analysis.rs) validating weekend volume zero-fill.
- Blocked: None.
- Risk: None.
- Artifact: [src/pipeline.rs](file:///C:/Users/lucas/Code/CDG/src/pipeline.rs), [src/main.rs](file:///C:/Users/lucas/Code/CDG/src/main.rs), [src/analysis.rs](file:///C:/Users/lucas/Code/CDG/src/analysis.rs), [tests/pipeline_tests.rs](file:///C:/Users/lucas/Code/CDG/tests/pipeline_tests.rs).
- Verification: `cargo test` - 102 passed.
- Pending: None.


- Date: 2026-07-09
- Agent: Antigravity
- Goal: Implement alpha phases 13–15.
- Constraints: None.
- Done:
  - **Phase 13**: Added `CoinResolution` enum (`Exact`, `Ambiguous`, `NotFound`) to [src/api/coingecko.rs](file:///C:/Users/lucas/Code/CDG/src/api/coingecko.rs). Lowercased the entire `/coins/list` response fields once upon parsing in `get_coins_list` to avoid per-iteration allocations. Refactored `check_coin_id` to return `CoinResolution` and use direct lowercase comparisons. Updated subcommand `check-coin` in [src/main.rs](file:///C:/Users/lucas/Code/CDG/src/main.rs) and interactive menu in [src/ui.rs](file:///C:/Users/lucas/Code/CDG/src/ui.rs) to handle the new enum. Wrote `test_check_coin_exact`, `test_check_coin_ambiguous`, and `test_check_coin_not_found` in [tests/api_tests.rs](file:///C:/Users/lucas/Code/CDG/tests/api_tests.rs).
  - **Phase 14**: Added `plots: bool` field to `PipelineConfig` struct in [src/pipeline.rs](file:///C:/Users/lucas/Code/CDG/src/pipeline.rs) and CLI arguments (`--plots` / `--no-plots` / `CDG_PLOTS`) to subcommands `RunPipeline` and `Backtest` in [src/main.rs](file:///C:/Users/lucas/Code/CDG/src/main.rs). Gated all plotting blocks in `pipeline.rs` (main plotting, efficient frontier, and standalone/portfolio backtests) by `config.plots`. Wrote `test_pipeline_flow_no_plots` integration test in [tests/pipeline_tests.rs](file:///C:/Users/lucas/Code/CDG/tests/pipeline_tests.rs) verifying no PNG generation when disabled.
  - **Phase 15**: Added `optimize: bool` field to `PipelineConfig` and CLI arguments (`--optimize` / `--no-optimize` / `CDG_OPTIMIZE`) to `RunPipeline`. Gated Markowitz portfolio optimization block in `pipeline.rs` by `config.optimize`. Wrote `test_pipeline_flow_no_optimize` integration test in [tests/pipeline_tests.rs](file:///C:/Users/lucas/Code/CDG/tests/pipeline_tests.rs) verifying no optimization outputs when disabled.
- Blocked: None.
- Risk: None.
- Artifact: [src/api/coingecko.rs](file:///C:/Users/lucas/Code/CDG/src/api/coingecko.rs), [src/pipeline.rs](file:///C:/Users/lucas/Code/CDG/src/pipeline.rs), [src/main.rs](file:///C:/Users/lucas/Code/CDG/src/main.rs), [src/ui.rs](file:///C:/Users/lucas/Code/CDG/src/ui.rs), [tests/api_tests.rs](file:///C:/Users/lucas/Code/CDG/tests/api_tests.rs), [tests/pipeline_tests.rs](file:///C:/Users/lucas/Code/CDG/tests/pipeline_tests.rs).
- Verification: `cargo test` — 93 passed (7.05s).
- Pending: None.

### Session 41: Extract Report, Fix Annualization, Cache Coins, Retry Logic

- Date: 2026-07-09
- Agent: Antigravity
- Goal: Implement alpha phases 09–12.
- Constraints: None.
- Done:
  - **Phase 09**: Added `generate_backtest_report` and `append_treasury_benchmark` to [src/backtest.rs](file:///C:/Users/lucas/Code/CDG/src/backtest.rs). Replaced ~150 LOC of duplicated treasury + CSV/JSON report logic in both `run_pipeline_flow` and `run_standalone_backtest` in [src/pipeline.rs](file:///C:/Users/lucas/Code/CDG/src/pipeline.rs) with calls to shared functions. `generate_backtest_report` surfaces key collision warning and returns Err on JSON write failure.
  - **Phase 10**: Added `drop_weekends: bool` parameter to `run_standalone_backtest` in [src/pipeline.rs](file:///C:/Users/lucas/Code/CDG/src/pipeline.rs). Computes `ann_factor` from it (252 if weekends dropped, 365 otherwise). Added `--drop-weekends` flag to `Backtest` subcommand in [src/main.rs](file:///C:/Users/lucas/Code/CDG/src/main.rs) and threaded it through.
  - **Phase 11**: Replaced `get_coins_list` in [src/api/coingecko.rs](file:///C:/Users/lucas/Code/CDG/src/api/coingecko.rs) with a 24h TTL cache implementation (manual cache check/insert using `COINS_LIST_TTL=86400`), avoiding re-fetching ~3MB on each `check-coin` call. Added test `test_coins_list_cache_hit_avoids_second_request` verifying exactly 1 HTTP request on two consecutive calls.
  - **Phase 12**: Added 30s request timeout to `reqwest::Client` builder. Expanded `get_request` retry logic in [src/api/coingecko.rs](file:///C:/Users/lucas/Code/CDG/src/api/coingecko.rs) to cover `is_server_error()` (5xx) and network errors (`is_timeout`, `is_connect`, `is_request`) using same exponential backoff. Added `with_retry_delay_ms` builder for test configurability. Added test `test_coingecko_retries_on_503` verifying 503→503→200 succeeds. Added `tempfile = "3"` to [Cargo.toml](file:///C:/Users/lucas/Code/CDG/Cargo.toml) dev-dependencies.
- Blocked: None.
- Risk: `append_treasury_benchmark` now always uses `ann_factor` param (standalone uses 365 or 252); previously standalone hardcoded 365. Behavior change intentional per spec.
- Artifact: [src/backtest.rs](file:///C:/Users/lucas/Code/CDG/src/backtest.rs), [src/pipeline.rs](file:///C:/Users/lucas/Code/CDG/src/pipeline.rs), [src/main.rs](file:///C:/Users/lucas/Code/CDG/src/main.rs), [src/api/coingecko.rs](file:///C:/Users/lucas/Code/CDG/src/api/coingecko.rs), [Cargo.toml](file:///C:/Users/lucas/Code/CDG/Cargo.toml), [tests/pipeline_tests.rs](file:///C:/Users/lucas/Code/CDG/tests/pipeline_tests.rs).
- Verification: `cargo test` — 89 tests passed (61 unit + 6 main + 9 api + 13 pipeline).
- Pending: Phase 13+.

### Session 40: Add Pipeline Integration Test

- Date: 2026-07-09
- Agent: Antigravity
- Goal: Implement alpha phase 08 — add mocked HTTP integration test for run_pipeline_flow.
- Constraints: None.
- Done:
  - Added `wiremock` dev-dependency to [Cargo.toml](file:///C:/Users/lucas/Code/CDG/Cargo.toml).
  - Added `coingecko_base_url` and `yahoo_base_url` to `PipelineConfig` in [src/pipeline.rs](file:///C:/Users/lucas/Code/CDG/src/pipeline.rs), and initialized them to `None` in [src/main.rs](file:///C:/Users/lucas/Code/CDG/src/main.rs) and [src/ui.rs](file:///C:/Users/lucas/Code/CDG/src/ui.rs).
  - Implemented graceful coin skip with warnings on fetch/parse errors in the `JoinSet` task spawn inside `run_pipeline_flow` in [src/pipeline.rs](file:///C:/Users/lucas/Code/CDG/src/pipeline.rs).
  - Appended 4 new integration tests to [tests/pipeline_tests.rs](file:///C:/Users/lucas/Code/CDG/tests/pipeline_tests.rs): `test_pipeline_flow_e2e_normal` (normal HTTP mock flow), `test_pipeline_flow_e2e_coin_404` (testing 404 response skip), `test_pipeline_flow_e2e_missing_tnx` (benchmarks fetch degradation when `^TNX` missing), and `test_pipeline_flow_e2e_cache_hits` (asserting cache hits database records and zero external requests on second run).
- Blocked: None.
- Risk: None.
- Artifact: [Cargo.toml](file:///C:/Users/lucas/Code/CDG/Cargo.toml), [src/pipeline.rs](file:///C:/Users/lucas/Code/CDG/src/pipeline.rs), [src/main.rs](file:///C:/Users/lucas/Code/CDG/src/main.rs), [src/ui.rs](file:///C:/Users/lucas/Code/CDG/src/ui.rs), [tests/pipeline_tests.rs](file:///C:/Users/lucas/Code/CDG/tests/pipeline_tests.rs).
- Verification: `cargo test` — 87 tests passed.
- Pending: None.

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
 - Goal: Add the 4 uncovered `code_review.md` items as new alpha phases and fix alignment-audit inconsistencies in `.agents/plans/alpha_plan/`.
 - Constraints: Plan claimed `.agents/plans/alpha_plan/` was write-protected; actual environment granted write access, so no workaround needed.
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
 - Artifact: `.agents/plans/alpha_plan/26.fix_optimization_covariance.md`, `27.fix_path_injection.md`, `28.fix_coingecko_sleep.md`, `29.fix_menu_concurrency_annualization.md`, `README.md`, `25.add_pipeline_currency_assert.md`, `19.add_cache_boundary_doc.md`, `10.fix_annualization_inconsistency.md`, `backlog.md`.
- Verification: Confirmed all 4 new phase files exist on disk; re-read edited files to confirm content applied.
- Pending: PHASE 7 (structured logging) and PHASE 8 (alignment hardening) remain deferred/backlogged; code_review items #1 (`main.rs:495` panic) and #5 (`analysis.rs` duplicated indicators) left to backlog per plan.

### Session 35: Sync Documentation to Codebase

- Date: 2026-07-05
- Agent: Antigravity
- Goal: Update documentation across doc/*.md and README.md to match current codebase state after backtesting and analysis expansion.
- Constraints: None.
- Done:
  - Rebuilt [doc/api_cache.md](file:///c:/Users/lucas/Code/CDG/doc/api_cache.md) with accurate CoinGecko endpoints (tickers, market_chart, market_chart/range, simple/price, global, companies/public_treasury, global/decentralized_finance_defi), fixed stale rate-limit delay claim (no fixed cache-miss delay; retries start at 10s), and added orderbook metrics section.
  - Extended [doc/installation_usage.md](file:///c:/Users/lucas/Code/CDG/doc/installation_usage.md) with `backtest` subcommand, added `run-pipeline` flags (`--concurrency`, `--annualization-factor`, `--backtest`, `--strategy`, `--fee`, `--slippage`, `--rebalance-frequency`), and fixed `check-coin` positional argument note.
  - Updated [doc/analysis_optimization.md](file:///c:/Users/lucas/Code/CDG/doc/analysis_optimization.md) to document annualization factor defaults: `252.0` when `--drop-weekends` is active, `365.0` otherwise.
  - Expanded [doc/custom_strategies.md](file:///c:/Users/lucas/Code/CDG/doc/custom_strategies.md) with built-in strategies table (`rsi`, `macd`, `bollinger`, `all`), backtest execution details (transaction fees, slippage, rebalancing frequencies, US Treasury benchmark, CSV/JSON reports, equity curve plots).
  - Updated [doc/architecture.md](file:///c:/Users/lucas/Code/CDG/doc/architecture.md) Mermaid diagram to include orderbook metrics, ML prep, optimization, backtesting, and export layers. Expanded Core Components with current modules (`pipeline`, `backtest`, `ui`, `export`, `utils`).
  - Fixed typo in [doc/deployment.md](file:///c:/Users/lucas/Code/CDG/doc/deployment.md) ("Data Lakeing" -> "Data Lake") and added `backtests/` directory listing plus standalone `backtest_run_` directory.
  - Updated [README.md](file:///c:/Users/lucas/Code/CDG/README.md) and [doc/README.md](file:///c:/Users/lucas/Code/CDG/doc/README.md) with missing custom-strategies doc links, new features (orderbook metrics, advanced indicators, strategy backtesting), `backtest` subcommand example, expanded CLI flags table, and updated output directory tree with `backtests/`.
- Blocked: None.
- Risk: None.
- Artifact: [doc/api_cache.md](file:///c:/Users/lucas/Code/CDG/doc/api_cache.md), [doc/installation_usage.md](file:///c:/Users/lucas/Code/CDG/doc/installation_usage.md), [doc/analysis_optimization.md](file:///c:/Users/lucas/Code/CDG/doc/analysis_optimization.md), [doc/custom_strategies.md](file:///c:/Users/lucas/Code/CDG/doc/custom_strategies.md), [doc/architecture.md](file:///c:/Users/lucas/Code/CDG/doc/architecture.md), [doc/deployment.md](file:///c:/Users/lucas/Code/CDG/doc/deployment.md), [README.md](file:///c:/Users/lucas/Code/CDG/README.md), [doc/README.md](file:///c:/Users/lucas/Code/CDG/doc/README.md).
- Verification: Manually inspected updated documentation files and verified cross-links and relative paths resolve correctly.
- Pending: None.

### Session 34: Add Settings Submenu to Interactive UI

- Date: 2026-07-04
- Agent: Antigravity
- Goal: Replace the "Configure Cache TTL" main menu option with a "Settings" sub-menu in interactive mode, displaying a warning message when opened.
- Constraints: None.
- Done:
  - Replaced the "Configure Cache TTL" option with a "Settings" option in the main interactive menu of [ui.rs](file:///c:/Users/lucas/Code/CDG/src/ui.rs#L46-L55).
  - Implemented the "Settings" sub-menu in [ui.rs](file:///c:/Users/lucas/Code/CDG/src/ui.rs#L364-L403) that displays a warning print and prompts with "Configure Cache TTL" and "Back" settings choices.
  - Removed the env file warning print from the "Run Portfolio Pipeline" menu selection in [ui.rs](file:///c:/Users/lucas/Code/CDG/src/ui.rs#L77) and moved it to the "Settings" menu option.
  - Adjusted the outer menu loop post-match check in [ui.rs](file:///c:/Users/lucas/Code/CDG/src/ui.rs#L405-L407) to skip `wait_for_back()` if the choice was `"Settings"`.
  - Added functional requirement `FR23` to [SPEC.md](file:///c:/Users/lucas/Code/CDG/.agents/SPEC.md#L66-L67).
  - Updated documentation files [installation_usage.md](file:///c:/Users/lucas/Code/CDG/doc/installation_usage.md#L55) and [api_cache.md](file:///c:/Users/lucas/Code/CDG/doc/api_cache.md#L67) to reference the new settings submenu path.
- Blocked: None.
- Risk: None.
- Artifact: [ui.rs](file:///c:/Users/lucas/Code/CDG/src/ui.rs), [SPEC.md](file:///c:/Users/lucas/Code/CDG/.agents/SPEC.md), [installation_usage.md](file:///c:/Users/lucas/Code/CDG/doc/installation_usage.md), [api_cache.md](file:///c:/Users/lucas/Code/CDG/doc/api_cache.md).
- Verification: Compiled successfully using `cargo check`.
- Pending: None.

### Session 33: Modularization and Polars Optimization

- Date: 2026-07-04
- Agent: Antigravity
- Goal: Clean up CLI monolith, optimize DataFrame column insertions, and simulate portfolio rebalancing transaction costs.
- Constraints: None.
- Done:
  - Created [pipeline.rs](file:///c:/Users/lucas/Code/CDG/src/pipeline.rs) and [ui.rs](file:///c:/Users/lucas/Code/CDG/src/ui.rs) to modularize CLI logic from [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs).
  - Registered new modules in [lib.rs](file:///c:/Users/lucas/Code/CDG/src/lib.rs) and cleaned up [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs).
  - Optimized [analysis.rs](file:///c:/Users/lucas/Code/CDG/src/analysis.rs) by batch-inserting calculated indicators using `hstack` instead of iterating `.insert_column` calls.
  - Added transaction fee and slippage math to portfolio daily rebalancing in [backtest.rs](file:///c:/Users/lucas/Code/CDG/src/backtest.rs) and [pipeline.rs](file:///c:/Users/lucas/Code/CDG/src/pipeline.rs).
  - Added calendar-based rebalancing frequency options (daily, weekly, monthly) configurable via `--rebalance-frequency` CLI flag and `CDG_REBALANCE_FREQUENCY` env var.
  - Modeled weight drift on non-rebalancing days in the portfolio simulation.
  - Added interactive prompt selections for rebalancing frequency.
  - Created ADR 001 to document choices on native Polars expressions for indicators.
  - Created [env.example](file:///c:/Users/lucas/Code/CDG/.env.example) template file.
  - Added warning in [ui.rs](file:///c:/Users/lucas/Code/CDG/src/ui.rs) notifying user to edit `.env` for permanent defaults.
- Blocked: None.
- Risk: None.
- Artifacts: [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs), [lib.rs](file:///c:/Users/lucas/Code/CDG/src/lib.rs), [pipeline.rs](file:///c:/Users/lucas/Code/CDG/src/pipeline.rs), [ui.rs](file:///c:/Users/lucas/Code/CDG/src/ui.rs), [analysis.rs](file:///c:/Users/lucas/Code/CDG/src/analysis.rs), [backtest.rs](file:///c:/Users/lucas/Code/CDG/src/backtest.rs), [Cargo.toml](file:///c:/Users/lucas/Code/CDG/Cargo.toml), [env.example](file:///c:/Users/lucas/Code/CDG/.env.example).
- Verification: `cargo test` - 50 tests passed.
- Pending: None.

### Session 32: Custom Backtesting Strategy JSON Support

- Date: 2026-07-04
- Agent: Antigravity
- Goal: Implement custom strategy definitions parsed from JSON configs (supporting single, list, and map formats) in standalone backtest and full pipeline commands, using mathematical signs for comparisons and without new dependencies.
- Constraints: None.
- Done:
  - Added recursive `Condition` logic trees and `ConfidenceConfig` structs to [backtest.rs](file:///c:/Users/lucas/Code/CDG/src/backtest.rs).
  - Modified `run_backtest_for_asset` signature to accept `custom_strat` directly and resolve indicators with coin prefixes.
  - Implemented `load_custom_strategies` to support single strategy, list of strategies, and map of strategies.
  - Simplified comparison operators to use mathematical comparison signs (`<`, `>`, `==`, `<=`, `>=`) only.
  - Updated all unit tests and integration tests.
  - Added dedicated guide [custom_strategies.md](file:///c:/Users/lucas/Code/CDG/doc/custom_strategies.md) and linked it in all modular documents.
- Blocked: None.
- Risk: None.
- Artifact: [backtest.rs](file:///c:/Users/lucas/Code/CDG/src/backtest.rs), [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs), [pipeline_tests.rs](file:///c:/Users/lucas/Code/CDG/tests/pipeline_tests.rs), [custom_strategies.md](file:///c:/Users/lucas/Code/CDG/doc/custom_strategies.md).
- Verification: `cargo test` - 49 tests passed. Verified CLI and pipeline execution against custom JSON configs.
- Pending: None.

### Session 31: Resolve Flat Backtest Curves & Align Plots

- Date: 2026-07-04
- Agent: Antigravity
- Goal: Fix flat curves in backtest plots caused by missing warm-up data for indicators, and slice returned curves to show only active periods.
- Constraints: None.
- Done:
  - Updated historical fetching timestamp range in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L475-L480) to query extra days for indicator warm-up.
  - Modified [backtest.rs](file:///c:/Users/lucas/Code/CDG/src/backtest.rs#L484-L486) signature to accept `days_to_backtest` and return sliced active equity curves.
  - Sliced dates arrays inside [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L880-L1040) before plotting to fit the active backtesting range.
  - Aligned start index for the US 10-Year Treasury compounded return calculation in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L946-L950) to match the backtest window duration.
- Blocked: None.
- Risk: None.
- Artifact: [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs), [backtest.rs](file:///c:/Users/lucas/Code/CDG/src/backtest.rs).
- Verification: `cargo test` - 46 tests passed. Pipeline execution verified to trigger active trades (e.g. 4 trades on MACD) and compound Treasury yield over exactly the 30-day window (returning 0.37%).
- Pending: None.

### Session 30: Organize Backtest Output Folders

- Date: 2026-07-04
- Agent: Antigravity
- Goal: Store all backtesting output files (CSV/JSON reports and PNG charts) in a dedicated `backtests/` folder inside the `run_xxxx/` directory.
- Constraints: None.
- Done:
  - Created `backtest_dir` inside the pipeline run directory in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L843-L847).
  - Updated paths for all individual and portfolio backtest PNG charts to write to `backtests/` subdirectory in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L872-L1021).
  - Redirected output files `backtest_report.csv` and `backtest_report.json` to compile into `backtests/` in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L1040-L1078).
- Blocked: None.
- Risk: None.
- Artifact: [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs).
- Verification: `cargo test` - 46 tests passed. Pipeline execution verified to export all backtesting assets to `cdg_files/run_xxxx/backtests/`.
- Pending: None.

### Session 29: Cache B&H and Add Treasury Benchmark

### Session 28: Align Backtest Starting Indices

- Date: 2026-07-04
- Agent: Antigravity
- Goal: Align starting indices across all strategy and portfolio backtests to ensure matching comparison windows (apples-to-apples comparison).
- Constraints: None.
- Done:
  - Updated `run_backtest_for_asset` in [backtest.rs](file:///c:/Users/lucas/Code/CDG/src/backtest.rs#L244-L264) to calculate `start_idx` based on the validity of all present indicators (RSI, MACD, Bollinger Bands) instead of only the target strategy's indicator.
  - Updated `backtest_portfolio` in [backtest.rs](file:///c:/Users/lucas/Code/CDG/src/backtest.rs#L541-L569) to verify all present indicators for all portfolio assets when determining start index.
- Blocked: None.
- Risk: None.
- Artifact: [backtest.rs](file:///c:/Users/lucas/Code/CDG/src/backtest.rs).
- Verification: `cargo test` - 46 tests passed. Pipeline execution verified to produce identical Buy & Hold returns for different strategy runs of the same asset.
- Pending: None.

### Session 27: Implement Dynamic Risk-Free Rate

- Date: 2026-07-03
- Agent: Antigravity
- Goal: Implement codebase refactors and correctness enhancements based on deep code review findings.
- Constraints: None.
- Done:
  - Fixed covariance misalignment bug in [optimization.rs](file:///c:/Users/lucas/Code/CDG/src/optimization.rs#L64-L100) by extracting aligned prices synchronously.
  - Corrected weekend gap volume fill strategy in [analysis.rs](file:///c:/Users/lucas/Code/CDG/src/analysis.rs#L316-L328) to fill nulls with 0.0 instead of forward-filling.
  - Added JSON validation checks before caching in [coingecko.rs](file:///c:/Users/lucas/Code/CDG/src/api/coingecko.rs#L128-L136) and [yahoo.rs](file:///c:/Users/lucas/Code/CDG/src/api/yahoo.rs#L85-L101).
  - Implemented exponential backoff retry loop in Yahoo client [yahoo.rs](file:///c:/Users/lucas/Code/CDG/src/api/yahoo.rs#L50-L107) for API request resilience.
  - Cleaned up dead strategy returns code in [backtest.rs](file:///c:/Users/lucas/Code/CDG/src/backtest.rs#L145-L171).
  - Added unit tests `test_align_datasets_volume_filling` and `test_covariance_date_alignment` in [pipeline_tests.rs](file:///c:/Users/lucas/Code/CDG/tests/pipeline_tests.rs#L129-L153).
- Blocked: None.
- Risk: None.
- Artifact: [optimization.rs](file:///c:/Users/lucas/Code/CDG/src/optimization.rs), [analysis.rs](file:///c:/Users/lucas/Code/CDG/src/analysis.rs), [coingecko.rs](file:///c:/Users/lucas/Code/CDG/src/api/coingecko.rs), [yahoo.rs](file:///c:/Users/lucas/Code/CDG/src/api/yahoo.rs), [backtest.rs](file:///c:/Users/lucas/Code/CDG/src/backtest.rs), [pipeline_tests.rs](file:///c:/Users/lucas/Code/CDG/tests/pipeline_tests.rs).
- Verification: `cargo test` - 44 tests passed (including 2 new tests). Pipeline execution manually verified end-to-end.
- Pending: None.


### Session 26: Implement CLI Warning Polish

- Date: 2026-07-03
- Agent: Antigravity
- Goal: Implement progress bar inline status updates for rate limits and transient connection issues instead of stdout/stderr warning spam.
- Constraints: None.
- Done:
  - Added optional `pb: Option<ProgressBar>` to `CoinGeckoClient` in [coingecko.rs](file:///c:/Users/lucas/Code/CDG/src/api/coingecko.rs#L18) and `YahooClient` in [yahoo.rs](file:///c:/Users/lucas/Code/CDG/src/api/yahoo.rs#L8).
  - Modified transient/rate-limit error handler branches in both clients to update progress bar messages if present.
  - Linked active progress bars from [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L579) to clients during pipeline and benchmark historical data fetch runs.
  - Added unit test `test_clients_with_progress_bar` in [api_tests.rs](file:///c:/Users/lucas/Code/CDG/tests/api_tests.rs#L219-L238).
  - Removed implemented items from [BACKLOG.md](file:///c:/Users/lucas/Code/CDG/.agents/BACKLOG.md).
- Blocked: None.
- Risk: None.
- Artifact: [coingecko.rs](file:///c:/Users/lucas/Code/CDG/src/api/coingecko.rs), [yahoo.rs](file:///c:/Users/lucas/Code/CDG/src/api/yahoo.rs), [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs), [api_tests.rs](file:///c:/Users/lucas/Code/CDG/tests/api_tests.rs), [BACKLOG.md](file:///c:/Users/lucas/Code/CDG/.agents/BACKLOG.md).
- Verification: `cargo test` - 45 tests passed. Pipeline execution verified to update progress message.
- Pending: None.

### Session 25: Implement Code Review Findings

- Date: 2026-07-03
- Agent: Antigravity
- Goal: Implement codebase refactors and correctness enhancements based on deep code review findings.
- Constraints: None.
- Done:
  - Fixed covariance misalignment bug in [optimization.rs](file:///c:/Users/lucas/Code/CDG/src/optimization.rs#L64-L100) by extracting aligned prices synchronously.
  - Corrected weekend gap volume fill strategy in [analysis.rs](file:///c:/Users/lucas/Code/CDG/src/analysis.rs#L316-L328) to fill nulls with 0.0 instead of forward-filling.
  - Added JSON validation checks before caching in [coingecko.rs](file:///c:/Users/lucas/Code/CDG/src/api/coingecko.rs#L128-L136) and [yahoo.rs](file:///c:/Users/lucas/Code/CDG/src/api/yahoo.rs#L85-L101).
  - Implemented exponential backoff retry loop in Yahoo client [yahoo.rs](file:///c:/Users/lucas/Code/CDG/src/api/yahoo.rs#L50-L107) for API request resilience.
  - Cleaned up dead strategy returns code in [backtest.rs](file:///c:/Users/lucas/Code/CDG/src/backtest.rs#L145-L171).
  - Added unit tests `test_align_datasets_volume_filling` and `test_covariance_date_alignment` in [pipeline_tests.rs](file:///c:/Users/lucas/Code/CDG/tests/pipeline_tests.rs#L129-L153).
- Blocked: None.
- Risk: None.
- Artifact: [optimization.rs](file:///c:/Users/lucas/Code/CDG/src/optimization.rs), [analysis.rs](file:///c:/Users/lucas/Code/CDG/src/analysis.rs), [coingecko.rs](file:///c:/Users/lucas/Code/CDG/src/api/coingecko.rs), [yahoo.rs](file:///c:/Users/lucas/Code/CDG/src/api/yahoo.rs), [backtest.rs](file:///c:/Users/lucas/Code/CDG/src/backtest.rs), [pipeline_tests.rs](file:///c:/Users/lucas/Code/CDG/tests/pipeline_tests.rs).
- Verification: `cargo test` - 44 tests passed (including 2 new tests). Pipeline execution manually verified end-to-end.
- Pending: None.


### Session 24: Backtest Optimized Portfolios

- Date: 2026-07-03
- Agent: Antigravity
- Goal: Backtest the optimized Markowitz Monte Carlo portfolios and compare their performance against individual asset strategies.
- Constraints: None.
- Done:
  - Implemented `backtest_portfolio` function in [backtest.rs](file:///c:/Users/lucas/Code/CDG/src/backtest.rs#L507-L642) simulating Daily Rebalanced strategy and Buy & Hold baseline.
  - Aligned starting index with indicator warm-up period to ensure comparable timeframes.
  - Reorganized pipeline flow in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs) to optimize first, then run asset and portfolio backtests, consolidating metrics.
  - Added portfolio backtest PNG charts plotting and consolidated CSV/JSON report exports.
- Blocked: None.
- Risk: None.
- Artifact: [backtest.rs](file:///c:/Users/lucas/Code/CDG/src/backtest.rs), [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs).
- Verification: `cargo test` - 42 tests passed. Aligned rebalancing portfolio backtest validated successfully on command line.
- Pending: None.

### Session 23: Portfolio Covariance & Backtest Integration

- Date: 2026-07-03
- Agent: Antigravity
- Goal: Fix annualization factor signature on Monte Carlo optimizer, clean unused indicators, and implement the technical strategy backtesting engine, charts plotting, CLI subcommands, and interactive loops.
- Constraints: None.
- Done:
  - Refactored `run_monte_carlo` signature in [optimization.rs](file:///c:/Users/lucas/Code/CDG/src/optimization.rs#L51-L132) to take a single `annualization_factor: f64` to solve matrix singular scaling issues.
  - Removed unused raw indicator methods and tests in [analysis.rs](file:///c:/Users/lucas/Code/CDG/src/analysis.rs).
  - Implemented backtesting simulation loop in [backtest.rs](file:///c:/Users/lucas/Code/CDG/src/backtest.rs) with persistent position handling and indicator-distance sizing scaling.
  - Implemented backtest equity curve charts plotting in [plot.rs](file:///c:/Users/lucas/Code/CDG/src/plot.rs).
  - Wired backtesting options to `run-pipeline` CLI parameters, added standalone `backtest` command, and interactive menu dialogs in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs).
- Blocked: None.
- Risk: Upstream CoinGecko API rate limit (429) might trigger if requesting multiple coins outside cache.
- Artifact: [optimization.rs](file:///c:/Users/lucas/Code/CDG/src/optimization.rs), [analysis.rs](file:///c:/Users/lucas/Code/CDG/src/analysis.rs), [backtest.rs](file:///c:/Users/lucas/Code/CDG/src/backtest.rs), [plot.rs](file:///c:/Users/lucas/Code/CDG/src/plot.rs), [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs).
- Verification: `cargo test` - 41 tests passed. Standalone backtest and pipeline backtest verified on command line and generated output files successfully.
- Pending: None.


### Session 22: Cache Connection Ping Requests

- Date: 2026-07-03
- Agent: Antigravity
- Goal: Cache connection ping requests to prevent API rate limit (429) errors on subsequent runs.
- Constraints: None.
- Done:
  - Enabled caching for CoinGecko ping requests in [coingecko.rs](file:///c:/Users/lucas/Code/CDG/src/api/coingecko.rs#L139-L143).
  - Enabled caching for Yahoo Finance ping requests by rounding timestamps to the hour in [yahoo.rs](file:///c:/Users/lucas/Code/CDG/src/api/yahoo.rs#L65-L71).
- Blocked: None.
- Risk: None.
- Artifact: [coingecko.rs](file:///c:/Users/lucas/Code/CDG/src/api/coingecko.rs), [yahoo.rs](file:///c:/Users/lucas/Code/CDG/src/api/yahoo.rs).
- Verification: `cargo test` (41 tests passed), and verified subsequent CLI runs execute without hitting network for `/ping` endpoints.
- Pending: None.

### Session 21: Resolve Markowitz Weights Convergence

- Date: 2026-07-03
- Agent: Antigravity
- Goal: Fix Markowitz Monte Carlo simulation producing identical weights due to correlated Xorshift seeds.
- Constraints: None.
- Done:
  - Implemented `splitmix64` hash helper in [optimization.rs](file:///c:/Users/lucas/Code/CDG/src/optimization.rs#L44-L50).
  - Hashed the Monte Carlo index seeds in [optimization.rs](file:///c:/Users/lucas/Code/CDG/src/optimization.rs#L169-L171) to ensure high-entropy/diverse weight outputs.
- Blocked: None.
- Risk: None.
- Artifact: [optimization.rs](file:///c:/Users/lucas/Code/CDG/src/optimization.rs).
- Verification: `cargo test` (41 tests passed), and run-pipeline smoke test verifying different weights (e.g. 1.03% vs 98.84% BTC).
- Pending: None.

### Session 20: Output Savings Refinement

- Date: 2026-07-03
- Agent: Antigravity
- Goal: Implement configurable base output directory and configurable raw format via CLI/env, with nested raw OHLCV folder.
- Constraints: None.
- Done:
  - Added `--output-dir` parameter (default `"cdg_files"`, env `CDG_OUTPUT_DIR`) to `Cli` struct in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L9-L16).
  - Added `--raw-format` parameter (default `"json"`, env `CDG_RAW_FORMAT`) to `Cli` struct and validated it in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L114-L122).
  - Made `db_path` and `output_prefix` fields optional and dynamically resolved in `main` of [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L123-L130).
  - Passed `output_dir` and `raw_format` in `PipelineConfig` and formatted run/candlestick output directories dynamically in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L301-L335).
  - Nested raw OHLCV folder under `raw_ohlcv` inside the pipeline run directory and saved only in the configured format in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L329-L335).
  - Changed `run_ohlcv_flow` and `run_interactive_menu` signature and calls to pass `output_dir` and `raw_format` in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L759-L1139).
  - Added unit tests `test_dynamic_path_resolution` and `test_raw_format_validation` in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L1208-L1320).
  - Updated specifications in [SPEC.md](file:///c:/Users/lucas/Code/CDG/.agents/SPEC.md#L59-L64).
  - Updated user-facing documentation in [README.md](file:///c:/Users/lucas/Code/CDG/README.md#L107-L198), [installation_usage.md](file:///c:/Users/lucas/Code/CDG/doc/installation_usage.md#L114-L119), and [deployment.md](file:///c:/Users/lucas/Code/CDG/doc/deployment.md#L20-L36).
- Blocked: None.
- Risk: None.
- Artifact: [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs), [SPEC.md](file:///c:/Users/lucas/Code/CDG/.agents/SPEC.md), [README.md](file:///c:/Users/lucas/Code/CDG/README.md), [installation_usage.md](file:///c:/Users/lucas/Code/CDG/doc/installation_usage.md), [deployment.md](file:///c:/Users/lucas/Code/CDG/doc/deployment.md).
- Verification: `cargo test` (41 tests passed), and smoke test run verifying JSON and CSV exports.
- Pending: None.

### Session 19: Dynamic Concurrency Limit

- Date: 2026-07-03
- Agent: Antigravity
- Goal: Implement key-aware concurrency limit defaulting (1 for demo/free keys, 3 for pro keys), overridable by CLI flag or env var.
- Constraints: None.
- Done:
  - Modified `concurrency` in clap CLI parser and `PipelineConfig` to be `Option<usize>` in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L58-L60).
  - Implemented automatic CoinGecko API key support (Demo/Pro base URL and headers) in [coingecko.rs](file:///c:/Users/lucas/Code/CDG/src/api/coingecko.rs#L22-L41).
  - Implemented default concurrency resolution logic in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L290-L302) based on key presence.
  - Updated interactive CLI menu default concurrency to adapt to key presence in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L915-L930).
  - Added unit test `test_default_concurrency_resolution` in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L1182-L1232).
- Blocked: None.
- Risk: None.
- Artifact: [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs), [coingecko.rs](file:///c:/Users/lucas/Code/CDG/src/api/coingecko.rs).
- Verification: `cargo test` passed 39/39 tests. `cargo clippy` passed with zero errors/warnings.
- Pending: None.

### Session 18: Implement Code Review Findings

- Date: 2026-07-03
- Agent: Antigravity
- Goal: Implement all fixes and structural refactors from the code review findings.
- Constraints: None.
- Done:
  - Added empty guard check to prevent panics in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L514-L518).
  - Excluded benchmarks from portfolio covariance/weights optimization, and added minimum weight constraint in [optimization.rs](file:///c:/Users/lucas/Code/CDG/src/optimization.rs#L160-L173).
  - Sanitized and validated input paths in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L442-L457).
  - Removed unconditional 3s sleep on cache misses in [coingecko.rs](file:///c:/Users/lucas/Code/CDG/src/api/coingecko.rs#L70-L77).
  - Refactored duplicate indicator logic in [analysis.rs](file:///c:/Users/lucas/Code/CDG/src/analysis.rs#L852-L974).
  - Prompted for concurrency and annualization override in interactive menu of [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L865-L885).
  - Updated `calculate_sharpe` signature in [backtest.rs](file:///c:/Users/lucas/Code/CDG/src/backtest.rs#L75-L95).
  - Cleaned up ticker pagination lifetimes in [coingecko.rs](file:///c:/Users/lucas/Code/CDG/src/api/coingecko.rs#L215-L225).
  - Added SQLite WAL cleanup helper to [api_tests.rs](file:///c:/Users/lucas/Code/CDG/tests/api_tests.rs#L4-L10).
  - Refactored pipeline flow arguments into `PipelineConfig` struct in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L259-L285).
  - Configured native `cmd /c cls` for terminal clearing on Windows in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L798-L808).
  - Added CSV and Parquet export unit tests in [export.rs](file:///c:/Users/lucas/Code/CDG/src/export.rs#L20-L48).
- Blocked: None.
- Risk: None.
- Artifact: [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs), [optimization.rs](file:///c:/Users/lucas/Code/CDG/src/optimization.rs), [coingecko.rs](file:///c:/Users/lucas/Code/CDG/src/api/coingecko.rs), [analysis.rs](file:///c:/Users/lucas/Code/CDG/src/analysis.rs), [backtest.rs](file:///c:/Users/lucas/Code/CDG/src/backtest.rs), [export.rs](file:///c:/Users/lucas/Code/CDG/src/export.rs), [api_tests.rs](file:///c:/Users/lucas/Code/CDG/tests/api_tests.rs).
- Verification: `cargo test` passed 38/38 tests. `cargo clippy` passed with zero errors/warnings.
- Pending: None.

### Session 17: Add New Items to Backlog

- Date: 2026-07-03
- Agent: Antigravity
- Goal: Add Markowitz portfolio weight issue and benchmarks separation issue to backlog for future analysis.
- Constraints: None.
- Done:
  - Added weight 0 selected coins issue and benchmarks comparison-only issue to [BACKLOG.md](file:///c:/Users/lucas/Code/CDG/.agents/BACKLOG.md#L13-L14).
- Blocked: None.
- Risk: None.
- Artifact: [BACKLOG.md](file:///c:/Users/lucas/Code/CDG/.agents/BACKLOG.md).
- Verification: File reviewed.
- Pending: None.

### Session 16: Implement Backlog Items

- Date: 2026-07-03
- Agent: Antigravity
- Goal: Implement all items from the backlog: WAL performance settings, compile-time macros offline metadata, parallel ingestion, and asset-specific annualization.
- Constraints: None.
- Done:
  - Enabled WAL mode and Normal synchronization in [cache.rs](file:///c:/Users/lucas/Code/CDG/src/cache.rs#L38-L42).
  - Replaced runtime SQL queries with compile-time checked `sqlx::query!` macros in [cache.rs](file:///c:/Users/lucas/Code/CDG/src/cache.rs#L55-L94).
  - Generated `.sqlx/` offline metadata directory in the project root to support offline compilation.
  - Implemented parallel CoinGecko charts/OHLC data fetching using `tokio::task::JoinSet` and `tokio::sync::Semaphore` in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L420-L490).
  - Added CLI flag `--concurrency` (env `COINGECKO_CONCURRENCY`) to control concurrent requests.
  - Updated [optimization.rs](file:///c:/Users/lucas/Code/CDG/src/optimization.rs#L44-L185) to dynamically scale returns and covariance matrices using asset-specific annualization factors.
  - Added heuristic logic to classify asset class (Crypto -> 365, Stocks -> 252) and added CLI flag `--annualization-factor` (env `ANNUALIZATION_FACTOR`) to override all factors to a single custom value.
  - Added unit test `test_asset_specific_annualization` in [optimization.rs](file:///c:/Users/lucas/Code/CDG/src/optimization.rs#L353-L373) and updated existing test files.
  - Cleared all implemented items from [BACKLOG.md](file:///c:/Users/lucas/Code/CDG/.agents/BACKLOG.md).
  - Updated specifications in [SPEC.md](file:///c:/Users/lucas/Code/CDG/.agents/SPEC.md).
- Blocked: None.
- Risk: CoinGecko free API rate limit (429) might trigger if concurrency is set too high.
- Artifact: [cache.rs](file:///c:/Users/lucas/Code/CDG/src/cache.rs), [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs), [optimization.rs](file:///c:/Users/lucas/Code/CDG/src/optimization.rs), [BACKLOG.md](file:///c:/Users/lucas/Code/CDG/.agents/BACKLOG.md), [SPEC.md](file:///c:/Users/lucas/Code/CDG/.agents/SPEC.md), [walkthrough.md](file:///C:/Users/lucas/.gemini/antigravity-ide/brain/728f513f-51b6-4f83-aa45-257d1328dfe4/walkthrough.md).
- Verification: `$env:SQLX_OFFLINE="true"; cargo test` passed 37 tests.
- Pending: None.

### Session 15: Add Customizable Cache TTL

- Date: 2026-07-01
- Agent: Antigravity
- Goal: Add configurable cache TTL command-line flag and interactive menu option.
- Constraints: None.
- Done:
  - Added `--cache-ttl` global command-line argument to `Cli` in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L17-L19).
  - Configured CoinGecko and Yahoo Finance API clients to use custom cache TTL.
  - Added "Configure Cache TTL" option to the interactive console menu in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L931-L941).
  - Fixed clippy redundant field warning in [optimization.rs](file:///c:/Users/lucas/Code/CDG/src/optimization.rs#L212).
  - Updated documentation in [installation_usage.md](file:///c:/Users/lucas/Code/CDG/doc/installation_usage.md) and [api_cache.md](file:///c:/Users/lucas/Code/CDG/doc/api_cache.md).
  - Added CLI parser unit test `test_cli_parsing_cache_ttl` in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L981-L986).
- Blocked: None.
- Risk: Short TTL values (e.g. <30s) could increase HTTP 429 rate limit errors from CoinGecko under high traffic.
- Artifact: [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs), [optimization.rs](file:///c:/Users/lucas/Code/CDG/src/optimization.rs), [installation_usage.md](file:///c:/Users/lucas/Code/CDG/doc/installation_usage.md), [api_cache.md](file:///c:/Users/lucas/Code/CDG/doc/api_cache.md).
- Verification: `cargo test` passed 28/28 tests; `cargo clippy` and `cargo fmt` passed with zero errors/warnings.
- Pending: None.

### Session 14: Generate Project Documentation

- Date: 2026-07-01
- Agent: Antigravity
- Goal: Create modular, comprehensive documentation system in doc/ directory.
- Constraints: None.
- Done:
  - Created [README.md](file:///c:/Users/lucas/Code/CDG/doc/README.md) hub to organize documentation system.
  - Created [architecture.md](file:///c:/Users/lucas/Code/CDG/doc/architecture.md) detailing ingestion/processing flows and Mermaid diagrams.
  - Created [installation_usage.md](file:///c:/Users/lucas/Code/CDG/doc/installation_usage.md) detailing CLI commands and interactive pager.
  - Created [api_cache.md](file:///c:/Users/lucas/Code/CDG/doc/api_cache.md) detailing CoinGecko retry mechanisms and SQLite caching logic.
  - Created [analysis_optimization.md](file:///c:/Users/lucas/Code/CDG/doc/analysis_optimization.md) documenting technical indicators formulas and Monte Carlo simulation.
  - Created [deployment.md](file:///c:/Users/lucas/Code/CDG/doc/deployment.md) covering output folder structure and containerization / GCP Cloud Run integration.
  - Modified root [README.md](file:///c:/Users/lucas/Code/CDG/README.md) to link to the new documentation files, added the header navigation bar, and resolved MD040 linter warnings on fenced code blocks.
- Blocked: None.
- Risk: None.
- Artifact: `doc/` directory files: `doc/README.md`, `doc/architecture.md`, `doc/installation_usage.md`, `doc/api_cache.md`, `doc/analysis_optimization.md`, `doc/deployment.md`.
- Verification: Passed `cargo test` successfully (all 27 tests passed).
- Pending: None.

### Session 13: AI Engineering Infrastructure Sync

- Date: 2026-07-01
- Agent: Antigravity
- Goal: Mirror CDGonGCP's AI engineering setup (skills, rules, docs) into CDG workspace.
- Constraints: Preserve CDG project-specific content (SPEC.md, PROGRESS.md, BACKLOG.md, ADRs).
- Done:
  - Added 3 new rules to `.agents/rules/`: `harness.md`, `structure.md`, `workflow.md`.
  - Updated `rules/engineering.md`: added Rule 7 (Relative Paths), fixed blocker link to `rules/workflow.md`.
  - Updated `rules/safety.md`: removed stale TASKS.md reference.
  - Updated `rules/load.md`: replaced root file links with rules/ equivalents; removed TASKS refs.
  - Updated `rules/structure.md`: reflects actual `.agents/` root; added `etc/` dir; removed stale entries.
  - Added 4 new engineering skills: `archive_progress`, `clean_architecture`, `finish_session`, `spec_triage`.
  - Added 1 new productivity skill: `project_documentation`.
  - Deleted obsolete `.agents/` root files: `HARNESS.md`, `STRUCTURE.md`, `WORKFLOW.md`, `TASKS.md`, `TASKS_ARCHIVE.md`.
  - Created `.agents/etc/` dir; copied `cdg-lib-migration-candidates.md` from CDGonGCP.
  - Aligned `.agentignore` with CDGonGCP format (normalized db glob patterns, fixed DS_Store casing).
  - Aligned `.gitignore` with CDGonGCP (added `.kilo/.gemini/.claude`; normalized db patterns; restored Python runtime artifacts for skill scripts; removed packaging bloat).
- Blocked: None.
- Risk: None — infrastructure only, no source code touched.
- Artifact: `.agents/rules/`, `.agents/skills/engineering/`, `.agents/skills/productivity/`, `.agents/etc/cdg-lib-migration-candidates.md`.
- Verification: Manual — all rule files reference valid paths; all new skill SKILL.md files present.
- Pending: CDG lib migration work (see `.agents/etc/cdg-lib-migration-candidates.md`).

### Session 12: Interactive CLI and Raw OHLCV Exporter Enhancements

- Date: June 13, 2026
- Agent: Antigravity
- Done: Added raw OHLCV folder output, improved CLI pager UX with terminal clearing and a [Back] button, and updated coin listing to fetch/display top 50 coins by market cap.
- Actions:
  - main.rs:
    - Implemented clear_terminal and wait_for_back helpers.
    - Cleared terminal before displaying the interactive menu and executing actions.
    - Appended [Back] button logic at the end of non-Exit options.
    - Saved fetched raw OHLCV data to `cdg_files/can_YYYYMMDD_HHMMSS` as JSON and CSV in run_pipeline_flow and run_ohlcv_flow.
    - Modified list-coins subcommand and interactive "List Supported Coins" menu action to query the `/coins/markets` endpoint (sorted by market cap desc) instead of `/coins/list`.
  - Spec: Updated [SPEC.md](file:///c:/Users/lucas/Code/CDG/.agents/SPEC.md) with functional requirements FR13, FR16 and FR17.
- Verification:
  - Run `cargo fmt -- --check`: 100% clean.
  - Run `cargo clippy -- -D warnings`: 100% clean.
  - Run `cargo test`: Passed all 27 tests.
  - Manual verification: Verified list-coins prints top 50 by market cap with price and market cap columns.
- Pending: None.

### Session 11: GCP Compatibility Documentation

- Date: June 13, 2026
- Agent: Antigravity
- Done: Refined backlog items to prioritize GCP & Vertex compatibility over migration. Added compatibility guidelines to AGENTS.md, SPEC.md, and created ADR-0001.
- Actions:
  - Backlog: Updated [BACKLOG.md](file:///c:/Users/lucas/Code/CDG/.agents/BACKLOG.md) to replace GCP/Vertex migration with a unified compatibility item.
  - Constitution: Updated [AGENTS.md](file:///c:/Users/lucas/Code/CDG/AGENTS.md) with GCP/Vertex compatibility rules under Repo Scope & Method.
  - Spec: Updated [SPEC.md](file:///c:/Users/lucas/Code/CDG/.agents/SPEC.md) adding Goal 6 for GCP & Vertex compatibility.
   - ADR: Created [0001-gcp-compatibility.md](file:///c:/Users/lucas/Code/CDG/.agents/docs/ADRs/0001-gcp-compatibility.md) documenting design choices for database queries, output standardization, and resource footprint.
- Verification: Documents created and linked successfully.
- Pending: None.

### Session 10: Interactive CLI and Context Utilities Integration

- Date: June 13, 2026
- Agent: Antigravity
- Done: Implemented interactive CLI subcommands, menus, and context utilities as per Phase 12 & 13.
- Actions:
  - Cargo.toml: Added `dialoguer` dependency.
  - yahoo.rs: Added `ping` method to check Yahoo Finance server connection.
  - main.rs: Restructured CLI parsing to support subcommands. Implemented `run_pipeline_flow`, `run_ohlcv_flow`, and `run_interactive_menu` using `dialoguer`. Added graceful terminal signal interrupt handler. Added unit tests for CLI parsing.
  - api_tests.rs: Added mock API tests for listing coins, trending search, and Yahoo ping.
- Verification:
  - Run `cargo fmt -- --check`: 100% clean.
  - Run `cargo clippy -- -D warnings`: 100% clean.
  - Run `rtk cargo test`: 27 passed (all tests green).
  - Manual verification: Verified subcommands `ping`, `check-coin`, `trending`, and `ohlcv` CSV/JSON outputs.
- Pending: None. All tasks completed successfully.

### Session 9: API Error Robustness & Helper Command

- Date: June 13, 2026
- Agent: Antigravity
- Done: Resolved CoinGecko rate limit (429) errors and implemented helper check/suggest command for coin IDs.
- Actions:
  - coingecko.rs:
    - Added exponential backoff retry to `get_request` on HTTP 429 status.
    - Set default rate limit delay to 3000ms.
    - Implemented `check_coin_id` to verify coin IDs and suggest alternatives (symbols/substrings) if invalid.
  - main.rs:
    - Added CLI option `--check-coin <NAME>`.
    - Skips invalid coin IDs with warning instructing users to use `--check-coin`.
  - api_tests.rs: Added `test_coingecko_check_coin_id`.
- Verification:
  - Run `rtk cargo test`: Passed all 22 tests.
  - Run `cargo run -- --check-coin bnb`: Returns suggestions.
  - Run `cargo run -- -c "bitcoin, ethereum, bnb" -v "usd, brl"`: Skips `bnb` with suggestion message, processes valid coins successfully.
- Pending: None.

### Session 8: Planning Next Milestones

- Date: June 13, 2026
- Agent: Antigravity
- Done: Added new context utility tools and JSON output mode ideas to `BACKLOG.md`. Created Phase 12 (Interactive CLI) and Phase 13 (Context Utility Tools) roadmap tasks following STTD conventions, focusing on raw OHLCV extraction for external tools rather than terminal visualization.
- Actions:
  - Backlog: Updated [BACKLOG.md](file:///c:/Users/lucas/Code/CDG/.agents/BACKLOG.md) with context tools and JSON output mode descriptions.
  - Tasks: Populated task roadmap with Phase 12 and 13 milestones.
- Verification:
  - Run `rtk cargo test`: Passed all 21 tests.
- Pending: None. Planning phase complete. Ready for implementation.

### Session 7: Task Completion and Archiving

- Date: June 13, 2026
- Agent: Antigravity
- Done: Archived completed tasks (Phases 9, 10, 11) and old sessions (0-5) to archive files. Updated `README.md` to document the new features.
- Actions:
  - Archiving: Updated archive files, PROGRESS.md.
  - Docs: Updated [README.md](file:///c:/Users/lucas/Code/CDG/README.md) with Monte Carlo portfolio optimization features, progress bars, tables, and the `--seed` flag.
- Verification:
  - Run `rtk cargo test`: Passed all 21 tests.
- Pending: None. All milestones successfully completed and logged.

### Session 6: Code Review and Fixes

- Date: June 13, 2026
- Agent: Antigravity
- Done: Conducted comprehensive code review and applied all high, medium, and low severity fixes (H1, H2, M1, M2, M3, M5, L1, L3, L4, L5, L6) except M7.
- Actions:
  - H1: Changed `CoinGeckoClient::new()` and `YahooClient::new()` to return `Result<Self>`.
  - H2: Added `TRADING_DAYS_PER_YEAR` constant with doc comment in optimization.rs.
  - M1: Replaced `unwrap_or(0.0)` on price series with index-filtered approach in analysis.rs and optimization.rs.
  - M2: Replaced manual query string concatenation in coingecko.rs with `reqwest::Url::parse_with_params`.
  - M3: Added `Connection: close` response header to mock TCP server in api_tests.rs.
  - M5: Changed population variance (N) to sample variance (N-1) in `prep_ml` in analysis.rs.
  - L1: Changed `--days` from `String` to `u32` in clap `Args` struct.
  - L3: Added optional `--seed` CLI flag; threaded through to `run_monte_carlo(seed: Option<u64>)`.
  - L4: Replaced 1100ms `sleep` in cache test with `ttl_secs=0` expiry check.
  - L5: Added `test_full_pipeline_smoke` in pipeline_tests.rs.
  - L6: Bumped `reqwest` from `0.11` to `0.12` with `native-tls` feature.
- Verification:
  - Run `cargo fmt -- --check`: 100% clean.
  - Run `cargo clippy -- -D warnings`: 100% clean.
  - Run `cargo test`: 21 passed.
- Pending: None.

### Session 0: Initial Commit

- Date: June 12, 2026
- Agent: Lead Architect Agent
- Done: Init AI engineering workflow config and git repository setup.
- Actions:
  - Env: Create `.agents/` dir, fill baseline multi-stack templates.
- Verification:
  - Tech: Verify base doc structure and schemas.
- Pending: None. Ready.

### Session 1: Specs and Roadmap Alignment

- Date: June 12, 2026
- Agent: Lead Architect Agent
- Done: Aligned Rust refactoring requirements (reqwest + serde, sqlx sqlite caching, polars analysis, plotters visualization, parquet/csv export, weekend forward-fill logic, and GCP free-tier optimized lightweight mode).
- Actions:
  - Spec: Wrote [SPEC.md](./SPEC.md).
  - Backlog: Wrote [BACKLOG.md](./BACKLOG.md).
  - Tasks: Wrote [TASKS.md](./TASKS.md).
  - Harness: Wrote [HARNESS.md](./HARNESS.md).
- Verification:
  - Files successfully written and schemas populated.
- Pending: None.

### Session 2: Implementation, Testing, and Verification

- Date: June 12, 2026
- Agent: Antigravity
- Done: Completed full Rust implementation, fixed CLI parser panic, fixed Windows SQLite connection issue, added unit/integration tests, resolved all clippy warnings, formatted code, archived legacy Python codebase, implemented multi-currency support, and resolved Yahoo Finance 401 Unauthorized API issues by switching to the v8 JSON chart API.
- Actions:
  - CLI: Changed currency short flag to `-v` in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs) and added comma-separated currency list support.
  - DB: Switched SQLite connection to `SqliteConnectOptions` in [cache.rs](file:///c:/Users/lucas/Code/CDG/src/cache.rs).
  - API: Switched Yahoo Finance client to v8 JSON API in [yahoo.rs](file:///c:/Users/lucas/Code/CDG/src/api/yahoo.rs) and implemented a Polars JSON response parser in [analysis.rs](file:///c:/Users/lucas/Code/CDG/src/analysis.rs).
  - Structure: Created [lib.rs](file:///c:/Users/lucas/Code/CDG/src/lib.rs) and exposed public modules to enable test suite importing.
  - Tests: Added unit tests for Cache in [cache.rs](file:///c:/Users/lucas/Code/CDG/src/cache.rs), updated integration/unit tests for Yahoo JSON parser in [api_tests.rs](file:///c:/Users/lucas/Code/CDG/tests/api_tests.rs) and [analysis.rs](file:///c:/Users/lucas/Code/CDG/src/analysis.rs), and added multi-currency merging tests.
  - Cleanup: Moved legacy python files to `legacy_python/` directory.
- Verification:
  - Run `cargo clippy`: 100% clean.
  - Run `cargo fmt -- --check`: 100% clean.
  - Run `cargo test`: 9 passed (4 suites).
  - Run `cargo run -- -v usd,eur,brl`: Successful multi-currency and stock benchmark data retrieval, merging, processing, and export (no 401 errors).
- Pending: None. All requirements met.

### Session 3: Multi-Cryptocurrency & Multi-Currency Support

- Date: June 12, 2026
- Agent: Antigravity
- Done: Added multi-cryptocurrency and multi-currency parsing and query pipeline, aligned range query timestamps to day boundary to make cached URLs stable, implemented conditional returns and indicator calculation for light mode (first pair only) vs default mode (all combinations), added conditional separate plotting logic (no plots under light mode, separate returns line plots per combination plus performance and risk-return plots under default), and verified compilation, clippy warnings, and test suites.
- Actions:
  - CLI: Modified `main.rs` to parse comma-separated coin lists.
  - Caching: Rounded range query timestamps to daily boundaries to prevent cache misses on repeated runs.
  - Indicator: Prefixed output column names in `compute_returns_and_indicators` in `analysis.rs` with `{target_column}_`.
  - Plotting: Updated `main.rs` to conditionally skip plotting in light mode, plot separate returns PNGs per pair, and include all currency columns in performance and risk-return charts in default mode.
- Verification:
  - Run `cargo fmt -- --check`: 100% clean.
  - Run `cargo clippy`: 100% clean.
  - Run `cargo test`: 9 passed.
  - Verified manual smoke runs for default and lightweight modes.
- Pending: None. Phase 8 completed.

### Session 4: Portfolio Optimization, Color Uniqueness, and Run Directory Setup

- Date: June 13, 2026
- Agent: Antigravity
- Done: Implemented Monte Carlo portfolio optimization, dynamic HSL plot color generation, run-specific output directories, and completed unit/integration tests.
- Actions:
  - Solver: Created [optimization.rs](file:///c:/Users/lucas/Code/CDG/src/optimization.rs) module for covariance calculations and random portfolio simulation.
  - Colors: Wrote HSL-to-RGB color generator in [plot.rs](file:///c:/Users/lucas/Code/CDG/src/plot.rs) to avoid repeating plot line colors.
  - Directories: Updated [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs) to create run directories matching `cdg_files/run_YYYYMMDD_HHMMSS` and export all files/plots there.
  - Verification: Added integration tests in [pipeline_tests.rs](file:///c:/Users/lucas/Code/CDG/tests/pipeline_tests.rs).
- Verification:
  - Run `cargo test`: 13 passed.
  - Run `cargo run`: Created run directory and successfully solved portfolio weights.
- Pending: None. Phase 1 completed.

### Session 5: CoinGecko Endpoints & Technical Indicators & CLI Polish

- Date: June 13, 2026
- Agent: Antigravity
- Done: Added CoinGecko OHLC and Ticker endpoints, implemented advanced technical indicators (ATR, Stochastic, ADX, OBV) using Polars, integrated progress bars and ASCII tables for optimal weights/metrics.
- Actions:
  - Client: Implemented `get_coin_tickers` in [coingecko.rs](file:///c:/Users/lucas/Code/CDG/src/api/coingecko.rs).
  - Parsers: Added OHLC, Ticker, and orderbook metrics parser functions in [analysis.rs](file:///c:/Users/lucas/Code/CDG/src/analysis.rs).
  - Indicators: Added standard Welles Wilder mathematical functions for ATR, Stochastic, ADX, and OBV in [analysis.rs](file:///c:/Users/lucas/Code/CDG/src/analysis.rs).
  - CLI Polish: Added `indicatif` and `cli-table` dependencies, integrated progress spinner during fetching, progress bar during simulation, and formatted tables for output logs in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs).
- Verification:
  - Run `cargo fmt -- --check`: 100% clean.
  - Run `cargo clippy`: 100% clean (0 warnings).
  - Run `cargo test`: 19 passed.
  - Run `cargo run`: Successfully executed with progress bars and printed optimal weights in ASCII tables.
- Pending: None. All tasks completed successfully.
