# Progress (PROGRESS.md)

## Status

- State: Completed portfolio backtest simulation and consolidation of all backtest results (individual assets and portfolios) in tables and files.
- Last: Integrated Max Sharpe and Min Volatility portfolio backtests, verified math compounding, and checked file chart generation.

## Log

Old sessions: [PROGRESS_ARCHIVE.md](./PROGRESS_ARCHIVE.md).

### Session 24: Backtest Optimized Portfolios

- Date: 2026-07-03
- Agent: Antigravity
- Goal: Backtest the optimized Markowitz Monte Carlo portfolios and compare their performance against individual asset strategies.
- Constraints: None.
- Done:
  - Implemented `backtest_portfolio` function in [backtest.rs](file:///c:/Users/lucas/Code/CDG/src/backtest.rs#L507-L642) simulating Daily Rebalanced strategy and Buy & Hold baseline.
  - Aligned starting index with indicator warm-up period to ensure comparable timeframes.
  - Reorganized pipeline flow in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs) to optimize first, then run asset and portfolio backtests, consolidating metrics.
  - Added portfolio backtest PNG charts plotting and consolidated CSV/JSON report exports.
- Blocked: None.
- Risk: None.
- Artifact: [backtest.rs](file:///c:/Users/lucas/Code/CDG/src/backtest.rs), [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs).
- Verification: `cargo test` - 42 tests passed. Aligned rebalancing portfolio backtest validated successfully on command line.
- Pending: None.

### Session 23: Portfolio Covariance & Backtest Integration

- Date: 2026-07-03
- Agent: Antigravity
- Goal: Fix annualization factor signature on Monte Carlo optimizer, clean unused indicators, and implement the technical strategy backtesting engine, charts plotting, CLI subcommands, and interactive loops.
- Constraints: None.
- Done:
  - Refactored `run_monte_carlo` signature in [optimization.rs](file:///c:/Users/lucas/Code/CDG/src/optimization.rs#L51-L132) to take a single `annualization_factor: f64` to solve matrix singular scaling issues.
  - Removed unused raw indicator methods and tests in [analysis.rs](file:///c:/Users/lucas/Code/CDG/src/analysis.rs).
  - Implemented backtesting simulation loop in [backtest.rs](file:///c:/Users/lucas/Code/CDG/src/backtest.rs) with persistent position handling and indicator-distance sizing scaling.
  - Implemented backtest equity curve charts plotting in [plot.rs](file:///c:/Users/lucas/Code/CDG/src/plot.rs).
  - Wired backtesting options to `run-pipeline` CLI parameters, added standalone `backtest` command, and interactive menu dialogs in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs).
- Blocked: None.
- Risk: Upstream CoinGecko API rate limit (429) might trigger if requesting multiple coins outside cache.
- Artifact: [optimization.rs](file:///c:/Users/lucas/Code/CDG/src/optimization.rs), [analysis.rs](file:///c:/Users/lucas/Code/CDG/src/analysis.rs), [backtest.rs](file:///c:/Users/lucas/Code/CDG/src/backtest.rs), [plot.rs](file:///c:/Users/lucas/Code/CDG/src/plot.rs), [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs).
- Verification: `cargo test` - 41 tests passed. Standalone backtest and pipeline backtest verified on command line and generated output files successfully.
- Pending: None.

