# Progress Log History (PROGRESS_ARCHIVE.md)

## Archived Sessions

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
  - ADR: Created [0001-gcp-compatibility.md](file:///c:/Users/lucas/Code/CDG/.agents/adr/0001-gcp-compatibility.md) documenting design choices for database queries, output standardization, and resource footprint.
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
