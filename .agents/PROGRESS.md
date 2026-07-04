# Progress (PROGRESS.md)

## Status

- State: Backtest flat curves resolved and portfolio plots aligned.
- Last: Fetched extra historical warm-up data for indicators and sliced returned curves.

## Log

Old sessions: [PROGRESS_ARCHIVE.md](./PROGRESS_ARCHIVE.md).

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


