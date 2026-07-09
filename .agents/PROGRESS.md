# Progress (PROGRESS.md)

## Status

- State: Implementing Alpha plan; Phases 01–08 implemented.
- Last: Added mocked HTTP integration test for run_pipeline_flow (Phase 08).

## Log

Old sessions: [PROGRESS_ARCHIVE.md](./PROGRESS_ARCHIVE.md).

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

