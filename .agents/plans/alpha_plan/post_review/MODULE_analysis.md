# Module Review: `analysis.rs`

**File:** `src/analysis.rs` (2030 lines)  
**Tests passing:** `cargo test -p cdg analysis::` green

## What is implemented well
- Parser panic fixes (01): `total_volumes.get(i)` safe access, last-close aggregation, first/last OHLC, malformed-row warning, yahoo null continuity.
- Null propagation (02): `highs_raw`/`lows_raw`/`volumes_raw` collected as `Vec<Option<f64>>`; ATR/Stoch/ADX/OBV emit `None` on missing OHLC.
- `prep_ml` (02): `is_finite()` guard + documented `std=1.0` fallback.
- Weekend tests (17/18): T1–T6 + T12a/T12b all present and passing.
- Indicator math (04b/04c): SMA/EMA/RSI/MACD/Bollinger/ATR/Stoch/ADX/OBV implementations are mathematically correct.

## Remaining gaps / risks

| # | Severity | Gap | Evidence |
|---|----------|-----|----------|
| A1 | P1 | `data_quality_warnings: Vec<String>` not returned or tested | Plan 02 validation requires warning contents asserted. Only `eprintln!` exists. |
| A2 | P1 | MACD & ADX golden tests missing | No `test_macd_golden` / `test_adx_golden` in `#[cfg(test)]` |
| A3 | P1 | `test_returns_and_indicators_computation` not upgraded | Still only column-existence checks; missing RSI bounded, Bollinger upper>lower, MACD hist=line−signal assertions |
| A4 | P2 | Duplicate indicator logic (`compute_indicators_raw` vs `compute_returns_and_indicators`) | `analysis.rs:852` and `:679` duplicate EMA/RSI/OBV; divergence risk |
| A5 | P2 | `align_datasets` clones `other_df` before `.lazy()` | `analysis.rs:301` — unnecessary clone of full DataFrame on every join |
| A6 | P3 | `drop_weekends` uses Rust-loop date parsing | `analysis.rs:332-344` — 10k string allocations; polars `str.strptime`+`dt.weekday()` is vectorized |
| A7 | P3 | `parse_coingecko_tickers` / `calculate_orderbook_metrics` still `unwrap_or(0.0)` | `analysis.rs:135-137`, `:160,165,170` — silent null poisoning in orderbook display data |

## Recommendations
1. Add `data_quality_warnings` return + test (A1).
2. Add MACD/ADX golden vectors + upgrade `test_returns_and_indicators_computation` (A2/A3).
3. Extract shared indicator primitives to eliminate A4 duplication.
4. Drop `.clone()` in A5 and vectorize A6 when touching the module next.
