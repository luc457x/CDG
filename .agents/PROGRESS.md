# Progress (PROGRESS.md)

## Status

- State: Backlog fully implemented: WAL performance settings, offline compile-time verification, parallel CoinGecko ingestion, and asset-specific annualization.
- Last: Completed backlog implementation, added unit tests, verified offline compilations, and updated specs.

## Log

Old sessions: [PROGRESS_ARCHIVE.md](./PROGRESS_ARCHIVE.md).

### Session 16: Implement Backlog Items

- Date: 2026-07-03
- Agent: Antigravity
- Goal: Implement all items from the backlog: WAL performance settings, compile-time macros offline metadata, parallel ingestion, and asset-specific annualization.
- Constraints: None.
- Done:
  - Enabled WAL mode and Normal synchronization in [cache.rs](file:///c:/Users/lucas/Code/CDG/src/cache.rs#L38-L42).
  - Replaced runtime SQL queries with compile-time checked `sqlx::query!` macros in [cache.rs](file:///c:/Users/lucas/Code/CDG/src/cache.rs#L55-L94).
  - Generated `.sqlx/` offline metadata directory in the project root to support offline compilation.
  - Implemented parallel CoinGecko charts/OHLC data fetching using `tokio::task::JoinSet` and `tokio::sync::Semaphore` in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L420-L490).
  - Added CLI flag `--concurrency` (env `COINGECKO_CONCURRENCY`) to control concurrent requests.
  - Updated [optimization.rs](file:///c:/Users/lucas/Code/CDG/src/optimization.rs#L44-L185) to dynamically scale returns and covariance matrices using asset-specific annualization factors.
  - Added heuristic logic to classify asset class (Crypto -> 365, Stocks -> 252) and added CLI flag `--annualization-factor` (env `ANNUALIZATION_FACTOR`) to override all factors to a single custom value.
  - Added unit test `test_asset_specific_annualization` in [optimization.rs](file:///c:/Users/lucas/Code/CDG/src/optimization.rs#L353-L373) and updated existing test files.
  - Cleared all implemented items from [BACKLOG.md](file:///c:/Users/lucas/Code/CDG/.agents/BACKLOG.md).
  - Updated specifications in [SPEC.md](file:///c:/Users/lucas/Code/CDG/.agents/SPEC.md).
- Blocked: None.
- Risk: CoinGecko free API rate limit (429) might trigger if concurrency is set too high.
- Artifact: [cache.rs](file:///c:/Users/lucas/Code/CDG/src/cache.rs), [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs), [optimization.rs](file:///c:/Users/lucas/Code/CDG/src/optimization.rs), [BACKLOG.md](file:///c:/Users/lucas/Code/CDG/.agents/BACKLOG.md), [SPEC.md](file:///c:/Users/lucas/Code/CDG/.agents/SPEC.md), [walkthrough.md](file:///C:/Users/lucas/.gemini/antigravity-ide/brain/728f513f-51b6-4f83-aa45-257d1328dfe4/walkthrough.md).
- Verification: `$env:SQLX_OFFLINE="true"; cargo test` passed 37 tests.
- Pending: None.

### Session 15: Add Customizable Cache TTL

- Date: 2026-07-01
- Agent: Antigravity
- Goal: Add configurable cache TTL command-line flag and interactive menu option.
- Constraints: None.
- Done:
  - Added `--cache-ttl` global command-line argument to `Cli` in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L17-L19).
  - Configured CoinGecko and Yahoo Finance API clients to use custom cache TTL.
  - Added "Configure Cache TTL" option to the interactive console menu in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L931-L941).
  - Fixed clippy redundant field warning in [optimization.rs](file:///c:/Users/lucas/Code/CDG/src/optimization.rs#L212).
  - Updated documentation in [installation_usage.md](file:///c:/Users/lucas/Code/CDG/doc/installation_usage.md) and [api_cache.md](file:///c:/Users/lucas/Code/CDG/doc/api_cache.md).
  - Added CLI parser unit test `test_cli_parsing_cache_ttl` in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L981-L986).
- Blocked: None.
- Risk: Short TTL values (e.g. <30s) could increase HTTP 429 rate limit errors from CoinGecko under high traffic.
- Artifact: [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs), [optimization.rs](file:///c:/Users/lucas/Code/CDG/src/optimization.rs), [installation_usage.md](file:///c:/Users/lucas/Code/CDG/doc/installation_usage.md), [api_cache.md](file:///c:/Users/lucas/Code/CDG/doc/api_cache.md).
- Verification: `cargo test` passed 28/28 tests; `cargo clippy` and `cargo fmt` passed with zero errors/warnings.
- Pending: None.
