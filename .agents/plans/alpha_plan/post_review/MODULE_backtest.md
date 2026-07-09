# Module Review: `backtest.rs`

**File:** `src/backtest.rs` (2291 lines)  
**Tests passing:** `cargo test -p cdg backtest::` green (27 tests)

## What is implemented well
- Neutral bug fix (04): Explicit `prev_position`/`new_position: i32`; fee on transition; return signed by `prev_position`; deterministic `test_neutral_exit_fees_pnl`.
- Metrics fixes (05): `calculate_r2` perfect-prediction guard; `calculate_max_drawdown` `peak != 0.0`; tests for both.
- Equity tests (06): RSI, MACD, Bollinger, custom JSON, weekly/monthly portfolio, edge cases (single bar, all-None, zero volume).
- `prediction_r2` computed in single-asset path.

## Remaining gaps / risks

| # | Severity | Gap | Evidence |
|---|----------|-----|----------|
| B1 | P1 | Portfolio/treasury `BacktestMetrics` still hardcode prediction placeholders | `backtest.rs:1329-1332`, `:1059-1062` — `prediction_accuracy: 0.0`, `active_win_rate: 0.0`, `prediction_rating: "n/a"`. Plan 05 requires removal/`NotApplicable`. |
| B2 | P2 | `prediction_r2` computed as actual vs strategy, not strategy vs buy-and-hold | `backtest.rs:806` — plan 05 Option A specifies B&H comparison |
| B3 | P2 | `BacktestReport` uses non-deterministic `HashMap` ordering | `backtest.rs:33` — iteration order flaky; suggest `IndexMap` or sorted keys |
| B4 | P3 | Some equity tests assert inequalities, not exact values | `test_macd_strategy_equity_exact`, `test_bollinger...` use `>` / `== 1` rather than exact float assertions |

## Recommendations
1. Drop or replace portfolio/treasury prediction placeholders (B1).
2. If keeping `prediction_r2`, compute as strategy-vs-B&H per plan (B2).
3. Use `IndexMap` for deterministic report output (B3).
