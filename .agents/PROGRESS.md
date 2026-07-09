# Progress (PROGRESS.md)

## Status

- State: Implementing Alpha plan; Phases 01–18 implemented.
- Last: Implemented --light conflict warnings and weekend alignment/fill regression tests.

## Log

Old sessions: [PROGRESS_ARCHIVE.md](./PROGRESS_ARCHIVE.md).

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

### Session 42: Implement Ambiguity & Optimization Toggles

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
