# Progress (PROGRESS.md)

## Status

- State: /ping endpoints cached to prevent CoinGecko and Yahoo Finance API rate limits.
- Last: Cached connection ping requests for both CoinGecko and Yahoo Finance API clients.

## Log

Old sessions: [PROGRESS_ARCHIVE.md](./PROGRESS_ARCHIVE.md).

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
