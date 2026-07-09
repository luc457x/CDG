# Progress (PROGRESS.md)

## Status

- State: Implementing Alpha plan; Phases 01–25 implemented.
- Last: Implemented cache boundary doc, candlestick PNG/ASCII, Cargo dev debug info, seed doc fix, cache backend hardening, export signature and path hardening, and pipeline empty checks.

## Log

Old sessions: [PROGRESS_ARCHIVE.md](./PROGRESS_ARCHIVE.md).

### Session 44: Complete Alpha Phases 19-25

- Date: 2026-07-09
- Agent: Antigravity
- Goal: Implement alpha phases 19 to 25.
- Constraints: None.
- Done:
  - **Phase 19**: Documented daily boundary rounding, canonical URL cache key, eviction rule, and endpoint TTLs in [doc/api_cache.md](file:///C:/Users/lucas/Code/CDG/doc/api_cache.md) and linked from [README.md](file:///C:/Users/lucas/Code/CDG/README.md). Logged cache misses in [src/api/coingecko.rs](file:///C:/Users/lucas/Code/CDG/src/api/coingecko.rs). Added test `test_coingecko_market_chart_range_same_day_alignment` in [tests/api_tests.rs](file:///C:/Users/lucas/Code/CDG/tests/api_tests.rs).
  - **Phase 20**: Added `plot_candlestick` (PNG) and `print_candlestick_stdout` (ASCII) in [src/plot.rs](file:///C:/Users/lucas/Code/CDG/src/plot.rs). Added `--candle-stdout` flag to `RunPipeline` and `Ohlcv` commands in [src/main.rs](file:///C:/Users/lucas/Code/CDG/src/main.rs), wired through [src/pipeline.rs](file:///C:/Users/lucas/Code/CDG/src/pipeline.rs) and [src/ui.rs](file:///C:/Users/lucas/Code/CDG/src/ui.rs). Added tests `test_plot_candlestick_and_stdout` and `test_print_candlestick_stdout_flat_and_single_row` in [tests/pipeline_tests.rs](file:///C:/Users/lucas/Code/CDG/tests/pipeline_tests.rs).
  - **Phase 21**: Added comment and set `debug = 1` for dev profile in [Cargo.toml](file:///C:/Users/lucas/Code/CDG/Cargo.toml) for backtraces.
  - **Phase 22**: Updated seed default documentation to `1337` in [README.md](file:///C:/Users/lucas/Code/CDG/README.md).
  - **Phase 23**: Hardened [src/cache.rs](file:///C:/Users/lucas/Code/CDG/src/cache.rs): internal helper methods made private, negative TTL guard, 10MB response limit, concurrent hits checking via `futures::future::join_all`. Added `futures = "0.3"` to [Cargo.toml](file:///C:/Users/lucas/Code/CDG/Cargo.toml). Added test `test_cache_negative_ttl` and `test_cache_max_body_bytes` in `src/cache.rs`.
  - **Phase 24**: Changed signatures to `&DataFrame` in [src/export.rs](file:///C:/Users/lucas/Code/CDG/src/export.rs) and callers in [src/pipeline.rs](file:///C:/Users/lucas/Code/CDG/src/pipeline.rs). Added path safety validation using `utils::validate_safe_path`. Added anyhow context to parent directory creation. Isolated tests using `tempfile::tempdir`.
  - **Phase 25**: Implemented empty dataset safety rails in [src/pipeline.rs](file:///C:/Users/lucas/Code/CDG/src/pipeline.rs) to return clean errors on fetch failures instead of out-of-bounds panics. Added test `test_pipeline_flow_all_coins_fail` in [tests/pipeline_tests.rs](file:///C:/Users/lucas/Code/CDG/tests/pipeline_tests.rs).
- Blocked: None.
- Risk: None.
- Artifact: [src/cache.rs](file:///C:/Users/lucas/Code/CDG/src/cache.rs), [src/export.rs](file:///C:/Users/lucas/Code/CDG/src/export.rs), [src/pipeline.rs](file:///C:/Users/lucas/Code/CDG/src/pipeline.rs), [src/plot.rs](file:///C:/Users/lucas/Code/CDG/src/plot.rs), [src/main.rs](file:///C:/Users/lucas/Code/CDG/src/main.rs), [src/ui.rs](file:///C:/Users/lucas/Code/CDG/src/ui.rs), [Cargo.toml](file:///C:/Users/lucas/Code/CDG/Cargo.toml), [README.md](file:///C:/Users/lucas/Code/CDG/README.md), [doc/api_cache.md](file:///C:/Users/lucas/Code/CDG/doc/api_cache.md), [tests/api_tests.rs](file:///C:/Users/lucas/Code/CDG/tests/api_tests.rs), [tests/pipeline_tests.rs](file:///C:/Users/lucas/Code/CDG/tests/pipeline_tests.rs).
- Verification: `cargo test` — 108 tests passed.
- Pending: None.

### Session 43: Implement --light Warnings & Weekend Tests

- Date: 2026-07-09
- Agent: Antigravity
- Goal: Implement alpha phases 16-18.
- Constraints: None.
- Done:
  - **Phase 16**: Added `pub fn warn_light_conflicts` in [src/pipeline.rs](file:///C:/Users/lucas/Code/CDG/src/pipeline.rs) to check `--light` conflicts and print warnings via `eprintln!` in `run_pipeline_flow`. Removed misleading "forces coin=bitcoin" from `--light` docstring in [src/main.rs](file:///C:/Users/lucas/Code/CDG/src/main.rs). Added `test_warn_light_conflicts` in [tests/pipeline_tests.rs](file:///C:/Users/lucas/Code/CDG/tests/pipeline_tests.rs).
  - **Phase 17**: Added tests `test_weekend_alignment_t1` through `test_weekend_alignment_t6` in [src/analysis.rs](file:///C:/Users/lucas/Code/CDG/src/analysis.rs) validating weekend alignment and drop logic.
  - **Phase 18**: Added tests `test_weekend_fill_t12a` and `test_weekend_fill_t12b` in [src/analysis.rs](file:///C:/Users/lucas/Code/CDG/src/analysis.rs) validating weekend volume zero-fill.
- Blocked: None.
- Risk: None.
- Artifact: [src/pipeline.rs](file:///C:/Users/lucas/Code/CDG/src/pipeline.rs), [src/main.rs](file:///C:/Users/lucas/Code/CDG/src/main.rs), [src/analysis.rs](file:///C:/Users/lucas/Code/CDG/src/analysis.rs), [tests/pipeline_tests.rs](file:///C:/Users/lucas/Code/CDG/tests/pipeline_tests.rs).
- Verification: `cargo test` - 102 passed.
- Pending: None.
