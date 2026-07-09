# CDG — Implementation Plan (Tracked Problems)

A new implementation agent executes this plan. All `file:line` references are verified against the current `dev` branch. Note: `src/pipeline.rs` and `src/ui.rs` are new on dev (not present on main); all other references below were re-verified against dev.

## Goal & scope

| # | Title | Status |
|---|-------|--------|
| P1 | `--light` conflict warning | **Actionable** |
| P2 | Timestamp boundary alignment (cache) | **Actionable** (verify + doc; keep daily pin) |
| P3 | Weekend gap alignment | **Actionable** (tests only) |
| P4 | Check math | **Actionable** (tests + 1 real fix) |
| P5 | Portfolio optimization optional (default ON) | **Actionable** |
| P7 | Plots optional (flag + env, default ON) + verify | **Actionable** |
| P11 | Candlestick OHLCV (PNG) + stdout ASCII | **Actionable** |
| P12 | Weekend forward-fill semantics | **Actionable** (tests only; already correct) |
| P8 | `--light` → generalist low-resource refactor | Deferred (Backlog) |
| P9 | Unified multi-source ingestion (Yahoo+CoinGecko ML) | Deferred (Backlog B1–B3) |
| P10 | ML-centered ADR | Deferred (Backlog) |

**Next run = P1, P2, P3, P4, P5, P7, P11, P12.** P8/P9/P10 are documented in Backlog for later.

---

## Verified context (read before coding)

- **Cache**: keyed by full canonical URL (`cache.rs:19-26`); TTL check at `cache.rs:77` (`now - cached_at_timestamp < ttl`). Rounding affects only the *URL*, not the TTL stamp.
- **Daily boundary rounding** already implemented: `pipeline.rs:109-127` and `:921-935` (`rounded_now = (now/86400)*86400`; `to=rounded_now`, `from=rounded_now - days*86400`). Makes same-UTC-day runs share one cache key.
- **`--light`**: forces `days=30` (`pipeline.rs:29-31`). Does **NOT** skip Markowitz (block `:424-477` has no `light` guard) and does **NOT** force 1 thread (concurrency from API key, `:45-57`). `main.rs:52` docstring wrongly says "forces coin=bitcoin".
- **Weekend fill** already correct: `analysis.rs:317-329` — non-`_volume` cols forward+backward fill (price = Friday close); `_volume` cols → `Zero` (volume = 0, not filled). Fill runs **before** the `drop_weekends` filter (`:331-347`).
- **Plotting**: 5 fns in `src/plot.rs` (line-chart, performance, risk-return, efficient-frontier, backtest-equity), all `plotters` `BitMapBackend` PNG (headless). Every pipeline call site wraps plotting in `if let Err(e)` warnings — a plot failure never aborts. `light` skips all plots (`:376`). Date/equity length alignment correct at all 4 backtest call sites (`dates[dates.len()-equity.len()..]`).
- **OHLC columns** present in `final_df` for every currency: `{col}_open/_high/_low/_close` (parsed at `pipeline.rs:275`, aligned at `pipeline.rs:277`). `plotters = "0.3"` → `Candlestick` available.
- **Yahoo client** (`src/api/yahoo.rs`) only `fetch_ticker_chart` (daily OHLCV), used **only as benchmarks** (`pipeline.rs:321`). No dividends/splits/fundamentals. **CoinGecko** (`src/api/coingecko.rs`) uses only daily `market_chart`/`ohlc` + display-only `tickers` — underutilized for ML (see Backlog P9).
- **`prep_ml`** (`analysis.rs:800`) scales every numeric column of `final_df` (minmax + std). Orderbook metrics never enter `final_df` (display-only, `pipeline.rs:148-201`), so excluded from ML.

---

## Actionable tasks

### P2 — Cache boundary (verify + doc; keep daily pin)
- Add test: two `now` values on the same UTC day at different times → identical `from`/`to` query params built by `coingecko::get_coin_market_chart_range`. Confirm `from` uses the same `rounded_now` anchor (yes).
- Document in `doc/api_cache.md`: boundary rounding purpose + TTL distinction. **Do not change rounding** (finer granularity is Backlog).

### P3 — Weekend alignment tests (`src/analysis.rs` test module)
No source change unless a test fails.
- T1: base with Sat+Sun rows + weekdays, `drop_weekends=true` → weekend rows absent, weekday rows retain forward-filled values.
- T2: `drop_weekends=false` → all rows present, weekend nulls forward-filled (no NaN).
- T3: `_volume` column → weekend volume null filled with **Zero**, not forward-filled.
- T4: crypto(7d) + stock(5d weekdays) join, `drop_weekends=true` → only weekdays, stock nulls forward-filled from Friday, no gaps.
- T5: unparseable date string → mask keeps row (`true`).
- T6: `parse_coingecko_market_chart` on synthetic 7-day JSON incl. a weekend → confirms weekend rows exist pre-align.

### P4 — Math tests + annualization fix
- **Tests** (`src/analysis.rs`, `src/backtest.rs` test modules): reference vectors for `calculate_sma`/`ema`/`rsi`/`macd`/`bollinger`/`atr`/`stochastic`/`adx`/`obv` (analysis.rs) and `calculate_sharpe` (backtest.rs:299), `calculate_max_drawdown` (backtest.rs:318). Returns are percent-scaled (`analysis.rs:705-706`); sharpe uses `((mean-rf)/std)*ann_factor.sqrt()` with sample std (n-1) — these are correct, lock them.
- **BUG FIX (annualization inconsistency)**:
  - `RunPipeline` backtest path uses `annualization_factor.unwrap_or(if drop_weekends {252.0} else {365.0})` (`pipeline.rs:431`, same at `:484` for optimization).
  - `run_standalone_backtest` (`pipeline.rs:894-906`) **hardcodes `365.0`** at `:1016` and has no `drop_weekends` param.
  - Fix: add `annualization_factor: Option<f64>` (or `drop_weekends: bool`) param to `run_standalone_backtest`; compute `let ann = annualization_factor.unwrap_or(if drop_weekends {252.0} else {365.0});` and pass `ann` to `run_backtest_for_asset` (replace literal `365.0`). Add `--drop-weekends` flag to the `Backtest` subcommand (`main.rs:93+`) and pass it through `main.rs:223-245`. Default (no flag) → `drop_weekends=false` → `365.0` (unchanged for crypto 7-day).

### P1 — `--light` conflict warning
- Add `warn_light_conflicts(config: &PipelineConfig) -> Vec<String>` in `pipeline.rs`; print each via `eprintln!` early in `run_pipeline_flow` (after the `:29-31` override, before timestamp calc).
- Detect overrides by comparing pre-override values: capture `original_days` before `config.days = 30`; warn if `original_days != 30`: `"--light overrides --days {n} -> 30"`. Multi-coin: `coin.split(',').filter(non-empty).count() > 1` → `"--light expects a single coin; {n} supplied, only first used"`.
- **Do NOT** enforce bitcoin-only. Fix the misleading docstring at `main.rs:52` (remove "forces coin=bitcoin").
- Unit test on the helper: warns for (multi-coin) and (days=90); no warn for (light=false) and (days=30).

### P5 — Optimization toggle (default ON)
- Add `optimize: bool` to `PipelineConfig` (`pipeline.rs:5-25`), default `true`.
- `RunPipeline` CLI (`main.rs` ~line 50): `#[arg(long, default_value_t = true, env = "CDG_OPTIMIZE")] optimize: bool` AND `#[arg(long = "no-optimize")] no_optimize: bool` (default false). Effective = `optimize && !no_optimize`.
- Gate the block at `pipeline.rs:426` as `if config.optimize && currency_cols.len() >= 2`. When skipped, print `"Skipping portfolio optimization (disabled via --no-optimize / CDG_OPTIMIZE=false)."`.
- Pass `optimize` through `main.rs:200` config construction.

### P7 — Plots toggle (flag + env, default ON) + verify
- Add `plots: bool` to `PipelineConfig` (`pipeline.rs:5-25`), default `true`.
- `RunPipeline` CLI: `#[arg(long, default_value_t = true, env = "CDG_PLOTS")] plots: bool` AND `#[arg(long = "no-plots")] no_plots: bool`. Effective = `plots && !no_plots`.
- Change gate at `pipeline.rs:376` from `if !light` to `if config.plots && !light`. Print skip message when disabled.
- Thread `plots` into `run_standalone_backtest`; gate the equity plot at `:1024-1039`. Add `--no-plots`/`CDG_PLOTS` to `Backtest` subcommand (`main.rs:93+`).
- **Tests** (`src/plot.rs` test module, render to `std::env::temp_dir()`, assert PNG signature `[137,80,78,71,13,10,26,10]`):
  - T1 `plot_line_chart` synthetic multi-col → valid PNG.
  - T2 `plot_performance` → valid PNG; assert first non-zero point ≈ 100 (base-100 normalization).
  - T3 `plot_risk_return` → valid for ≥2 assets; single asset still renders.
  - T4 `plot_efficient_frontier` synthetic → valid; empty points → `Err`.
  - T5 `plot_backtest_equity` synthetic; mismatched/longer dates → no OOB panic (formatter guards `idx < n_rows`).
  - T6 empty DataFrame → `Err` for each fn.
  - T7 all-null column → renders without panic.
  - T8 missing column name → `Err` gracefully.

### P11 — Candlestick OHLCV (PNG) + stdout ASCII
- `src/plot.rs`: add `plot_candlestick(df, open_col, high_col, low_col, close_col, title, output_path)` using plotters `Candlestick` (rise=GREEN, fall=RED), x = `0..n_rows` with `date` x-label formatter (mirror `plot_line_chart` `:95-104`); empty/missing cols → `Err`.
- `src/plot.rs` (or new `src/ui_text.rs`): add `print_candlestick_stdout(df, col, max_width)` — downsample to terminal width (default 80, fall back 80), block glyphs + ANSI color (rise green / fall red); guard single-row / all-flat.
- `src/pipeline.rs`: add `candle_stdout: bool` to `PipelineConfig`. In the `:376` plotting block, per `currency_cols`, write `{run_dir}/{col}_candlestick.png` gated by `config.plots && !light`. If `config.candle_stdout`, print ASCII candles to stdout per `currency_cols` (independent of `light`/`plots`).
- `src/main.rs`: add `--candle-stdout` to `RunPipeline` (optionally `Ohlcv` subcommand); pass through.
- **Tests**: `plot_candlestick` → valid PNG signature; `print_candlestick_stdout` → non-empty, contains glyphs, no panic on single-row/all-flat; missing OHLC col → `Err`.

### P12 — Weekend fill semantics (tests only; already correct)
- Add to `src/analysis.rs` test module:
  - T12a: crypto base (7d) LEFT JOINed with stock-like series (Fri close=100/vol=1000, null Sat/Sun) → assert Sat/Sun price == 100 AND volume == 0, for both `drop_weekends=false` (retained) and `true` (dropped; validate fill pre-drop in the non-dropping variant).
  - T12b: `{ticker}_volume` column with weekend nulls → asserts Zero branch (regression anchor for Backlog P9).
- No source change unless a test fails.
- **Backlog dependency note**: P9/B1 stock OHLCV volume columns MUST use `_volume` suffix (or fill must detect them) so they receive Zero-fill; otherwise stock volume would be wrongly forward-filled with Friday's volume.

---

## Deferred backlog (not this run)

- **P8 — `--light` generalist refactor**: force `concurrency=1`, skip Markowitz/optimization under light (align with P5 `optimize` default-off under light), skip most benchmarks + orderbook ticker fetch, skip plots (align with P7), single-coin focus, lower `days`. NOTE: today light does NOT skip Markowitz nor force 1 thread (additive). Open: force coin=bitcoin? which benchmarks to keep?
- **P9 — Unified multi-source ingestion** (phased):
  - **B1**: unified OHLCV for CoinGecko coins + Yahoo tickers; auto-detect source via local `get_coins_list` match (cached) then one Yahoo quote probe; precedence CG-id → Yahoo → error; cache positive resolutions ~1-day TTL under `resolve://<asset>` in existing `Cache`; on hard 404 invalidate + re-probe once + "renamed/delisted <24h" error; no negative caching; normalize both to `date,open,high,low,close,volume` + `source` tag.
  - **B2**: CoinGecko ML extras (hourly `market_chart`, `coins/markets`, `global`/`defi`), flag-gated (`--ml-features`).
  - **B3**: Yahoo dividends/splits/fundamentals (lowest priority).
- **P10 — ML-centered ADR**: create `doc/adr/0001-ml-centered-development.md` (standard ADR template) documenting that CDG is ML-first; references P9 schema, P6, toggles.

---

## Validation

- `cargo test` — all new + existing tests pass (P2, P3, P4, P7, P11, P12 tests).
- `cargo build` and `cargo clippy` — clean.
- Manual:
  - P1: `cdg collect --light --days 90 --coin bitcoin,ethereum` ⇒ two warnings.
  - P4: `cdg backtest --drop-weekends ...` uses 252; without flag 365.
  - P5: `cdg collect --coin btc,eth --no-optimize` (or `CDG_OPTIMIZE=false`) ⇒ optimization skipped with message.
  - P7: `cdg collect --coin btc,eth --no-plots` (or `CDG_PLOTS=false`) ⇒ no PNGs; `cdg backtest --no-plots` ⇒ no equity PNG.
  - P11: `cdg collect --coin btc,eth` ⇒ `{col}_candlestick.png`; `cdg collect --coin btc --candle-stdout` ⇒ ASCII candles.

## Decisions (resolved)

| Point | Decision |
|-------|----------|
| P2 | Keep daily (24h) pin; finer granularity → Backlog. |
| P3 | Verify + tests only; no source change unless test fails. |
| P4 | Verify + fix annualization inconsistency. |
| P5 | `--no-optimize` flag + `CDG_OPTIMIZE` env, default ON. |
| P7 | `--no-plots` flag + `CDG_PLOTS` env, default ON; effective `plots && !no_optimize/plots` and `plots && !light`. |
| P8 | Deferred; noted light does NOT currently skip Markowitz / force 1 thread. |
| P9 | Deferred; phased B1→B2→B3; auto-detect + cached resolution. |
| P10 | Deferred; concrete ADR deliverable. |
| P11 | PNG candlestick + flag-gated stdout ASCII. |
| P12 | Verified correct in code; add explicit scenario tests. |

## Risks / gotchas

- **P4 fix touches `run_standalone_backtest` signature** — update both the definition (`pipeline.rs:894`) and the call site (`main.rs:223-245`); keep default behavior (no flag → 365) to avoid regressions.
- **P5/P7 flag parsing**: clap bools with `env` parse `"true"/"false"`; `no_optimize`/`no_plots` are separate negation flags. Effective = `opt && !no_opt`.
- **P7 plots gate**: must stay `config.plots && !light` so `light` always wins (lightweight = no plots).
- **P11 candlestick x-axis**: use index `0..n_rows` with `date` labels (like existing plots), not raw date strings, to avoid categorical-axis issues.
- **P12 fill order matters**: fill runs before `drop_weekends`; do not reorder. Volume must use `_volume` suffix to hit Zero branch (P9 dependency).
