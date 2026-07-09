# Progress (PROGRESS.md)

## Status

- State: Implementing Alpha plan; Phases 01, 02, and 03 implemented.
- Last: Fixed parser panic/aggregation, propagated OHLC null/NaN, added golden tests.

## Log

Old sessions: [PROGRESS_ARCHIVE.md](./PROGRESS_ARCHIVE.md).

### Session 37: Fix Parsers and Indicators

- Date: 2026-07-08
- Agent: Antigravity
- Goal: Fix aggregation & panic in CoinGecko/Yahoo parsers; propagate nulls/NaNs in indicators; add golden-value tests.
- Constraints: None.
- Done:
  - Fixed market chart OOB panic when prices outnumber volumes in [analysis.rs:27-32](file:///C:/Users/lucas/Code/CDG/src/analysis.rs#L27-L32).
  - Modified [analysis.rs](file:///C:/Users/lucas/Code/CDG/src/analysis.rs) to use `last` close of day and `mean` volume in market chart aggregation, and `first` open / `last` close in OHLC aggregation.
  - Added warning log skipped malformed OHLC rows in [analysis.rs:73-75](file:///C:/Users/lucas/Code/CDG/src/analysis.rs#L73-L75).
  - Preserved date continuity on null Yahoo prices in [analysis.rs:271-282](file:///C:/Users/lucas/Code/CDG/src/analysis.rs#L271-L282).
  - Gated advanced indicators (ATR, Stochastic, ADX, OBV) against missing OHLC/volume nulls, propagating `None`/`NaN` correctly and printing warnings to stderr in [analysis.rs:728-764](file:///C:/Users/lucas/Code/CDG/src/analysis.rs#L728-L764).
  - Filtered out `NaN`/`infinite` inputs in `prep_ml` in [analysis.rs:787-790](file:///C:/Users/lucas/Code/CDG/src/analysis.rs#L787-790).
  - Added 15 new test functions covering parser bug fixes, null propagation, indicator edge cases, zero variance, and golden value checks in [analysis.rs:1427-1744](file:///C:/Users/lucas/Code/CDG/src/analysis.rs#L1427-L1744).
- Blocked: None.
- Risk: None.
- Artifact: [analysis.rs](file:///C:/Users/lucas/Code/CDG/src/analysis.rs).
- Verification: `cargo test` — 66 tests passed.
- Pending: None.

### Session 36: Align Alpha Plan with code_review.md

- Date: 2026-07-08
- Agent: Antigravity
- Goal: Add the 4 uncovered `code_review.md` items as new alpha phases and fix alignment-audit inconsistencies in `.agents/etc/alpha_plan/`.
- Constraints: Plan claimed `.agents/etc/alpha_plan/` was write-protected; actual environment granted write access, so no workaround needed.
- Done:
  - Created `26.fix_optimization_covariance.md` (Critical covariance annualization + Monte Carlo hardening, source `code_review.md` #2).
  - Created `27.fix_path_injection.md` (wire existing `sanitize_name`/`validate_safe_path`, source #3).
  - Created `28.fix_coingecko_sleep.md` (remove blanket 3s cache-miss sleep, source #4).
  - Created `29.fix_menu_concurrency_annualization.md` (thread menu options, source #6).
  - `README.md`: added `code_analysis_2026-07-04.md` + `code_review.md` to Sources; appended `26 → 27 → 28 → 29` execution order; bumped pre-alpha polish range `09–25` → `09–29` with Critical note.
  - `25.add_pipeline_currency_assert.md`: added parallel `currency_dfs[0]` (`main.rs:495`) safety-rail note.
  - `19.add_cache_boundary_doc.md`: added same-UTC-day cache-key unit test task + validation.
  - `10.fix_annualization_inconsistency.md`: added out-of-scope Note for `calculate_sharpe` hardcoded `365.0` (`code_review.md` #7).
  - `backlog.md`: parked PHASE 8 alignment hardening (`align_datasets` clone, `drop_weekends` vs config, `parse_coingecko_tickers` `unwrap_or(0.0)`).
- Blocked: None.
- Risk: Plan's stated write-protection did not apply; doc edits only, low regression risk.
- Artifact: `.agents/etc/alpha_plan/26.fix_optimization_covariance.md`, `27.fix_path_injection.md`, `28.fix_coingecko_sleep.md`, `29.fix_menu_concurrency_annualization.md`, `README.md`, `25.add_pipeline_currency_assert.md`, `19.add_cache_boundary_doc.md`, `10.fix_annualization_inconsistency.md`, `backlog.md`.
- Verification: Confirmed all 4 new phase files exist on disk; re-read edited files to confirm content applied.
- Pending: PHASE 7 (structured logging) and PHASE 8 (alignment hardening) remain deferred/backlogged; code_review items #1 (`main.rs:495` panic) and #5 (`analysis.rs` duplicated indicators) left to backlog per plan.
