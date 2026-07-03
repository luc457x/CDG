# Progress (PROGRESS.md)

## Status

- State: Completed portfolio optimization fixes, strategy backtesting engine, CLI and interactive integrations, and PNG equity chart generation.
- Last: Implemented standalone backtest CLI command, strategy plotting, and verified all 41 cargo tests pass.

## Log

Old sessions: [PROGRESS_ARCHIVE.md](./PROGRESS_ARCHIVE.md).

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

### Session 22: Cache Connection Ping Requests

- Date: 2026-07-03
- Agent: Antigravity
- Goal: Cache connection ping requests to prevent API rate limit (429) errors on subsequent runs.
- Constraints: None.
- Done:
  - Enabled caching for CoinGecko ping requests in [coingecko.rs](file:///c:/Users/lucas/Code/CDG/src/api/coingecko.rs#L139-L143).
  - Enabled caching for Yahoo Finance ping requests by rounding timestamps to the hour in [yahoo.rs](file:///c:/Users/lucas/Code/CDG/src/api/yahoo.rs#L65-L71).
- Blocked: None.
- Risk: None.
- Artifact: [coingecko.rs](file:///c:/Users/lucas/Code/CDG/src/api/coingecko.rs), [yahoo.rs](file:///c:/Users/lucas/Code/CDG/src/api/yahoo.rs).
- Verification: `cargo test` (41 tests passed), and verified subsequent CLI runs execute without hitting network for `/ping` endpoints.
- Pending: None.

### Session 21: Resolve Markowitz Weights Convergence

- Date: 2026-07-03
- Agent: Antigravity
- Goal: Fix Markowitz Monte Carlo simulation producing identical weights due to correlated Xorshift seeds.
- Constraints: None.
- Done:
  - Implemented `splitmix64` hash helper in [optimization.rs](file:///c:/Users/lucas/Code/CDG/src/optimization.rs#L44-L50).
  - Hashed the Monte Carlo index seeds in [optimization.rs](file:///c:/Users/lucas/Code/CDG/src/optimization.rs#L169-L171) to ensure high-entropy/diverse weight outputs.
- Blocked: None.
- Risk: None.
- Artifact: [optimization.rs](file:///c:/Users/lucas/Code/CDG/src/optimization.rs).
- Verification: `cargo test` (41 tests passed), and run-pipeline smoke test verifying different weights (e.g. 1.03% vs 98.84% BTC).
- Pending: None.
