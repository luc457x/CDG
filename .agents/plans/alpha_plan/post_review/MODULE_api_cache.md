# Module Review: `api/` + `cache.rs` + `utils.rs`

**Files:** `src/api/coingecko.rs`, `src/api/yahoo.rs`, `src/cache.rs`, `src/utils.rs`  
**Tests passing:** all cache, API mock, and path-safety tests green

## What is implemented well
- Coins list cache (11): 24h SQLite TTL; `check_coin_id` reuses cache.
- Retry hardening (12): 429/5xx/network retry with backoff, 30s timeout, max 4 attempts.
- `CoinResolution` enum (13): Exact/Ambiguous/NotFound; list lowercased once.
- Cache doc + miss log (19): `doc/api_cache.md` present; miss logged.
- Cache internals (23): Internal methods private, negative TTL guard, concurrent `check_cache_hits`, 10MB body cap.
- Path injection (27): `sanitize_name` + `validate_safe_path` wired before path construction.
- Sleep removed (28): No unconditional 3s delay remains.

## Remaining gaps / risks

| # | Severity | Gap | Evidence |
|---|----------|-----|----------|
| D1 | P2 | `sanitize_name` whitelist still deferred | `utils.rs:13-17` — only replaces `^`/`-`; plan backlog notes whitelist rewrite for beta |
| D2 | P2 | `validate_safe_path` allows empty string | `utils.rs:24-28` — empty `""` passes traversal check; could write to CWD |
| D3 | P3 | `check_coin_id` cache key uses plain URL (no `?cache=static`) | `coingecko.rs:225` — acceptable per min-viable clause, but differs from doc suggestion |

## Recommendations
1. Whitelist rewrite (D1) stays in backlog per plan.
2. Guard empty path in `validate_safe_path` (D2) — low effort, prevents CWD writes.
