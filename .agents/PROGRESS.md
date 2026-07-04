# Progress (PROGRESS.md)

## Status

- State: CLI modularized, Polars hstack optimized, and portfolio rebalancing fees simulated.
- Last: Refactored analysis.rs to use hstack, split main.rs into pipeline.rs/ui.rs, and verified tests.

## Log

Old sessions: [PROGRESS_ARCHIVE.md](./PROGRESS_ARCHIVE.md).

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
- Blocked: None.
- Risk: None.
- Artifacts: [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs), [lib.rs](file:///c:/Users/lucas/Code/CDG/src/lib.rs), [pipeline.rs](file:///c:/Users/lucas/Code/CDG/src/pipeline.rs), [ui.rs](file:///c:/Users/lucas/Code/CDG/src/ui.rs), [analysis.rs](file:///c:/Users/lucas/Code/CDG/src/analysis.rs), [backtest.rs](file:///c:/Users/lucas/Code/CDG/src/backtest.rs), [Cargo.toml](file:///c:/Users/lucas/Code/CDG/Cargo.toml).
- Verification: `cargo test` - 49 tests passed.

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
