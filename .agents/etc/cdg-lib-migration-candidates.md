# CDG Lib Migration Candidates

> **Purpose**: Guide for an agent to audit CDGonGCP source, identify domain logic
> duplicated or misplaced here, and move it upstream into the CDG library
> (`https://github.com/luc457x/CDG.git`, branch `main`).
>
> **Scope**: Read-only analysis of CDGonGCP → PRs/commits go to CDG repo.
> Do NOT modify CDGonGCP source until CDG lib version is published and
> CDGonGCP's `Cargo.toml` dependency is updated.

---

## Repo Context

| Item | Value |
|---|---|
| CDGonGCP root | `c:\Users\lucas\Code\CDGonGCP` |
| CDG lib dep | `cdg = { git = "https://github.com/luc457x/CDG.git", branch = "main" }` |
| CDG lib import | `use cdg::{analysis, api, cache, export, plot, optimization};` |
| CDGonGCP src | `c:\Users\lucas\Code\CDGonGCP\src\` |

---

## Candidates (priority order)

### 1. `calculate_indicators` — `historical.rs` [HIGH]

**File**: `src/historical.rs` lines 22–200 (approx)

**What it is**: Full re-implementation of technical indicators on raw `Vec<f64>`:
- Simple returns, log returns
- EMA(12), EMA(26)
- MACD, MACD signal, MACD histogram
- RSI(14)
- OBV (On-Balance Volume)

**Problem**: CDG lib already has `analysis::compute_returns_and_indicators` that does
the same on a Polars `DataFrame`. This version operates on raw vectors, creating
two diverging implementations of the same domain logic.

**Signature** (current in CDGonGCP):
```rust
fn calculate_indicators(
    prices: &[f64],
    volumes: &[f64],
) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>)
// returns: simple_returns, log_returns, ema_12, ema_26, macd, macd_signal, macd_histogram, rsi_14, obv
```

**Called from**: `historical.rs` (internal — used when building historical Parquet data from raw API responses before DataFrame construction).

**Action for CDG lib**:
- Add a `analysis::compute_indicators_raw(prices: &[f64], volumes: &[f64]) -> IndicatorResult`
  or equivalent struct-based API.
- `IndicatorResult` should expose named fields (not a tuple) to avoid positional errors.
- Keep `compute_returns_and_indicators` on Polars DF as-is; the raw version is a lower-level
  sibling for streaming/historical ingestion scenarios.

---

### 2. `validate_safe_path` — duplicated 4×  [HIGH]

**Files and line ranges**:
- `src/main.rs` line 589–597
- `src/gcs.rs` line 9–17
- `src/historical.rs` line 12–20
- `src/backtest.rs` line 38–46

**What it is**: Path traversal guard — rejects paths containing `..` components.

```rust
fn validate_safe_path(path: &str) -> Result<()> {
    let p = std::path::Path::new(path);
    for component in p.components() {
        if let std::path::Component::ParentDir = component {
            return Err(anyhow!("Path traversal detected in path: {}", path));
        }
    }
    Ok(())
}
```

**Action for CDG lib**:
- Add to `cdg::utils` (or create module if not exists): `pub fn validate_safe_path(path: &str) -> Result<()>`
- CDGonGCP all 4 call sites replace local fn with `cdg::utils::validate_safe_path`.

---

### 3. `sanitize_name` — duplicated 2× [HIGH]

**Files and line ranges**:
- `src/main.rs` line 599–603
- `src/backtest.rs` line 48–52

**What it is**: Replaces `^` and `-` with `_` in ticker/column names for Polars compatibility.

```rust
fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|ch| if ch == '^' || ch == '-' { '_' } else { ch })
        .collect()
}
```

**Action for CDG lib**:
- Add to `cdg::utils`: `pub fn sanitize_name(name: &str) -> String`
- Used everywhere ticker names become DataFrame column names.

---

### 4. Backtest math — `backtest.rs` [MEDIUM]

**File**: `src/backtest.rs`

**What it is**: Pure financial/ML metrics with zero GCP dependency:
- `BacktestMetrics` struct (lines 12–30) — Sharpe, max drawdown, returns, MDA, R², confusion matrix counts
- `BacktestReport` struct (lines 32–36)
- `calculate_mda` (lines 54–79) — Mean Directional Accuracy
- Sharpe ratio computation, max drawdown computation (embedded in run_backtest fn)

**Why candidate**: Pure domain math — no cloud, no IO, only `polars` + arithmetic.
Reusable for any backtesting scenario in CDG.

**Structs to move**:
```rust
pub struct BacktestMetrics { /* coin, currency, returns, sharpe, drawdown, accuracy, r2, win_rate, rating, confusion */ }
pub struct BacktestReport  { /* timestamp, metrics: HashMap<String, BacktestMetrics> */ }
pub fn calculate_mda(actuals: &[f64], predicted: &[f64]) -> f64
```

**Action for CDG lib**:
- Create `cdg::backtest` module (or `cdg::metrics`) exposing structs + `calculate_mda`.
- Sharpe/drawdown helpers can be private or exported depending on CDG API design.
- CDGonGCP `backtest.rs` imports types from `cdg::backtest`, keeps GCS upload / file IO logic locally.

---

### 5. `check_cache_hits` — `main.rs` [LOW]

**File**: `src/main.rs` lines 1544–1560

**What it is**: Utility that checks a list of URLs against a `CacheBackend` and returns
`(hits, total)`. Operates on `Arc<dyn CacheBackend>` from CDG.

```rust
async fn check_cache_hits(
    cache: Arc<dyn CacheBackend>,
    urls: &[String],
    ttl_secs: i64,
) -> Result<(usize, usize)>
```

**Action for CDG lib**:
- Add to `cdg::cache` as a free function or method on `CacheBackend` trait.
- Low priority — thin helper, easy to keep local. Only migrate if CDG cache module grows.

---

## Implementation Order

1. `validate_safe_path` + `sanitize_name` — zero risk, pure utils, easiest PR
2. `calculate_indicators` raw vec version — creates `analysis::compute_indicators_raw`
3. `BacktestMetrics` / `calculate_mda` — creates `cdg::backtest` module skeleton
4. `check_cache_hits` — sweep up the rest

---

## Files to Read Before Implementing

| File | Why |
|---|---|
| `src/historical.rs` full | See how `calculate_indicators` is called and what data flows into it |
| `src/backtest.rs` full | Full backtest logic to extract the pure math |
| `src/main.rs` lines 589–603 | `validate_safe_path` + `sanitize_name` originals |
| `src/gcs.rs` lines 9–17 | `validate_safe_path` copy |
| CDG lib `src/analysis.rs` | Check existing `compute_returns_and_indicators` signature to avoid API clash |
| CDG lib `src/` root | Understand existing module structure before adding new ones |

---

## Notes

- CDGonGCP uses `anyhow::Result` everywhere. CDG lib should match.
- Polars version in CDGonGCP: `0.36`. Check CDG lib version for compat before adding Polars-based APIs.
- `calculate_indicators` raw version used in historical ingestion (not Polars context) — keep both APIs.
- After CDG lib PR merged, bump CDGonGCP `Cargo.toml` git dep and replace local fns.
