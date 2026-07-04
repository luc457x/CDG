# Progress (PROGRESS.md)

## Status

- State: Backtest starting indices aligned across strategies and portfolios.
- Last: Calculated start_idx based on the validity of all available indicators, ensuring matching time windows.

## Log

Old sessions: [PROGRESS_ARCHIVE.md](./PROGRESS_ARCHIVE.md).

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


