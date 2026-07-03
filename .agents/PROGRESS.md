# Progress (PROGRESS.md)

## Status

- State: Concurrency default dynamically resolved (1 for demo/free, 3 for pro), overridable by flag/env.
- Last: Implemented dynamic key-based concurrency limits and mock tests.

## Log

Old sessions: [PROGRESS_ARCHIVE.md](./PROGRESS_ARCHIVE.md).

### Session 19: Dynamic Concurrency Limit

- Date: 2026-07-03
- Agent: Antigravity
- Goal: Implement key-aware concurrency limit defaulting (1 for demo/free keys, 3 for pro keys), overridable by CLI flag or env var.
- Constraints: None.
- Done:
  - Modified `concurrency` in clap CLI parser and `PipelineConfig` to be `Option<usize>` in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L58-L60).
  - Implemented automatic CoinGecko API key support (Demo/Pro base URL and headers) in [coingecko.rs](file:///c:/Users/lucas/Code/CDG/src/api/coingecko.rs#L22-L41).
  - Implemented default concurrency resolution logic in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L290-L302) based on key presence.
  - Updated interactive CLI menu default concurrency to adapt to key presence in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L915-L930).
  - Added unit test `test_default_concurrency_resolution` in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L1182-L1232).
- Blocked: None.
- Risk: None.
- Artifact: [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs), [coingecko.rs](file:///c:/Users/lucas/Code/CDG/src/api/coingecko.rs).
- Verification: `cargo test` passed 39/39 tests. `cargo clippy` passed with zero errors/warnings.
- Pending: None.

### Session 18: Implement Code Review Findings

- Date: 2026-07-03
- Agent: Antigravity
- Goal: Implement all fixes and structural refactors from the code review findings.
- Constraints: None.
- Done:
  - Added empty guard check to prevent panics in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L514-L518).
  - Excluded benchmarks from portfolio covariance/weights optimization, and added minimum weight constraint in [optimization.rs](file:///c:/Users/lucas/Code/CDG/src/optimization.rs#L160-L173).
  - Sanitized and validated input paths in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L442-L457).
  - Removed unconditional 3s sleep on cache misses in [coingecko.rs](file:///c:/Users/lucas/Code/CDG/src/api/coingecko.rs#L70-L77).
  - Refactored duplicate indicator logic in [analysis.rs](file:///c:/Users/lucas/Code/CDG/src/analysis.rs#L852-L974).
  - Prompted for concurrency and annualization override in interactive menu of [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L865-L885).
  - Updated `calculate_sharpe` signature in [backtest.rs](file:///c:/Users/lucas/Code/CDG/src/backtest.rs#L75-L95).
  - Cleaned up ticker pagination lifetimes in [coingecko.rs](file:///c:/Users/lucas/Code/CDG/src/api/coingecko.rs#L215-L225).
  - Added SQLite WAL cleanup helper to [api_tests.rs](file:///c:/Users/lucas/Code/CDG/tests/api_tests.rs#L4-L10).
  - Refactored pipeline flow arguments into `PipelineConfig` struct in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L259-L285).
  - Configured native `cmd /c cls` for terminal clearing on Windows in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L798-L808).
  - Added CSV and Parquet export unit tests in [export.rs](file:///c:/Users/lucas/Code/CDG/src/export.rs#L20-L48).
- Blocked: None.
- Risk: None.
- Artifact: [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs), [optimization.rs](file:///c:/Users/lucas/Code/CDG/src/optimization.rs), [coingecko.rs](file:///c:/Users/lucas/Code/CDG/src/api/coingecko.rs), [analysis.rs](file:///c:/Users/lucas/Code/CDG/src/analysis.rs), [backtest.rs](file:///c:/Users/lucas/Code/CDG/src/backtest.rs), [export.rs](file:///c:/Users/lucas/Code/CDG/src/export.rs), [api_tests.rs](file:///c:/Users/lucas/Code/CDG/tests/api_tests.rs).
- Verification: `cargo test` passed 38/38 tests. `cargo clippy` passed with zero errors/warnings.
- Pending: None.

### Session 17: Add New Items to Backlog

- Date: 2026-07-03
- Agent: Antigravity
- Goal: Add Markowitz portfolio weight issue and benchmarks separation issue to backlog for future analysis.
- Constraints: None.
- Done:
  - Added weight 0 selected coins issue and benchmarks comparison-only issue to [BACKLOG.md](file:///c:/Users/lucas/Code/CDG/.agents/BACKLOG.md#L13-L14).
- Blocked: None.
- Risk: None.
- Artifact: [BACKLOG.md](file:///c:/Users/lucas/Code/CDG/.agents/BACKLOG.md).
- Verification: File reviewed.
- Pending: None.

