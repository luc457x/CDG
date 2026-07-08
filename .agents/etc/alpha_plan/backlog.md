# Backlog — Deferred out of Alpha Plan

**Date:** 2026-07-08  
**Rule:** Do not start any item here until `v0.1.0-alpha` is tagged and stable.

---

## Explicitly Deferred (from `alpha_readiness_plan_2026-07-07.md`)

| Item | Reason deferred |
|------|----------------|
| **P8 — `--light` generalist refactor** | Large scope: forces `concurrency=1`, skips Markowitz, skips plots, skips benchmarks, single-coin focus. Currently `--light` only overrides `days=30`. Full refactor needs its own plan. |
| **P9/B1 — Unified multi-source ingestion** | Phased multi-source OHLCV auto-detection (CoinGecko ↔ Yahoo) with `resolve://` caching. Large feature, not alpha-blocking. |
| **P9/B2 — CoinGecko ML extras** | Hourly `market_chart`, `coins/markets`, `global`/`defi` flag-gated features. |
| **P9/B3 — Yahoo dividends/splits/fundamentals** | Lowest priority; no alpha use case requires it today. |
| **P10 — ML-centered ADR** | Documentation task (`doc/adr/0001-ml-centered-development.md`). Valuable but not release-blocking. |

---

## Not Planned (explicitly backlogged from review findings)

These items appeared in `line_by_line_review.md` or `code_review.md` but were judged too large or out-of-scope for alpha:

| Item | Rationale |
|------|-----------|
| **Structured logging (`tracing`/`env_logger`)** | Medium/large scope: logger setup, `--verbose`/`--quiet` flags, structured `cdg.log` writer. Good for beta, not alpha. |
| **`ui.rs` session state / `config.json` logging** | Reproducibility improvement. Alpha users can rerun same CLI args; full config serialization is a beta feature. |
| **Configurable benchmark ticker list** | Hardcoded `["^GSPC", "^DJI", "^IXIC", "^HSI", "^BVSP", "^TNX"]` works; failure is handled gracefully. Configurable list is a feature, not a bug fix. |
| **`analysis.rs` / `backtest.rs` god-function splitting** | Large refactor beyond `extract_shared_backtest_report`. Splitting `compute_returns_and_indicators` and `run_backtest_for_asset` into smaller fns improves maintainability but is not required for alpha correctness. |
| **`sanitize_name` whitelist rewrite** | Moving from blacklist to `[a-z0-9_-]` whitelist is a security hygiene improvement, not alpha-blocking. Current `validate_safe_path` already blocks traversal. |
| **`ui.rs` ANSI clear / input validation / `PipelineConfig` lifetime fixes** | Polish items. Do not block alpha. |
| **Atomic writes in `export.rs`** | Crash-safety improvement. For alpha, `File::create` truncation is acceptable. Move to tempfile+rename after alpha. |
| **`cache.rs` concurrent `check_cache_hits` + size-limit guard** | Concurrency improvement in a diagnostic function; not alpha-blocking. |
| **`plot.rs` non-black single-series color / `y_max == y_min` padding** | Visual edge cases. Do not block alpha. |
| **Replace custom `Xorshift` with `rand` crate** | Current RNG is deterministic and adequate for 10k sims. Swap is a refactor, not a bug fix. |
| **Remove `#[allow(clippy::type_complexity)]` in MACD/Bollinger** | Design improvement. Suppressing the warning is fine for alpha; struct return types are a refactor. |
| **PHASE 8 — alignment hardening** | `align_datasets` unnecessary clone, `drop_weekends` inconsistency vs config, `parse_coingecko_tickers` `unwrap_or(0.0)` (`alpha_readiness_plan_2026-07-07.md` PHASE 8). Parked so the gap isn't lost; revisit for beta. |

---

## Exit Criteria

Do not move items from Backlog into the Alpha Plan unless:
1. An alpha blocker is discovered that requires the backlog item, or
2. The item is rescheduled by explicit user decision.

After `v0.1.0-alpha` ships, schedule the high-priority backlog items (P9/B1, structured logging, config.json) for beta.
