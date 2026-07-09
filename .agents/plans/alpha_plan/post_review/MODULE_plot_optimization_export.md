# Module Review: `plot.rs` + `optimization.rs` + `export.rs`

**Files:** `src/plot.rs`, `src/optimization.rs`, `src/export.rs`, `Cargo.toml`  
**Tests passing:** plot, optimization, export tests green

## What is implemented well
- Candlestick charts (20): PNG + ASCII stdout + `candle_stdout` toggle.
- Dev debug (21): `Cargo.toml` `debug = 1`.
- README seed (22): Matches CLI default `1337`.
- Export hardening (24): `&DataFrame` signatures, `tempfile`, `validate_safe_path`.
- Optimization covariance (26): Unified annualization, `max_simulations` cap (50k), explicit RF rate (0.04 fallback + warning), min-weight bias documented + regression test, progress bar fixed.

## Remaining gaps / risks

| # | Severity | Gap | Evidence |
|---|----------|-----|----------|
| E1 | P3 | `export.rs` directory creation lacks context | `export.rs:7-9`, `:16-18` — `create_dir_all` failure propagates without `.with_context` |

## Recommendations
1. Add `.with_context` to `create_dir_all` (E1) — one-line improvement for debugability.
