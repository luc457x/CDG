# Progress (PROGRESS.md)

## Status

- State: B&H caching implemented and US Treasury 10Y comparison row added.
- Last: Cached B&H calculations across strategies and calculated dynamic US Treasury 10Y yields as a backtesting benchmark.

## Log

Old sessions: [PROGRESS_ARCHIVE.md](./PROGRESS_ARCHIVE.md).

### Session 29: Cache B&H and Add Treasury Benchmark

- Date: 2026-07-04
- Agent: Antigravity
- Goal: Skip redundant B&H calculations across different strategy runs, and incorporate a US 10-Year Treasury Note yield (`^TNX`) benchmark row in backtest results.
- Constraints: None.
- Done:
  - Defined `BhCache` in [backtest.rs](file:///c:/Users/lucas/Code/CDG/src/backtest.rs#L190-L417) and updated `run_backtest_for_asset` to accept `bh_cache: &mut Option<BhCache>`. On first run, it computes B&H and updates cache; subsequent runs copy B&H details directly.
  - Linked `BhCache` in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L843-L1226) for both pipeline and standalone runs.
  - Implemented dynamic US Treasury 10Y benchmark calculations in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L881-L1230), compounding daily yields over the aligned backtesting time window.
- Blocked: None.
- Risk: None.
- Artifact: [backtest.rs](file:///c:/Users/lucas/Code/CDG/src/backtest.rs), [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs).
- Verification: `cargo test` - 46 tests passed. Pipeline run manually verified to display `US_TREASURY_10Y` row with cumulative yields.
- Pending: None.

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


