# Progress (PROGRESS.md)

## Status

- State: Implementing Alpha plan; Phases 01–12 implemented.
- Last: Extracted shared backtest report + treasury logic, fixed annualization, cached coins list with 24h TTL, added 5xx/timeout retry to CoinGecko client.

## Log

Old sessions: [PROGRESS_ARCHIVE.md](./PROGRESS_ARCHIVE.md).

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

