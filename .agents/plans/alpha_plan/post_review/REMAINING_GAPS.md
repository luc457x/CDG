# Remaining Gaps — Prioritized

**Date:** 2026-07-09  
**Source:** `.agents/plans/alpha_plan/post_review/` module reviews

## P1 — Should fix before alpha tag

| ID | Module | Gap | Effort |
|----|--------|-----|--------|
| G1 | analysis.rs | Add `data_quality_warnings: Vec<String>` return + test | Small |
| G2 | analysis.rs | Add MACD & ADX golden tests | Small |
| G3 | analysis.rs | Upgrade `test_returns_and_indicators_computation` (RSI bounded, Bollinger upper>lower, MACD hist=line−signal) | Small |
| G4 | backtest.rs | Remove/replace portfolio/treasury prediction placeholders (`prediction_accuracy: 0.0`, `active_win_rate: 0.0`, `prediction_rating: "n/a"`) | Small |
| G5 | pipeline.rs | Make final success message reflect actual status (plots/optimization skipped?) | Small |
| G6 | pipeline.rs | Add `annualization_factor: Option<f64>` to `run_standalone_backtest` | Small |

## P2 — Fix before beta

| ID | Module | Gap | Effort |
|----|--------|-----|--------|
| G7 | backtest.rs | Compute `prediction_r2` as strategy vs buy-and-hold (plan Option A) | Medium |
| G8 | backtest.rs | Use `IndexMap` or sorted keys for `BacktestReport` deterministic output | Small |
| G9 | analysis.rs | Extract shared indicator primitives (duplicate `compute_indicators_raw`) | Medium |
| G10 | analysis.rs | Remove unnecessary `.clone()` in `align_datasets` join | Small |
| G11 | analysis.rs | Vectorize `drop_weekends` with polars `str.strptime` + `dt.weekday()` | Medium |
| G12 | utils.rs | Guard empty string in `validate_safe_path` | Small |
| G13 | pipeline.rs | Reduce `run_pipeline_flow` to <400 LOC (continue extracting orchestration) | Medium |

## P3 — Backlog / polish

| ID | Module | Gap | Effort |
|----|--------|-----|--------|
| G14 | analysis.rs | `parse_coingecko_tickers` / `calculate_orderbook_metrics` — document or fix `unwrap_or(0.0)` semantics | Small |
| G15 | backtest.rs | Some equity tests use inequalities; convert to exact assertions where feasible | Small |
| G16 | export.rs | Add `.with_context` to `create_dir_all` calls | Small |
| G17 | utils.rs | Whitelist rewrite for `sanitize_name` (deferred per backlog) | Medium |
