# CDG Code Review

- Date: 2026-07-03
- Reviewer: Antigravity

Fast review of full codebase. Findings ordered severity: Critical → Warning → Minor.

---

## 🔴 Critical

### 1. `currency_dfs[0]` panics on empty result (main.rs L495)

```rust
let mut main_df = currency_dfs[0].clone();  // panics if all tasks errored
```

`join_set` errors return early, but if `currency_dfs` is empty the index panics.

**Fix:**
```rust
if currency_dfs.is_empty() {
    return Err(anyhow!("All data fetch tasks failed"));
}
let mut main_df = currency_dfs[0].clone();
```

---

### 2. Covariance annualization formula — near-singular matrix risk (optimization.rs L112)

```rust
cov_matrix[i][j] = daily_cov * (factors[i] * factors[j]).sqrt();
```

`sqrt(fi*fj)` is symmetric and diagonal-consistent (sqrt(f*f)=f), but when factors differ
significantly (365 vs 252) the off-diagonal scaling is inconsistent with standard finance
convention. This can produce a near-singular or poorly conditioned covariance matrix,
causing Markowitz weights to degenerate to 0 for selected coins.

Standard: `cov_ij_ann = cov_ij_daily * min(fi, fj)` or use a single unified factor.

This is the root of the zero-weight backlog issue.

---

## 🟡 Warning

### 3. Path injection via coin names (main.rs + utils.rs)

`validate_safe_path` and `sanitize_name` exist in `utils.rs` but are never called before
`std::fs::write` / `create_dir_all` in `run_pipeline_flow` or `run_ohlcv_flow`.
User-supplied coin IDs are interpolated directly into file paths.

```rust
let json_file_path = format!("{}/{}_{}.json", ohlcv_dir_clone, c, curr);
// c = "../../../etc/passwd" → path traversal
```

**Fix:** call `sanitize_name(c)` and `validate_safe_path` before building paths.

---

### 4. Hardcoded 3s delay on every cache miss (coingecko.rs L74)

```rust
tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
```

Fires on every single cache miss unconditionally — even single-coin queries.
With 5 benchmarks + 1 coin × 2 endpoints = 12 requests → minimum 36s of forced sleep.
Only needed to avoid 429, not as a blanket rule.

**Fix:** exponential backoff only on 429 response; remove upfront delay.

---

### 5. Duplicate indicator logic (analysis.rs)

`compute_indicators_raw` (L852) and `compute_returns_and_indicators` (L679) implement
the same EMA, RSI, OBV logic independently. Different fill strategies (`0.0` vs `Option<f64>`).
Bug fixed in one won't propagate to the other.

**Fix:** extract shared indicator primitives; keep one implementation.

---

### 6. Interactive menu misses concurrency and annualization options (main.rs L880)

```rust
run_pipeline_flow(..., 3, None).await  // concurrency, annualization_factor hardcoded
```

CLI exposes `--concurrency` and `--annualization-factor`. Interactive menu silently ignores both.

---

### 7. Hardcoded 365 annualization in backtest Sharpe (backtest.rs L94)

```rust
(mean / std_dev) * (365.0_f64).sqrt()
```

Ignores the asset-specific annualization factor added in optimization.rs.
`calculate_sharpe` should accept `annualization_factor: f64` as parameter.

---

## 🟠 Minor

### 8. Awkward lifetime workaround in `get_coin_tickers` (coingecko.rs L219)

`page_str` declared before `let query = if...` block to extend lifetime. Fragile pattern.
Refactor to `Vec<(&str, String)>` with owned values.

### 9. SQLite artifacts leaked in tests/ dir

18 SQLite files present including `-shm` / `-wal` WAL sidecars indicating tests that didn't
close cleanly. Should be `.gitignore`d at minimum.

### 10. `run_pipeline_flow` has 12 arguments (main.rs L265)

`#[allow(clippy::too_many_arguments)]` suppresses the clippy warning.
Extract into a `PipelineConfig` struct.

### 11. ANSI escape for terminal clear incompatible with legacy cmd.exe (main.rs L771)

```rust
print!("\x1B[2J\x1B[1;1H");
```

Works on Windows Terminal / PowerShell with VT100 enabled; fails silently on `cmd.exe`.

### 12. `export.rs` has no tests

Only two thin wrappers (`export_csv`, `export_parquet`). Worth adding at least a smoke test.

---

## ✅ Praise

- **Cache abstraction**: `CacheBackend` trait + `Arc<dyn CacheBackend>` — clean and testable.
- **Xorshift RNG**: no dep, deterministic, fast. Right call for Monte Carlo.
- **Parallel JoinSet + Semaphore**: correct pattern for concurrent CoinGecko fetching.
- **Offline sqlx macros**: `.sqlx/` dir enables compile-time query checks without live DB.
- **Forward + backward fill**: handles crypto/stock time gaps correctly in `align_datasets`.
- **Indicator null-scattering**: `valid_indices` + `scatter` closure avoids NaN propagation.
- **Test coverage**: 37 unit tests across modules. Good baseline.
- **`validate_safe_path` + `sanitize_name` exist** — just need wiring in.

---

## Summary Table

| # | Severity | File | Issue |
|---|----------|------|-------|
| 1 | 🔴 Critical | main.rs | Panic on `currency_dfs[0]` when all tasks fail |
| 2 | 🔴 Critical | optimization.rs | Covariance formula → near-singular matrix → zero weights |
| 3 | 🟡 Warning | main.rs / utils.rs | Path injection via coin names |
| 4 | 🟡 Warning | coingecko.rs | Hardcoded 3s delay on every cache miss |
| 5 | 🟡 Warning | analysis.rs | Duplicated indicator logic |
| 6 | 🟡 Warning | main.rs | Interactive menu missing concurrency/annualization |
| 7 | 🟡 Warning | backtest.rs | Hardcoded 365 annualization in Sharpe |
| 8 | 🟠 Minor | coingecko.rs | Awkward lifetime workaround for query params |
| 9 | 🟠 Minor | tests/ | Leaked SQLite WAL artifacts |
| 10 | 🟠 Minor | main.rs | 12-arg function → PipelineConfig struct |
| 11 | 🟠 Minor | main.rs | ANSI clear breaks on legacy cmd.exe |
| 12 | 🟠 Minor | export.rs | No tests |
