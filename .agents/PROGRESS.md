# Progress (PROGRESS.md)

## Status

- State: Custom JSON strategy backtesting engine integrated and documented.
- Last: Removed word-based operators and enforced mathematical comparison signs globally.

## Log

Old sessions: [PROGRESS_ARCHIVE.md](./PROGRESS_ARCHIVE.md).

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
