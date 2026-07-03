# Progress (PROGRESS.md)

## Status

- State: Code review findings implemented (covariance math, volume filling, JSON cache checks, Yahoo client retries, dead code removed).
- Last: Implemented date-aligned returns calculation, Zero volume fill strategy, JSON body caching validation, Yahoo retry loop, and cleaned up dead code.

## Log

Old sessions: [PROGRESS_ARCHIVE.md](./PROGRESS_ARCHIVE.md).

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

