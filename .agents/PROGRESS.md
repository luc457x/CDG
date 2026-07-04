# Progress (PROGRESS.md)

## Status

- State: Dynamic risk-free rate (^TNX) integrated in portfolio optimization and backtesting Sharpe ratios.
- Last: Fetched treasury yield, calculated risk-free rate adjustments, and verified Sharpe ratio calculations across 46 unit tests.

## Log

Old sessions: [PROGRESS_ARCHIVE.md](./PROGRESS_ARCHIVE.md).

### Session 27: Implement Dynamic Risk-Free Rate

- Date: 2026-07-04
- Agent: Antigravity
- Goal: Integrate dynamic US 10-year Treasury Note yield (^TNX) as the risk-free rate for portfolio optimization and backtesting Sharpe ratio metrics.
- Constraints: None.
- Done:
  - Added `^TNX` to Yahoo Finance benchmark fetch in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L668-L685) and excluded it from index plotting.
  - Implemented average annual risk-free rate parsing and Sharpe excess return optimizations in [optimization.rs](file:///c:/Users/lucas/Code/CDG/src/optimization.rs#L143-L215).
  - Refactored `calculate_sharpe` in [backtest.rs](file:///c:/Users/lucas/Code/CDG/src/backtest.rs#L79-L652) to accept `rf_rate` and compute excess-adjusted Sharpe ratios cleanly.
  - Added unit test `test_backtest_with_risk_free_rate` in [pipeline_tests.rs](file:///c:/Users/lucas/Code/CDG/tests/pipeline_tests.rs#L156-L181).
- Blocked: None.
- Risk: None.
- Artifact: [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs), [optimization.rs](file:///c:/Users/lucas/Code/CDG/src/optimization.rs), [backtest.rs](file:///c:/Users/lucas/Code/CDG/src/backtest.rs), [pipeline_tests.rs](file:///c:/Users/lucas/Code/CDG/tests/pipeline_tests.rs).
- Verification: `cargo test` - 46 tests passed. Pipeline execution verified to fetch yield and output risk-free adjusted Sharpe ratios correctly.
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


