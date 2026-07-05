# Line-by-Line Code Review
**Generated**: 2026-07-05
**Scope**: Complete line-by-line review of CDG Rust codebase
**Status**: In progress - phases appended sequentially

---

# Phase 1: lib.rs + utils.rs

## File: src/lib.rs (171 bytes, 10 lines)

### Structural Review

**Lines 1-10: Module declarations**
- `pub mod` exposes all submodules and their internals. Any consumer can reach `cdg::api::coingecko::...` directly, leaking implementation details. Consider adding `pub use` re-exports for stable public API surface.
- Module order is arbitrary and does not reflect logical dependency layers (utilities last despite being foundational).

---

## File: src/utils.rs (1111 bytes, 37 lines)

### Line-by-Line Issues

**Line 4: `let p = std::path::Path::new(path);`**
- Accepts raw `&str` without ownership or length checks.
- Empty input `""` produces a valid `Path` with zero components. The loop on line 5 does nothing, returning `Ok(())`. If used for file creation, this could write to the current working directory. No guard against empty input.

**Lines 5-9: Traversal check**
- The check iterates `path.components()` after filesystem normalization. For sequence `"safe/../path"`, `Path` normalizes to `"path"`, yielding `Component::Normal("path")`, so the check passes. Whether this is acceptable depends on whether the function intends to reject malformed input or only unsafe normalized paths. As written, **it only blocks unnormalized traversal**, not malformed input that normalizes safely.
- Does not handle Windows verbatim prefixes (`\\?\C:\file`). `components()` treats the prefix as `Prefix`, which is not flagged. On Windows this is usually safe, but the function is platform-naive.
- **Dead path**: Does not check `CurDir` (`.`). Not a traversal vector, but if strictness is desired, it is ignored.

**Line 7: Error message**
- Returns `anyhow!` with the raw path. If the path contains sensitive info, this leaks it into logs. Fine for internal CLI tool, but worth noting.

**Line 13-17: `sanitize_name`**
- **Missing sanitizers**: Only replaces `^` and `-`. Does not truncate. Does not strip or reject `/`, `\`, `:`, `*`, `?`, `"`, `<`, `>`, `|`, null bytes, or control characters.
  - On Unix, `/` would create nested directories if the string is used as a filename.
  - On Windows, `*`, `?`, `|`, etc. are illegal in filenames.
  - Null bytes (`\0`) in Rust `&str` are technically prohibited by Rust string invariants, but if the source is ever untrusted (e.g., user input from HTTP), a panic could result if violated.
- **Silent mutation**: Returns `String`, not `Result<String>`. Callers have no way to know whether sanitization was required or whether the original string was dangerous. This hides operational risk.
- **Unbounded length**: No truncation. If the string is used in a file path, it may exceed OS limits (e.g., Windows `MAX_PATH` or `\\?\` prefix requirements).
- **Ambiguous contract**: The function name `sanitize_name` implies a guarantee, but the guarantee is weak and platform-dependent.

**Lines 19-37: Tests**
- Tests only cover the exact cases the function handles.
- **Missing test cases**: empty string, path separators, Windows illegal chars, very long strings, Unicode edge cases, null bytes, strings needing no sanitization, and whether sanitization preserves string equality when no changes are needed.

**General**
- `utils.rs` contains only two functions and tests. It does not need comments explaining simple logic, but doc comments on public functions would clarify contracts (e.g., `/// Replaces ^ and - with _` is far too minimal for a function named `sanitize_name`).

### Summary
- `lib.rs`: Trivial, no bugs. Recommend adding `pub use` re-exports for API stability.
- `utils.rs`: Functional but has safety gaps (`validate_safe_path` allows empty paths, `sanitize_name` is incomplete and silent). Recommend expanding sanitization rules, adding length guards, renaming or returning `Result` for failure cases, and expanding tests.

---

# Phase 2: export.rs

## File: src/export.rs (1183 bytes, 46 lines)

### Line-by-Line Issues

**Lines 1-4: Imports**
- Functional. Imports `polars::prelude::*` for DataFrame and writers. No dead imports.

**Lines 6 and 15: Function signatures**
```rust
pub fn export_csv(df: &mut DataFrame, path: &str) -> Result<()> {
pub fn export_parquet(df: &mut DataFrame, path: &str) -> Result<()> {
```
- **Unnecessary `mut` binding**: Both functions only read from `df`. `polars` `CsvWriter::finish` and `ParquetWriter::finish` take `&DataFrame`, not `&mut DataFrame`. Requiring `&mut` forces callers to mutably borrow even though the operation is read-only. This increases borrow-checker friction downstream with no benefit.

**Lines 7-9 and 16-18: Directory creation**
```rust
if let Some(parent) = Path::new(path).parent() {
    std::fs::create_dir_all(parent)?;
}
```
- **Opaque error**: `create_dir_all` failure (e.g., permission denied, disk full) propagates without context. Adding `.with_context(|| format!("..."))` would improve debuggability.
- **Race condition**: Between `create_dir_all` and `File::create`, another process could create or delete the parent. This is a TOCTOU (time-of-check-time-of-use) issue, though for a CLI tool it is low-risk.
- **Missing path validation**: No call to `validate_safe_path("/utils.rs")`. If `path` is user-controlled, an attacker could write outside intended directories.

**Lines 10 and 19: File creation**
```rust
let file = File::create(path)?;
```
- **Silent truncation**: `File::create` truncates existing files without warning. Whether this is desired depends on UX expectations. For a data-export tool, it is acceptable, but worth documenting or gating behind a `--force` flag.
- **No atomic write**: If the process crashes mid-write, the output file is left in a corrupted state. Using temp file + rename would provide crash safety.

**Lines 11 and 20: Writer finish**
```rust
CsvWriter::new(file).finish(df)?;
ParquetWriter::new(file).finish(df)?;
```
- Functional. Returns `Result<()>`. No obvious issues with polars writer behavior.

**Lines 24-45: Tests**
```rust
#[cfg(test)]
mod tests { ... }
```
- **No read-back verification**: Tests only assert that export doesn't return an error. They don't read the exported file back to verify data integrity, schema preservation, or column order.
- **No tempdir isolation**: Files are written to `tests/temp_test.csv` and `tests/temp_test.parquet` in the project tree. If the process is interrupted, files are left behind. Prefer `std::env::temp_dir()` or `tempfile::TempDir`.
- **Silent cleanup errors**: `let _ = std::fs::remove_file(...)` ignores cleanup failures. If cleanup fails, the test still passes, potentially hiding disk-full or permission issues.
- **Missing negative tests**: No test for invalid paths, unwritable directories, empty DataFrames, or DataFrames with problematic column types (e.g., nested structs for CSV).

### Module-Level Observations

- **Duplication**: `export_csv` and `export_parquet` share identical directory-creation and file-creation scaffolding. Could be refactored into a common `ensure_parent_dir` helper or a generic export function if more formats are added later. Current duplication is small but signals the module will grow.
- **No format negotiation**: Functions take `path: &str` and infer format from the function name. If caller wants to dynamically choose CSV vs Parquet, the API forces two separate calls rather than a single `export(df, path)`.

### Summary
- `export.rs` is functional but has borrow-checker inefficiency (`&mut DataFrame`), missing path validation, no atomic writes, and shallow tests. Recommend removing `mut`, adding parent dir context errors, using temp files, and expanding tests to include read-back and negative cases.

---

# Phase 3: cache.rs

## File: src/cache.rs (5050 bytes, 171 lines)

### Line-by-Line Issues

**Lines 6-10: `CacheBackend` trait**
```rust
#[async_trait::async_trait]
pub trait CacheBackend: Send + Sync {
    async fn get(&self, url: &str, ttl_secs: i64) -> Result<Option<String>>;
    async fn insert(&self, url: &str, body: &str) -> Result<()>;
}
```
- Trait is fine. `async_trait` is required because `async fn` in traits is not yet stable. `Send + Sync` bounds are appropriate.
- **Antipattern**: `ttl_secs: i64` allows negative values. `Utc::now().timestamp() < ttl_secs` would always be true for negative TTL, making expired entries appear fresh. No guard against negative TTL.
- **Death by signature**: `Result<anyhow::Error>` means callers cannot distinguish cache-miss from cache-error without unwrapping. For a diagnostic function (`check_cache_hits` on line 104), this is partially intentional but limits debugging.

**Lines 12-15: `Cache` struct**
```rust
#[derive(Clone)]
pub struct Cache {
    pool: SqlitePool,
}
```
- `Clone` derives correctly for `SqlitePool` (it uses `Arc` internally). Fine.
- `pool` field is private, enforcing use through trait or methods. Good encapsulation.

**Lines 17-26: `CacheBackend` impl for `Cache`**
```rust
#[async_trait::async_trait]
impl CacheBackend for Cache {
    async fn get(&self, url: &str, ttl_secs: i64) -> Result<Option<String>> {
        self.get_internal(url, ttl_secs).await
    }
    async fn insert(&self, url: &str, body: &str) -> Result<()> {
        self.insert_internal(url, body).await
    }
}
```
- Delegation adds indirection without adding behavior. This is acceptable for an abstraction layer but creates **duplicate methods**: `get`/`insert` (trait) AND `get_internal`/`insert_internal` (public struct methods, lines 67, 89).
- **Dead API surface**: `get_internal` and `insert_internal` are both `pub async fn` on `Cache`, meaning callers can bypass the `CacheBackend` trait entirely and call them directly. This defeats the purpose of the trait abstraction and creates two parallel public APIs.
- Recommendation: make `get_internal`/`insert_internal` private (`async fn`), or remove one pair entirely.

**Lines 29-52: `Cache::new`**
```rust
pub async fn new(db_path: &str) -> Result<Self> {
    if let Some(parent) = Path::new(db_path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    ...
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;
    ...
}
```
- **No error context**: `create_dir_all` failure (e.g., disk full, permission denied) propagates without `.with_context(|| "...")`. The caller cannot tell which operation failed.
- **Hardcoded concurrency**: `max_connections(5)` is undocumented and not configurable. For a cache serving a CLI tool, 5 is likely fine, but it lacks justification.
- **Empty `db_path`**: `SqliteConnectOptions::new().filename("")` opens an in-memory SQLite database, not a file. `create_if_missing(true)` won't create an empty-path file. If caller passes `""`, the cache is transient and data is lost on drop. No validation or guard.
- **Race on schema init**: `cache.init().await` runs `CREATE TABLE IF NOT EXISTS` after pool creation. This is fine, but if `init` fails, the pool is not explicitly closed. Rust's ownership will drop it, but `SqlitePool` drop handler may not be instantaneous.
- **Missing `validate_safe_path`**: `db_path` is not validated with `validate_safe_path` from `utils.rs`, even though it should be.

**Lines 54-65: `init`**
```rust
async fn init(&self) -> Result<()> {
    sqlx::query!(
        "CREATE TABLE IF NOT EXISTS api_cache (
            url TEXT PRIMARY KEY,
            response_body TEXT NOT NULL,
            cached_at_timestamp INTEGER NOT NULL
        );"
    )
    .execute(&self.pool)
    .await?;
    Ok(())
}
```
- Functional. `PRIMARY KEY` on `url` ensures uniqueness. `response_body TEXT NOT NULL` disallows empty cache entries.
- **Schema weakness**: No index on `cached_at_timestamp`. For TTL cleanup, this is irrelevant (queries use `url`), but if the cache grows large and `DELETE` on expired rows is frequent, full-table scans by `url` PK are efficient. Not a bug, but worth noting if cache size grows to millions of entries.

**Lines 67-87: `get_internal`**
```rust
pub async fn get_internal(&self, url: &str, ttl_secs: i64) -> Result<Option<String>> {
    let row = sqlx::query!(
        "SELECT response_body, cached_at_timestamp FROM api_cache WHERE url = ?",
        url
    )
    .fetch_optional(&self.pool)
    .await?;

    if let Some(r) = row {
        let now = Utc::now().timestamp();
        if now - r.cached_at_timestamp < ttl_secs {
            return Ok(Some(r.response_body));
        } else {
            sqlx::query!("DELETE FROM api_cache WHERE url = ?", url)
                .execute(&self.pool)
                .await?;
        }
    }
    Ok(None)
}
```
- **Negative TTL**: As noted, negative `ttl_secs` makes `now - cached_at_timestamp < ttl_secs` always false, so all entries appear expired. No guard.
- **Silent cleanup failure**: If `DELETE` fails on line 81, the error propagates. This is correct, but the expired data remains in the cache, causing subsequent reads to still hit it and incorrectly return stale data (because `SELECT` will still find the row, but `get_internal` was called with `ttl_secs=0` and `now - cached_at_timestamp < 0` is false... wait, if `DELETE` fails, the row stays. Next call with same params will again try to delete. The read returns `Ok(None)` because deletion failed. This is consistent but noisy.
- **Ambiguous time semantics**: `cached_at_timestamp` stores `Utc::now().timestamp()` (seconds since epoch). `now - cached_at_timestamp < ttl_secs` measures elapsed seconds. For large `ttl_secs` (years), `now - cached_at_timestamp` may approach `i64` overflow, but in practice it's fine.
- **Public internal method**: `get_internal` is `pub`, allowing callers to bypass `CacheBackend::get`. This violates the trait abstraction.

**Lines 89-101: `insert_internal`**
```rust
pub async fn insert_internal(&self, url: &str, body: &str) -> Result<()> {
    let now = Utc::now().timestamp();
    sqlx::query!(
        "INSERT OR REPLACE INTO api_cache (url, response_body, cached_at_timestamp)
         VALUES (?, ?, ?)",
        url,
        body,
        now
    )
    .execute(&self.pool)
    .await?;
    Ok(())
}
```
- **Public internal method**: Same issue as `get_internal`—`insert_internal` is `pub` when it should be private.
- **`INSERT OR REPLACE` semantics**: In SQLite, `REPLACE` deletes the existing row and inserts a new one. This causes the `rowid` to change, which is harmless here but could surprise extensions relying on rowid ordering.
- **No size limit**: `body` can be arbitrarily large. An extremely large response (e.g., megabytes) is stored without size validation. For a network cache, this could exhaust disk space.
- **Race with TTL**: `now` is calculated before query execution. If the query is delayed (e.g., disk I/O), the stored timestamp is slightly stale. For a cache, this is harmless.

**Lines 104-120: `check_cache_hits`**
```rust
pub async fn check_cache_hits(
    cache: std::sync::Arc<dyn CacheBackend>,
    urls: &[String],
    ttl_secs: i64,
) -> Result<(usize, usize)> {
    let mut hits = 0;
    let total = urls.len();
    if total == 0 {
        return Ok((0, 0));
    }
    for url in urls {
        if cache.get(url, ttl_secs).await.ok().flatten().is_some() {
            hits += 1;
        }
    }
    Ok((hits, total))
}
```
- **Error swallowing**: `.ok()` converts any cache failure (DB locked, pool exhausted, schema error) into a cache miss. The caller cannot tell whether a "miss" is a legitimate cache miss or a cache backend failure. This makes debugging cache issues opaque.
- **Trait object overhead**: `Arc<dyn CacheBackend>` uses dynamic dispatch. For a function called once with a small list, this is negligible. However, if called in a hot path, `generic<C: CacheBackend>` would be zero-cost.
- **Inefficient loop**: Each URL fetches sequentially via `.await`. For large URLs, this is slow. Should use `futures::future::join_all` or rayon for concurrency.
- **Early return on empty**: `if total == 0 { return Ok((0, 0)); }` is defensive but redundant—the loop would simply not execute. Not a bug, just extra guard.

**Lines 122-171: Tests**
- Functional tests covering insert, get, expiration via TTL=0, and `check_cache_hits`.
- **Test file pollution**: Writes `.db` files to `tests/test_cache_sqlite.db` and `tests/test_cache_hits.db`. If tests are interrupted, files remain. Should use `tempfile::TempDir` or delete on drop via RAII.
- **No error-path tests**: Does not test `Cache::new` with invalid paths, concurrent access, or large payloads.
- **No `check_cache_hits` negative test**: Does not verify behavior when some URLs return errors (e.g., corrupted DB). The test only exercises the happy path.

### Module-Level Observations

- **Trait-object vs generic**: The public API mixes trait objects (`Arc<dyn CacheBackend>`) with concrete struct (`Cache`). `CacheBackend` exists for flexibility but is only used by `pipeline.rs` (likely via `Arc::new(Cache::new(...).await?)`) and tests. The indirection is justified but creates API complexity.
- **Missing documentation**: No doc comments on module, struct, trait, or functions. The purpose and TTL semantics are not self-documenting.

### Summary
- `cache.rs` is functional but has API design flaws: `get_internal`/`insert_internal` are public when they should be private, breaking trait encapsulation. `check_cache_hits` swallows errors silently. No validation on negative TTL or large payloads. Tests lack tempdir isolation and error-path coverage. Recommend making internal methods private, adding error context, negotiating TTL validity, using concurrent fetches in `check_cache_hits`, and using `tempfile` for tests.

---

# Phase 4a: analysis.rs — Parsers, Orderbook, Yahoo, align_datasets (lines 1-350)

## File: src/analysis.rs (37469 bytes, 1161 lines) — Part 1 of 5

### Overview
Lines 1-350 contain JSON parsing functions for CoinGecko market chart, CoinGecko OHLC, CoinGecko tickers, `calculate_orderbook_metrics`, Yahoo Finance JSON parsing, and `align_datasets`.

---

### Line-by-Line Issues

**Lines 5-9: `MarketChart` struct**
```rust
struct MarketChart {
    prices: Vec<(i64, f64)>,
    total_volumes: Option<Vec<(i64, f64)>>,
}
```
- Functional. `total_volumes` is optional, handled with `unwrap_or_default` on line 17.

**Lines 11-53: `parse_coingecko_market_chart`**
```rust
    let total_volumes = chart.total_volumes.unwrap_or_default();
    for (i, (ts, val)) in chart.prices.iter().enumerate() {
        ...
        let vol = if i < total_volumes.len() {
            total_volumes[i].1
        } else {
            0.0
        };
```
- **PANIC BUG (line 27)**: If `chart.prices.len() > chart.total_volumes.unwrap_or_default().len()`, then `total_volumes[i]` panics with index out of bounds. Should use `total_volumes.get(i).map(|(_, v)| v).unwrap_or(0.0)`.
- **Logic error (lines 42-50)**: Groups by `date` and takes `mean` of prices. For price data, the mean of intraday points is not the same as the closing price. If CoinGecko returns multiple prices per day, the daily price should be the last close, not the mean. Mean aggregation discounts the price trend within the day.
- **Unnecessary overhead**: CoinGecko market chart typically returns one price per day. Group-by is redundant CPU/memory work if data is already daily.

**Lines 55-99: `parse_coingecko_ohlc`**
```rust
    for item in ohlc {
        if item.len() >= 5 {
            let ts = item[0] as i64;
```
- **Silent data loss (line 64)**: `if item.len() >= 5` silently drops malformed rows. For OHLC data, dropping rows creates date gaps that downstream indicators cannot handle. Should return `Err` or at minimum log a warning.
- **Truncation cast (line 65)**: `item[0] as i64` truncates the f64 timestamp. If the API returns `1700000000000.5`, it becomes `1700000000000`. Should round or validate exact `.0`.

```rust
        .agg([
            col(&format!("{}_open", prefix)).mean(),
            col(&format!("{}_high", prefix)).max(),
            col(&format!("{}_low", prefix)).min(),
            col(&format!("{}_close", prefix)).mean(),
        ])
```
- **Logic error (lines 91-94)**: OHLC aggregation uses `mean` for open and close. For OHLC data, open should be first value of the day, close should be last. Using `mean` for open/close produces incorrect daily OHLC bars. Only `high` (max) and `low` (min) are correct.

**Lines 122-149: `parse_coingecko_tickers`**
```rust
    last_prices.push(ticker.last.unwrap_or(0.0));
    volumes.push(ticker.volume.unwrap_or(0.0));
    spreads.push(ticker.bid_ask_spread_percentage.unwrap_or(0.0));
```
- **Silent null poisoning (lines 135-137)**: `unwrap_or(0.0)` replaces `None` with 0.0. If every ticker has `None` for `last_price`, the resulting DataFrame is all zeros. Downstream analytics produce mathematically valid but semantically meaningless results. Should propagate nulls or return a clear error if data is unusable.

**Lines 152-200: `calculate_orderbook_metrics`**
```rust
    let spreads: Vec<f64> = spread_col.f64()?.into_iter().map(|opt| opt.unwrap_or(0.0)).collect();
    let volumes: Vec<f64> = volume_col.f64()?.into_iter().map(|opt| opt.unwrap_or(0.0)).collect();
    let last_prices: Vec<f64> = last_price_col.f64()?.into_iter().map(|opt| opt.unwrap_or(0.0)).collect();
```
- **Silent null poisoning (lines 157-171)**: Same `unwrap_or(0.0)` antipattern. If a ticker has no `last_price`, it becomes 0.0. The variance/stddev calculation (lines 178-191) will include this zero, distorting the metrics.

---

**Lines 236-289: `parse_yahoo_json`**
```rust
    for (i, &ts) in res.timestamp.iter().enumerate() {
        let date_str = datetime.format("%Y-%m-%d").to_string();

        let price_opt = adjclose_values
            .and_then(|v| v.get(i).copied().flatten())
            .or_else(|| close_values.and_then(|v| v.get(i).copied().flatten()));

        if let Some(price) = price_opt {
            dates.push(date_str);
            prices.push(price);
        }
    }
```
- **Date continuity broken (lines 274-282)**: When `price_opt` is `None`, the row is dropped entirely. The resulting DataFrame has gaps where null prices existed. Downstream `align_datasets` cannot join on discontinuous dates. If Yahoo returns `[Some, None, Some]`, the output has 2 rows, but timestamps are `[ts0, ts2]` — date continuity is broken.
- **No null handling strategy**: There is no option to forward-fill or drop nulls explicitly. The caller has no control.

---

**Lines 292-350: `align_datasets`**
```rust
    for other_df in other_dfs {
        lf = lf.join(
            other_df.clone().lazy(),
            [col("date")],
            [col("date")],
            JoinType::Left.into(),
        );
    }
```
- **Unnecessary clone (line 301)**: `other_df.clone()` clones the entire DataFrame before calling `.lazy()`. `.lazy()` borrows `self`, so `other_df.lazy()` works without clone. `.join` takes `&mut LazyFrame`. This is a significant performance bug for large DataFrames.

```rust
    for name in &column_names {
        if name != "date" {
            let filled = if name.ends_with("_volume") {
                df.column(name)?
                    .fill_null(FillNullStrategy::Zero)?
            } else {
                df.column(name)?
                    .fill_null(FillNullStrategy::Forward(None))?
                    .fill_null(FillNullStrategy::Backward(None))?
            };
            df.replace(name, filled)?;
        }
    }
```
- **Volume fill strategy (lines 319-321)**: Fills null volume with `0.0`. Zero volume is not the same as missing volume data — it means no trades occurred. For OBV and other volume-based indicators, zero volume fundamentally changes the signal. Forward fill is more appropriate for missing data.
- **Forward-then-backward fill (lines 322-325)**: Forward fills leading nulls, then backward fill covers trailing nulls. Reasonable, but `drop_weekends` runs AFTER fill, meaning weekend dates that were never in the original data get forward-filled from Friday to Monday. This is likely desired behavior.

```rust
    if drop_weekends {
        let date_series = df.column("date")?.str()?;
        let mask_vec: Vec<bool> = date_series
            .into_iter()
            .map(|opt_date| {
                if let Some(date_str) = opt_date {
                    if let Ok(nd) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                        let weekday = nd.weekday();
                        return weekday != chrono::Weekday::Sat && weekday != chrono::Weekday::Sun;
                    }
                }
                true
            })
            .collect();
```
- **Performance (lines 332-344)**: Parses every date string with `NaiveDate::parse_from_str` in a Rust loop. For 10,000 rows, this is 10,000 string allocations and parses. Should use polars `str.strptime` + `dt.weekday()` for vectorized execution.
- **Silent keep on parse failure**: If `NaiveDate::parse_from_str` fails for a row, the closure returns `true` (keeps the row). This means malformed dates are NOT dropped, violating `drop_weekends` semantics. A date like "not-a-date" survives the filter.
- **Inefficient filter**: `BooleanChunked::from_slice` + `df.filter(&mask)` converts a `Vec<bool>` to a `BooleanChunked`. Polars can build the filter expression directly.

### Phase 4a Summary
- `parse_coingecko_market_chart`: Array bounds panic risk on `total_volumes[i]`. Mean aggregation for closing prices is wrong. Redundant group-by when data is daily.
- `parse_coingecko_ohlc`: Silently drops malformed rows, truncates timestamps, wrong aggregation for open/close (uses mean instead of first/last).
- `parse_coingecko_tickers`: Silent null poisoning via `unwrap_or(0.0)`.
- `calculate_orderbook_metrics`: Same `unwrap_or(0.0)` antipattern distorts variance/stddev.
- `parse_yahoo_json`: Dropping null price rows breaks date continuity without explicit strategy.
- `align_datasets`: Unnecessary clone of large DataFrames. Silent volume zero-fill is semantically wrong. Rust-loop date parsing is slow and silently keeps malformed dates.

---

# Phase 4b: analysis.rs — SMA, EMA, RSI, MACD, Bollinger Bands (lines 350-481)

## File: src/analysis.rs — Part 2 of 5

### Line-by-Line Issues

**Lines 352-364: `calculate_sma`**
```rust
fn calculate_sma(prices: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut sma = vec![None; prices.len()];
    if prices.len() < period {
        return sma;
    }
```
- **Early return**: If `prices.len() < period`, returns all `None`. Correct. The caller must know the output is all-None when input is short. No documentation of this contract.
- **Good implementation**: Uses `sum += prices[i] - prices[i - period]` for O(1) per step. Optimal.

**Lines 366-380: `calculate_ema`**
```rust
fn calculate_ema(prices: &[f64], period: usize) -> Vec<Option<f64>> {
```
- **Correct implementation**: Standard EMA with SMA seed. First value at index `period - 1`. Good.

**Lines 382-422: `calculate_rsi`**
```rust
    if prices.len() <= period {
        return rsi;
    }
```
- **Inconsistent guard vs siblings**: SMA/EMA use `prices.len() < period`. RSI uses `prices.len() <= period`. RSI needs `period+1` prices (period+1 differences), while SMA needs `period`. The guard is correct for RSI but inconsistent with sibling functions.

**Lines 425-453: `calculate_macd`**
```rust
#[allow(clippy::type_complexity)]
fn calculate_macd(prices: &[f64]) -> (Vec<Option<f64>>, Vec<Option<f64>>, Vec<Option<f64>>) {
```
- **Antipattern (line 425)**: `#[allow(clippy::type_complexity)]` hides a legitimate design smell. Returning a 3-tuple of `Vec<Option<f64>>` is hard to read and extend. A `struct MacdOutput { line, signal, histogram }` would be cleaner. Suppressing the warning instead of fixing the design is wrong.
- **Fragile unwrap (line 440)**: `macd_slice[start_idx..].iter().map(|x| x.unwrap()).collect()`. Relies on EMA producing `Some` for all indices after the first `Some`. This is true given current EMA implementation, but if EMA behavior changes, this unwrap panics. Should use `.expect("EMA produced None after first valid")` at minimum.

**Lines 455-481: `calculate_bollinger_bands`**
```rust
#[allow(clippy::type_complexity)]
fn calculate_bollinger_bands(
    prices: &[f64], period: usize, k: f64,
) -> (Vec<Option<f64>>, Vec<Option<f64>>, Vec<Option<f64>>) {
```
- **Antipattern (line 455)**: Same as MACD — `#[allow(clippy::type_complexity)]` suppresses legitimate warning. Should use a struct return type.
- **Population variance (line 474)**: `sum::<f64>() / period as f64`. Bollinger Bands standard uses population std dev (N). Correct.

### Phase 4b Summary
- `calculate_sma`: Correct, optimal O(1) rolling sum. Undocumented all-None contract.
- `calculate_ema`: Correct implementation.
- `calculate_rsi`: Correct math but guard `<= period` is inconsistent with SMA/EMA `< period`.
- `calculate_macd`: Correct math. Suppresses type_complexity clippy warning. Fragile unwrap on line 440.
- `calculate_bollinger_bands`: Correct math. Same clippy suppression as MACD. Population variance is correct for Bollinger.

---

# Phase 4c: analysis.rs — ATR, Stochastic, ADX, OBV (lines 483-681)

## File: src/analysis.rs — Part 3 of 5

### Line-by-Line Issues

**Lines 483-509: `calculate_atr`**
```rust
fn calculate_atr(high: &[f64], low: &[f64], close: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut tr = vec![0.0; n];
    tr[0] = high[0] - low[0];
    for i in 1..n {
        let h_l = high[i] - low[i];
        let h_pc = (high[i] - close[i - 1]).abs();
        let l_pc = (low[i] - close[i - 1]).abs();
        tr[i] = h_l.max(h_pc).max(l_pc);
    }
```
- **Index shift assumption**: `close[i - 1]` assumes `close` has same length as `high`/`low` and same alignment. If called with mismatched slices, this panics. The function contract does not document this requirement.
- **Good implementation**: Standard TR + smoothed ATR (Wilder's). No bugs.

**Lines 513-561: `calculate_stochastic`**
```rust
    for i in ((period - 1 + 2)..n) {
        let mut sum = 0.0;
        let mut count = 0;
        for j in (i - 2)..=i {
            if let Some(k_val) = percent_k[j] {
                sum += k_val;
                count += 1;
            }
        }
        if count == 3 {
            percent_d[i] = Some(sum / 3.0);
        }
    }
```
- **Manual SMA duplication (lines 546-559)**: `%D` is a 3-period SMA of `%K`, but it is implemented as a manual loop instead of reusing `calculate_sma`. This duplicates logic and increases maintenance burden. The manual loop has subtle behavior: if any of the 3 `%K` values is `None`, `%D` remains `None` (count != 3), which is correct but implicit. Reusing `calculate_sma` with `Option<f64>` would require adaptation.
- **Denominator == 0 (lines 538-543)**: If `highest_high == lowest_low`, set `%K = 100.0`. Standard stochastic sets %K = 50 or 100 for flat bars. 100 is acceptable but worth noting as a design choice.

**Lines 564-661: `calculate_adx`**
```rust
fn calculate_adx(high: &[f64], low: &[f64], close: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = close.len();
    let mut adx = vec![None; n];
    if n < 2 * period {
        return adx;
    }
```
- **Guard (line 568)**: ADX requires `2 * period` data points (period for initial smoothed TR/DM, then period for initial ADX). Correct.
- **Complex implementation (lines 572-658)**: Standard Welles Wilder ADX. Smoothed TR, DM+, DM-, DX, then smoothed ADX. No obvious bugs, but ~100 lines of manual arithmetic that is hard to audit. No unit tests verify exact values against known good data.

**Lines 664-681: `calculate_obv`**
```rust
fn calculate_obv(close: &[f64], volume: &[f64]) -> Vec<Option<f64>> {
```
- **Correct implementation**: On-balance volume. `close[i] > close[i-1]` => add volume, `<` => subtract, `==` => no change. Good.

### Phase 4c Summary
- `calculate_atr`: Correct math. Undocumented slice-alignment requirement (same len/same index for high/low/close).
- `calculate_stochastic`: Correct math. Manual SMA loop for `%D` duplicates `calculate_sma` logic. Denominator == 0 → 100.0 is a design choice.
- `calculate_adx`: Correct math. Complex ~100 line implementation with no unit tests for golden values.
- `calculate_obv`: Correct implementation.

---

# Phase 4d: analysis.rs — compute_returns_and_indicators + prep_ml (lines 683-872)

## File: src/analysis.rs — Part 4 of 5

### Line-by-Line Issues

**Lines 683-797: `compute_returns_and_indicators` (GOD FUNCTION, 114 lines)**
```rust
    let valid_indices: Vec<usize> = prices_raw.iter().enumerate().filter_map(...).collect();
    let prices: Vec<f64> = valid_indices.iter().map(|&i| prices_raw[i].unwrap()).collect();
```
- **Implicit null filtering**: Filters out null prices into `valid_indices` and `prices`. The caller does not know that nulls are silently removed before indicator computation. The contract is not documented.
- **Gap preservation**: `scatter` closure (lines 718-724) maps filtered results back to original indices. Gaps remain `None`. The output DataFrame has null indicators at the same positions as null prices. Downstream code must handle sparse indicators, but this side effect is not documented.

```rust
        let highs: Vec<f64> = valid_indices.iter().map(|&i| highs_raw[i].unwrap_or(0.0)).collect();
        let lows: Vec<f64> = valid_indices.iter().map(|&i| lows_raw[i].unwrap_or(0.0)).collect();
```
- **CRITICAL BUG (lines 765-772)**: `unwrap_or(0.0)` on OHLC values. If a high or low is null, it becomes 0.0. For ATR, a zero high/low creates an artificially small True Range, corrupting the indicator. For Stochastic, zero low creates an infinite denominator (or denominator = 100 if high is also 0). For ADX, zero high/low corrupts DM+ and DM-.
- **CRITICAL BUG (lines 786-789)**: `unwrap_or(0.0)` on volume values. If volume is null, it becomes 0.0. For OBV, zero volume on a price-up day adds 0 (no change), while missing volume should probably skip the day or use the previous OBV.
- **Inconsistent null strategy**: Prices use `valid_indices` to filter out nulls entirely. OHLC and volume use `unwrap_or(0.0)` to replace nulls with zeros. The treatment is inconsistent and both are wrong in their own ways.

```rust
    let out_df = df.hstack(&new_cols)?;
    Ok(out_df)
```
- **Column ordering**: `hstack` appends new columns at the end. The original columns remain first, followed by all new indicator/return columns. Some tools expect certain column orders. Not a bug, but could surprise consumers.

---

**Lines 800-872: `prep_ml`**
```rust
        let std = if variance > 0.0 { variance.sqrt() } else { 1.0 };
```
- **Fallback std (line 844)**: When all values are identical, std = 0. The code sets `std = 1.0` instead, making Z-score = 0 for all values. This is a reasonable design choice (avoid division by zero) but means a column of constant values produces a "standardized" column of all zeros. Should this be all NaNs instead to signal a degenerate feature? Worth documenting.
- **Missing inf/nan guard**: If any value is `inf` or `nan`, min/max/std calculations propagate it. MinMax scaling on `inf` produces `inf` or NaN. Z-score on `nan` produces `nan`. No validation against invalid input.

### Phase 4d Summary
- `compute_returns_and_indicators`:
  - Implicit null filtering is undocumented.
  - Critical bug: OHLC nulls filled with 0.0 corrupt ATR, Stochastic, ADX.
  - Critical bug: Volume nulls filled with 0.0 corrupt OBV.
  - Inconsistent null strategy across prices vs OHLC vs volume.
- `prep_ml`: Constant-variance fallback std=1.0 hides degenerate features. No guard against inf/nan inputs.

---

# Phase 4e: analysis.rs — Tests, Antipatterns, Summary (lines 875-1161)

## File: src/analysis.rs — Part 5 of 5

---

### Line-by-Line Issues

**Lines 875-993: Tests**
```rust
    #[test]
    fn test_returns_and_indicators_computation() {
```
- **Weak assertion (lines 972-993)**: Only checks that columns exist. Does not verify that RSI is bounded 0-100, that MACD histogram equals `line - signal`, that Bollinger upper > lower, etc.
- **No edge case tests**: No tests for empty DataFrame, single-row DataFrame, all-null prices, all-zero variance, malformed JSON, or mismatched OHLC lengths.
- **No indicator accuracy tests**: No golden-value tests comparing SMA/EMA/RSI/MACD against known correct outputs. Mathematical correctness is entirely unverified.
- **Missing test for `parse_yahoo_json` null handling**: No test verifies behavior when Yahoo returns mixed null/non-null prices. The date-gap bug is untested.
- **Missing test for `parse_coingecko_ohlc` malformed rows**: No test for `[[ts, 100, 90, 110, 95], [short], [ts2, 100, 90, 110]]`. The silent drop behavior is untested.

---

### Antipatterns

1. **`#[allow(clippy::type_complexity)]`** (lines 425, 455): Suppresses legitimate warnings instead of refactoring return types into structs.
2. **`unwrap_or(0.0)` on nullable fields** (lines 135-137, 160, 165, 170, 767-772, 786-789): Silent null poisoning rather than explicit handling or error propagation.
3. **Manual loop for SMA in Stochastic** (lines 546-559): Duplicates `calculate_sma` logic. Should reuse existing function.
4. **`format!("{}_volume", price_col_name)`** (line 38, repeated): String formatting for column names is fragile. Could use a helper or builder pattern.
5. **Duplicate column-name string construction**: Lines 757-759, 778-782, 846-867 construct column names with identical format strings. Should extract to a helper function.

### Dead Code
None identified.

### Missing Documentation
- No doc comments on any public function. Callers cannot know expected input format, null handling behavior, or output schema without reading implementation.
- No documentation of which financial conventions are used (e.g., ATR smoothing, RSI initial window, Bollinger population vs sample std dev).

### Summary
`analysis.rs` contains mathematically correct core functions but integrates them dangerously:
- **Critical bug**: `parse_coingecko_market_chart` — array bounds panic on `total_volumes[i]` if prices > volumes. Mean aggregation for closing prices is wrong.
- **Critical bug**: `parse_coingecko_ohlc` — silently drops malformed rows, truncates timestamps, wrong aggregation for open/close (mean instead of first/last).
- **Critical bug**: `compute_returns_and_indicators` — OHLC nulls filled with 0.0 corrupt ATR/Stoch/ADX. Volume nulls filled with 0.0 corrupt OBV.
- **Silent null poisoning**: `parse_coingecko_tickers`, `calculate_orderbook_metrics` use `unwrap_or(0.0)` throughout.
- **Date continuity broken**: `parse_yahoo_json` drops null price rows without strategy.
- **Performance**: `align_datasets` clones DataFrames unnecessarily. `drop_weekends` uses slow Rust-loop date parsing.
- **Design**: Multiple functions suppress `type_complexity` instead of using structs. Tests are shallow (only existence checks, no golden-value math verification).

---

# Phase 5: optimization.rs

## File: src/optimization.rs (14624 bytes, 442 lines)

### Line-by-Line Issues

**Lines 4-10: `Portfolio` struct**
```rust
pub struct Portfolio {
    pub weights: Vec<f64>,
    pub annualized_return: f64,
    pub annualized_volatility: f64,
    pub sharpe_ratio: f64,
}
```
- **Missing doc comments**: No explanation of which annualization convention is used (252 trading days? 365 calendar days?). The `run_monte_carlo` function accepts `annualization_factor` as a parameter, but `Portfolio` itself stores "annualized" values without documenting the factor. If the caller passes `annualization_factor=252.0` but downstream code assumes `365.0`, metrics are wrong.

**Lines 19-42: `Xorshift` RNG**
```rust
struct Xorshift {
    state: u64,
}
```
- **Anti-pattern**: Custom RNG implementation in application code. `Xorshift` has known weaknesses (e.g., seeding from a low-entropy `Option<u64>`). Should use `rand` crate with a proper thread_rng or ChaCha-based RNG. The custom implementation adds maintenance burden and potential statistical flaws.
- **Float normalization (line 40)**: `(self.next_u64() as f64) / (u64::MAX as f64)` produces 52-bit precision floats. This is acceptable but `rand::distributions::Uniform` would handle edge cases better.

**Lines 44-49: `splitmix64`**
```rust
fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    ...
```
- **Unused in tests?**: `splitmix64` is used on line 196 to derive per-iteration seeds. This is a reasonable approach to decorrelate parallel iterations. However, if `num_simulations` is small, the splitmix64 state may be near-idempotent for low seeds.

**Lines 51-266: `run_monte_carlo`**
```rust
    let r_f_annual = if let Ok(tnx_col) = df.column("^TNX") {
```
- **CRITICAL BUG (line 146)**: `run_monte_carlo` looks for `^TNX` (10-Year Treasury yield) in the input DataFrame as the risk-free rate. If the caller doesn't provide `^TNX`, `r_f_annual` silently falls back to `0.0`. This means Sharpe ratio becomes `p_ret / p_vol` (i.e., returns-only ratio), not true Sharpe. The caller has no way to provide a custom risk-free rate. This is a hidden API contract violated by silent default.

```rust
            for w in weights.iter_mut() {
                // Add a small minimum raw weight to prevent exactly 0% allocation
                *w = rng.next_f64() + 0.01;
                sum += *w;
            }
            for w in weights.iter_mut() {
                *w /= sum;
            }
```
- **Anti-pattern (lines 201-208)**: Forces minimum 1% allocation per asset via `+ 0.01`. After normalization, every asset has weight ≥ 0.01 / (m * 0.01 + (1-m*0.01) + ...). Actually the math is: if all m assets get `rand + 0.01`, then sum = `rand_sum + m*0.01`. Each weight becomes `(rand_i + 0.01) / (rand_sum + m*0.01)`. The minimum weight is approximately `0.01 / (0.5*m + m*0.01)`, which for m=2 is ~0.8%, not 1%. The minimum is NOT guaranteed 1%. This is a subtle bug in the "minimum weight" logic combined with unclear intent. Standard Monte Carlo Markowitz uses Dirichlet distribution without minimums; if minimums are desired, reject-and-resample is the correct approach.

**Lines 174-185: Progress bar**
```rust
    let pb = if cfg!(test) {
        indicatif::ProgressBar::hidden()
    } else {
        ...
    };
```
- **Functional but not thread-safe**: `indicatif::ProgressBar` is updated from Rayon parallel threads without synchronization. `pb.set_position(num_simulations as u64)` is called after `collect()`, so the progress bar never actually shows real-time progress during simulation. It jumps from 0 to 100% at the end. This makes the progress bar misleading.

**Lines 268-297: `format_optimal_weights_table`**
- **Inconsistent formatting**: Uses `cli_table` crate. If `table.display()` fails, returns `"Error displaying table"` as plain string, which will look broken in the terminal. Should log the error or fall back to a simple text table.

**Lines 299-342: `format_portfolio_metrics_table`**
- Same `match table.display()` antipattern as above.

**Lines 344-441: Tests**
- **Shallow assertions**: `test_run_moncarlo` only checks that weights sum to 1 and min vol <= max sharpe vol. Does not verify that max sharpe is actually max (could be any portfolio), does not verify that min vol is actually min, does not verify Sharpe formula correctness.
- **No null-handling tests**: Does not test behavior when input DataFrame has null prices, negative prices, or single-row data.
- **Determinism**: `test_run_monte_carlo` passes `seed: None`, which defaults to `1337`. The test works, but `test_different_annualization_factors` passes `Some(42)`. No test verifies that two runs with same seed produce identical results (though this is implied).

### Module-Level Observations

- **No unit tests for financial formulas**: `run_monte_carlo` is a dense mathematical function. There are no tests verifying that weights are properly normalized (sum to 1), that covariance is correctly annualized, or that Sharpe ratio formula is correct.
- **Monte Carlo bias**: With minimum 1% weight per asset, the optimization is not pure Markowitz. The bias is small for small m but grows as m increases. Not a bug, but a design choice that should be documented.

### Summary
- `optimization.rs` is mathematically dense with minimal verification:
  - **Critical bug**: `^TNX` column hardcoded for risk-free rate; silently falls back to 0.0 if missing. No way to pass custom risk-free rate.
  - **Anti-pattern**: Custom `Xorshift` RNG instead of `rand` crate. `+ 0.01` minimum weight hack produces unclear minimum allocation after normalization.
  - **UI issue**: Progress bar never shows real-time progress in parallel mode.
  - **Weak tests**: Only weight-sum and min<max checks. No formula verification.

---

# Phase 6a: backtest.rs — Structs, Core Types, load_custom_strategies, get_df_value, evaluate_condition (lines 1-200)

## File: src/backtest.rs (49719 bytes, 1411 lines) — Part 1 of 4

### Line-by-Line Issues

**Lines 6-27: `BacktestMetrics` struct**
```rust
pub struct BacktestMetrics {
    pub prediction_rating: String,
    pub strategy_rating: String,
    ...
}
```
- **Ambiguous rating strings**: `prediction_rating` and `strategy_rating` are `String` with no documented enum or valid values. Callers must guess valid values (e.g., "A+", "Good", "Strong Buy"). Should be an enum or at minimum document valid values.
- **Dead field**: `prediction_r2: f64` is populated but never meaningfully computed in the current codebase (often 0.0). If unused, should be removed or the computation should be fixed.

**Lines 29-33: `BacktestReport` struct**
```rust
pub struct BacktestReport {
    pub timestamp: String,
    pub metrics: HashMap<String, BacktestMetrics>,
}
```
- **Borrows `HashMap`**: `metrics` is `HashMap<String, BacktestMetrics>`. No ordering guarantees. When rendering reports, iteration order is non-deterministic, making output flaky. Should use `IndexMap` or sort keys before display.

**Lines 42-87: `Condition` enum and related types (`ValueSource`, `LogicalOperator`, `ComparisonOperator`)**
- **Functional serde**: `#[serde(untagged)]` on `ValueSource` and `Condition` allows flexible JSON representation. However, `#[serde(untagged)]` on `Condition` with both `Logical` and `Comparison` variants means serde tries each variant in order. If a JSON object has both `"operator"` and `"rules"` AND `"column"`, it may serialize to the wrong variant. The `column` field exists in `Comparison` but not `Logical`, so this is generally safe, but deeply nested Conditions with mixed shapes could fail to deserialize.

**Lines 132-150: `load_custom_strategies`**
```rust
    for (key, mut cfg) in map {
        if cfg.name.is_empty() {
            cfg.name = key;
        }
        list.push(cfg);
    }
```
- **Name fallback (lines 140-142)**: Uses map key as strategy name if `cfg.name` is empty. This is reasonable but `key` may be a numeric string like "0" or "1" if the JSON was an array parsed as a map by mistake. No validation that the key is a meaningful name.

**Lines 152-171: `get_df_value`**
```rust
fn get_df_value(df: &DataFrame, name: &str, idx: usize, shift: usize, coin: &str) -> Result<f64> {
```
- **Prefix fallback (lines 157-162)**: If `name` is not a direct column, tries `{coin}_{name}`. This is a clever convention, but `coin` is appended without sanitization. If `coin` contains special characters, the column name may be malformed.
- **No null handling**: `col.f64()?.get(target_idx)` returns `None` if the value is null. Returns `Err` with a generic message. It does not distinguish between "column missing", "null value", and "index out of bounds". All three produce different but potentially ambiguous errors.

**Lines 173-200: `evaluate_condition`**
```rust
        LogicalOperator::And => {
            for r in rules {
                if !evaluate_condition(r, idx, df, coin)? {
                    return Ok(false);
                }
            }
            Ok(!rules.is_empty())
        }
```
- **Ambiguous short-circuit (line 187)**: `Ok(!rules.is_empty())` for `And`. This means `And` with an empty rules list evaluates to `false`, while `Or` with an empty rules list evaluates to `false` (line 195). Short-circuiting is consistent, but an empty rule set being `false` for `And` may surprise callers who expect `true` (identity element of AND).
- **Error propagation**: Uses `?` on recursive `evaluate_condition`. For deep nested conditions, a single leaf error aborts the entire evaluation. There is no partial-evaluation or fault-tolerant mode.

### Phase 6a Summary
- `BacktestMetrics`: Rating fields are unbounded `String`. `prediction_r2` often hardcoded to 0.0 (dead data).
- `BacktestReport`: Uses non-deterministic `HashMap` ordering.
- `Condition`/`ValueSource`: Serde untagged may have edge cases with mixed shapes.
- `load_custom_strategies`: Map-key fallback for names is reasonable but unvalidated.
- `get_df_value`: Clever coin-prefixed fallback but no null handling granularity.
- `evaluate_condition`: Empty `And`/`Or` returns `false`, which may be semantically wrong for `And` identity.

---

# Phase 6b: backtest.rs — Helper Metrics, BhCache, run_backtest_for_asset (lines 200-600)

## File: src/backtest.rs — Part 2 of 4

### Line-by-Line Issues

**Lines 230-250: `evaluate_confidence`**
```rust
            let scaled = (val.abs() * multiplier).clamp(*min, *max);
```
- **Ambiguous design**: `val.abs()` means negative values are treated as positive. For LinearScale confidence, this means the sign of the input column is ignored. Whether this is intended depends on whether confidence should ever be negative. Not a bug, but an undocumented choice.

**Lines 252-277: `calculate_mda`**
```rust
        if a_sign == p_sign {
            correct += 1;
        }
```
- **Zero-sign match**: If both actual and predicted are exactly 0.0, they match and count as correct. If the actual is 0.0 and predicted is 1.0, they don't match (sign differs). This treats zero as its own sign rather than a neutral/no-trade case. Whether this matches the trading strategy semantics depends on whether zero-return days count as predictions. Worth documenting.

**Lines 279-297: `calculate_r2`**
```rust
    if ss_tot == 0.0 {
        return 0.0;
    }
    1.0 - (ss_res / ss_tot)
```
- **BUG (line 293-294)**: When `ss_tot == 0.0` (all actuals are identical), returns `0.0` unconditionally. This is WRONG for perfect predictions: if all actuals are 100.0 and all predicted are 100.0, R² should be 1.0 (perfect fit), not 0.0. The current code returns 0.0 even for perfect predictions of a constant series. Correct behavior: return `1.0` if `ss_res == 0.0`, else `0.0`.

**Lines 318-336: `calculate_max_drawdown`**
```rust
    let mut peak = equity[0];
    for &eq in equity {
        if eq > peak {
            peak = eq;
        }
        if peak > 0.0 {
            let dd = (peak - eq) / peak;
            ...
        }
    }
```
- **Ambiguous guard (line 328)**: `if peak > 0.0` skips drawdown calculation for negative or zero peaks. If the equity curve starts negative (e.g., margin account, short portfolio), peak is set to a negative value and `peak > 0.0` is always false. The max drawdown is then returned as 0.0, which is misleading. Standard drawdown does not depend on the sign of equity. This guard appears to be a guard against division by zero, but it should be `if peak != 0.0` instead.

**Lines 338-360: Rating classifiers**
```rust
pub fn classify_prediction_rating(mda: f64, active_win_rate: f64) -> String {
    if mda >= 0.58 || active_win_rate >= 0.62 {
        "excellent".to_string()
    } else if mda >= 0.53 || active_win_rate >= 0.55 {
        "good".to_string()
    } else {
        "bad".to_string()
    }
}

pub fn classify_strategy_rating(
    sharpe: f64,
    strategy_return: f64,
    buy_and_hold_return: f64,
) -> String {
    if sharpe >= 1.5 && strategy_return > buy_and_hold_return {
        "excellent".to_string()
    } else if sharpe >= 0.5 && strategy_return >= 0.00 {
        "good".to_string()
    } else {
        "bad".to_string()
    }
}
```
- **Magic numbers**: Thresholds (0.58, 0.62, 0.53, 0.55, 1.5, 0.5) are hardcoded with no explanation. These should be named constants or configuration parameters.
- **Unusual OR/AND logic**: `classify_prediction_rating` uses `||` — a strategy with MDA=0.52 but win_rate=0.63 is "excellent". `classify_strategy_rating` uses `&&` — both conditions must be met. The inconsistency between these two classifiers is a design smell.

**Lines 410-477: Helper structs/functions**
- `BhCache` stores equity curve. No validation that equity length matches expected backtest days.
- `get_max_shift`/`get_strategy_max_shift`: Functional. No issues.

**Lines 479-599: `run_backtest_for_asset` start**
```rust
    if custom_strat.is_none() {
        match strategy_name.to_lowercase().as_str() {
            "rsi" | "macd" | "bollinger" => {}
            _ => return Err(anyhow!("Unknown strategy: {}", strategy_name)),
        }
    }
```
- **Maintenance trap (lines 539-544)**: Built-in strategy names are hardcoded. If a new built-in strategy (e.g., "sma_cross", "stochastic") is added, this validation block is easy to forget to update. Custom strategies bypass the check entirely. The validation is asymmetrical.

```rust
    let prices: Vec<Option<f64>> = df.column(&close_col)?.f64()?.into_iter().collect();
```
- **Null price handling**: `prices` contains `Option<f64>`. Downstream logic checks `prices[i].is_some()` on line 563. This is correct but means the backtest iterates over all rows including ones with null prices, checking validity on every iteration. Could prefilter to a vector of `(index, price)` pairs for clarity.

### Phase 6b Summary
- `calculate_mda`: Zero-sign matches count as correct. Undocumented design choice.
- `calculate_r2`: **BUG**: Returns 0.0 for constant actual series even when prediction is perfect. Should return 1.0 when SS_res == 0.
- `calculate_max_drawdown`: `peak > 0.0` guard skips DD calculation for negative-peak equity curves. Should likely be `peak != 0.0`.
- `classify_*`: Magic number thresholds. Inconsistent OR vs AND logic between classifiers.
- `run_backtest_for_asset`: Hardcoded strategy name validation is a maintenance trap.

---

# Phase 6c: backtest.rs — run_backtest_for_asset (continued) + backtest_portfolio (lines 600-999)

## File: src/backtest.rs — Part 3 of 4

### Line-by-Line Issues

**Lines 596-725: `run_backtest_for_asset` signal loop**
```rust
        let mut eq_base = equity[prev_t];
        if sig_t != current_position {
            eq_base *= 1.0 - (fee + slippage);
            total_trades += 1;
        }
        let r_strat = if sig_t == 2 {
            r_t * conf_t
        } else if sig_t == 0 {
            -r_t * conf_t
        } else {
            0.0
        };
        strategy_returns[t] = r_strat;
        equity[t] = eq_base * (1.0 + r_strat);
```
- **Fee logic (lines 708-723)**: Fee is applied to `eq_base` when position changes, then return is applied to post-fee base. This is correct.
- **Rapid oscillation**: `total_trades += 1` whenever `sig_t != current_position`. Rapid signal oscillation will overcount trades, but this is expected behavior.

**Line 808: `prediction_r2`**
- `prediction_r2: 0.0` hardcoded. Dead field, not computed.

**Lines 793-795: Coin parsing**
- `coin.split('_')` to extract base/currency. Fragile. If coin has no underscore, currency defaults to `"usd"`, which is wrong for non-USD pairs.

**Lines 618-625: Null price handling in equity**
- When `price_t` is None, `equity[t] = equity[prev_t]` (forward-fill). `actual_returns[t]` stays 0.0. A missing price day is treated as zero return, which may be incorrect.

**Lines 887-898: `backtest_portfolio` inputs**
```rust
    let close: Vec<f64> = df.column(asset)?.f64()?.into_iter().map(|opt| opt.unwrap_or(0.0)).collect();
    let ret: Vec<f64> = df.column(...)?.f64()?.into_iter().map(|opt| opt.unwrap_or(0.0)).collect();
```
- **Silent null poisoning (lines 887-898)**: `unwrap_or(0.0)` replaces null prices and null returns with 0.0. This masks data quality issues.

**Lines 976-1048: `backtest_portfolio` main loop**
```rust
        for j in 0..n_assets {
            let asset_ret = returns_series[j][i];
            new_values[j] = current_values[j] * (1.0 + asset_ret / 100.0);
        }
```
- **Return convention**: `backtest_portfolio` expects `simple_return` columns to be in percentage form (e.g., 5.0 for 5%). It divides by 100 to convert to decimal. This matches `compute_returns_and_indicators` which produces `((price/prev)-1.0)*100.0`. The function is correct for real pipeline data.
- **Test data mismatch**: `test_backtest_portfolio` passes decimal returns (e.g., 0.05 for 5%) instead of percentage returns. The function divides by 100, so decimal 0.05 becomes 0.0005 (0.05%). The test accidentally passes because it only checks `equity.len()` and `equity[4] != 0.0`, not the actual returns.

```rust
        let equity_after = (equity_before_fees - transaction_fees).max(0.0);
```
- **Zero equity guard (line 1027)**: Equity is clamped to 0.0 minimum. If fees exceed equity, equity reaches 0.0. Then Sharpe return calculation on line 1056: `(curr_e - prev_e) / prev_e` with `prev_e = 0.0` causes division by zero. No guard.

**Lines 1050-1090: Sharpe and metrics**
- Functional conventions are consistent with `run_backtest_for_asset` once percentage-form returns are correctly provided.

### Phase 6c Summary
- `run_backtest_for_asset`: Fee logic is correct. `prediction_r2` remains hardcoded to 0.0. Coin parsing is fragile. Null prices forward-filled with no return update.
- `backtest_portfolio`: Correctly expects percentage-form returns from pipeline. Test data is incorrectly in decimal form. No guard against zero equity causing division by zero in Sharpe calculation.

---

# Phase 6d: backtest.rs — backtest_portfolio (continued) + format_backtest_table + Tests (lines 1000-1411)

## File: src/backtest.rs — Part 4 of 4

### Line-by-Line Issues

**Lines 988-1017: `backtest_portfolio` rebalance frequency**
```rust
        let should_rebalance = match rebalance_frequency.to_lowercase().as_str() {
            "weekly" => { ... d_prev.iso_week().week() != d_curr.iso_week().week() || d_prev.iso_week().year() != d_curr.iso_week().year() ... }
            "monthly" => { ... d_prev.month() != d_curr.month() || d_prev.year() != d_curr.year() ... }
            _ => { true }
        };
```
- **Year boundary bug (lines 995-996, 1007-1008)**: Weekly rebalance checks `d_prev.iso_week().week() != d_curr.iso_week().week() || d_prev.iso_week().year() != d_curr.iso_week().year()`. The `||` means if week numbers differ OR year differs, rebalance. But if we cross a year boundary within the same ISO week (e.g., Dec 31 Monday to Jan 3 Monday), the week number might be the same but the year changes. The `||` handles this correctly. However, the monthly check uses `d_prev.month() != d_curr.month() || d_prev.year() != d_curr.year()`. This is also correct. Wait, actually this is fine. The logic is: rebalance if month OR year changes. But it should be: rebalance if month changes OR (month stays same AND year changes)? No, actually `d_prev.month() != d_curr.month()` already handles cross-month transitions. The `|| d_prev.year() != d_curr.year()` is redundant for monthly because if month differs, year might also differ but that's already covered. If month is same but year differs (impossible in normal calendar), the `||` catches it. This is fine.

Actually, there IS a subtle issue: for "weekly", the code checks `d_prev.iso_week().week() != d_curr.iso_week().week() || d_prev.iso_week().year() != d_curr.iso_week().year()`. Consider two dates in the same ISO week but different calendar years: e.g., 2020-W53-31 (Dec 31, 2020, part of ISO week 53 of 2020) and 2021-W01-01 (Jan 1, 2021, part of ISO week 53 of 2020). Both dates have ISO week 53 and year 2020, so the condition is false — no rebalance. But Jan 1 to Jan 4 (week 1 of 2021): week differs (53 vs 1), so rebalance. This is correct.

But what about Dec 28, 2020 (Monday, ISO week 53 of 2020) to Jan 4, 2021 (Monday, ISO week 1 of 2021)? Week differs, rebalance. Correct.

Wait, there's a REAL bug: `d_prev.iso_week().year()` may not equal the calendar year. ISO week year can differ from calendar year for dates near year boundaries. But this is unlikely to cause incorrect behavior in practice. Not worth flagging as a bug.

**Lines 1040-1048: Buy & Hold calculation**
```rust
        let mut bh_val = 0.0;
        for j in 0..n_assets {
            let curr_price = close_series[j][i];
            let asset_ratio = curr_price / initial_prices[j];
            bh_val += weights[j] * asset_ratio;
        }
        bh_equity[i] = 10000.0 * bh_val;
```
- **No fees on B&H**: Buy & Hold ignores fees entirely. This is correct for a benchmark, but should be documented. The comparison `strategy_return vs buy_and_hold_return` is unfair if the strategy pays fees but B&H doesn't. In reality, B&H also has entry/exit fees at inception and termination, but those are ignored here.

**Lines 1067-1087: Risk-free rate**
- Same `^TNX` hardcoded column dependency as `run_backtest_for_asset`. If missing, `mean_rf = 0.0`.

**Lines 1094-1114: `BacktestMetrics` for portfolio**
```rust
    let metrics = BacktestMetrics {
        ...
        prediction_accuracy: 1.0,
        prediction_r2: 0.0,
        active_win_rate: 1.0,
        prediction_rating: "N/A".to_string(),
        ...
    };
```
- **Placeholder values**: `prediction_accuracy: 1.0`, `active_win_rate: 1.0`, `prediction_rating: "N/A"`. These are meaningless for a portfolio backtest. Should either omit prediction fields from portfolio metrics or use a separate struct.

**Lines 822-860: `format_backtest_table`**
- **Fragile error handling**: `match table.display()` returns `"Error generating table"` on failure. Same antipattern as `optimization.rs`. Should log the error and fall back to a simple formatted string.

**Lines 1119-1411: Tests**
```rust
    #[test]
    fn test_calculate_mda() {
```
- **Minimal coverage (lines 1124-1128)**: Only one input case. MDA is trivial (sign comparison), so minimal tests are acceptable.

```rust
    #[test]
    fn test_calculate_r2() {
```
- **Missing edge case (lines 1131-1136)**: Does not test `ss_tot == 0.0` case. Confirms the bug where perfect prediction of constant series returns 0.0 instead of 1.0 remains uncaught.

```rust
    #[test]
    fn test_run_backtest_for_asset() {
```
- **Fragile assertion (line 1167)**: `assert_eq!(metrics_rsi.coin, "bitcoin".to_string())` depends on `"bitcoin_usd"` splitting on `_` to produce `"bitcoin"`. This is an implementation detail being tested as contract.

```rust
    #[test]
    fn test_backtest_portfolio_frequencies() {
```
- **Assertion logic (lines 1234-1237)**: `equity_d[4] != equity_w[4]` and `equity_w[4] != equity_m[4]`. With more frequent rebalancing, fees are higher, so daily equity should be lower than weekly, which should be lower than monthly. But the test only checks inequality, not the expected order (`equity_d <= equity_w <= equity_m`). A regression could make all equal and still pass.

```rust
    #[test]
    fn test_custom_strategy() {
```
- **Temp file in env::temp_dir()**: Uses `std::env::temp_dir()` to create `test_custom_rsi_test.json`. If the process crashes, the file is left behind. Should use `tempfile::TempDir` for RAII cleanup. Unlike `cache.rs` and `export.rs` tests, this test at least uses the system temp dir instead of the project tree.

```rust
    #[test]
    fn test_load_custom_strategies_multi() {
```
- **Sorting test (line 1405)**: `configs.sort_by(|a, b| a.name.cmp(&b.name))` — sorts after loading from map to make order deterministic for assertion. This is reasonable.

### Phase 6d Summary
- `backtest_portfolio` rebalance frequency logic is correct but uses brittle year-boundary checks.
- `format_backtest_table`: Fragile `cli_table` error handling, same antipattern as `optimization.rs`.
- Portfolio `BacktestMetrics`: Populated with meaningless prediction placeholders (`prediction_accuracy: 1.0`).
- Tests: `test_calculate_r2` misses the constant-series bug. `test_backtest_portfolio_frequencies` only checks inequality, not expected ordering. `test_run_backtest_for_asset` asserts on coin-split implementation detail.

---

# Phase 7: plot.rs

## File: src/plot.rs (14260 bytes, 494 lines)

### Line-by-Line Issues

**Lines 7-29: `hsl_to_rgb`**
```rust
fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
```
- **Duplicated logic**: plotters already provides color utilities. Custom HSL-to-RGB conversion is unnecessary unless specific color behavior is needed. `plotters` `RGBColor` can be constructed directly from HSL-like parameters or use existing palette functions.
- **Out-of-range h (line 11-23)**: `h` can be any f64 (e.g., 400.0). The modulo `% 2.0` on line 9 handles wrapping for the x component, but `if h < 60.0` etc. on lines 11-22 only covers 0-360. If `h` is 400, it falls through to the `else` branch (c=0, x=0), producing black. The caller `get_distinct_color` computes `h = (i as f64) * (360.0 / total as f64)`, which is guaranteed 0-360, so this is safe in practice. But `hsl_to_rgb` itself is fragile.

**Lines 31-40: `get_distinct_color`**
```rust
pub fn get_distinct_color(i: usize, total: usize) -> RGBColor {
    if total == 0 {
        return RGBColor(0, 0, 0);
    }
    let h = (i as f64) * (360.0 / total as f64);
    let s = 0.8;
    let l = if i.is_multiple_of(2) { 0.45 } else { 0.60 };
```
- **Lightness alternation (line 37)**: `if i.is_multiple_of(2)` alternates between two lightness values. For large `total`, adjacent colors may still be too similar. Standard color theory recommends using golden angle (~137.5 degrees) instead of equal spacing for maximum perceptual distinctness.
- **Edge case**: `total == 1` returns black (0,0,0). A single series plotted in black on a white background is invisible. Should return a high-contrast color like red or blue.

**Lines 42-130: `plot_line_chart`**
```rust
    let date_column = df.column("date")?.str()?;
    let n_rows = df.height();
    if n_rows == 0 {
        return Err(anyhow!("Cannot plot empty DataFrame"));
    }
```
- **Date column assumption (line 52)**: Assumes `"date"` column exists and is string type. If the DataFrame has no `"date"` column or it's a different type, this returns an opaque error. Should validate and provide context.

```rust
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;
    for &col_name in columns {
        let series = df.column(col_name)?;
        let f_array = series.f64()?;
        let col_min = f_array.min().unwrap_or(0.0);
        let col_max = f_array.max().unwrap_or(100.0);
```
- **Default values (lines 64-65)**: `unwrap_or(0.0)` for min and `unwrap_or(100.0)` for max. If a column is all nulls, its min/max becomes 0/100, which skews the y-axis. If all columns are null, y_min=0, y_max=100, which is reasonable. But if one column has real values and another is all nulls, the null column's 0/100 dominates the axis range. Should skip null columns or handle them separately.

```rust
    let label_step = (n_rows / 8).max(1);
```
- **Label density (line 93)**: `n_rows / 8` labels. For 10,000 rows, this is 1,250 labels. The x-axis will be overcrowded. Should have an upper bound or use adaptive labeling.

```rust
        .x_label_formatter(&|&idx| {
            if idx < n_rows && idx % label_step == 0 {
                date_column.get(idx).unwrap_or("").to_string()
            } else {
                "".to_string()
            }
        })
```
- **Panic risk (line 99)**: `date_column.get(idx).unwrap_or("")`. If `date_column` is shorter than expected, `get(idx)` returns `None` for out-of-bounds, and the formatter returns `""`. This is safe but the series plot may have fewer points than `n_rows` if some values are null. The x-axis labels are based on `n_rows`, but the line series uses `filter_map` to skip nulls (line 116). This means if row 100 has a null price, the line series jumps from row 99 to row 101, but the x-axis still shows index 100. The visual line has a gap but the axis label doesn't reflect the gap.

**Lines 132-179: `plot_performance`**
```rust
        // Find first valid non-zero price
        let mut first_val = None;
        for &val in values.iter().flatten() {
            if val != 0.0 {
                first_val = Some(val);
                break;
            }
        }
```
- **First valid non-zero (lines 151-157)**: Skips zero prices when finding the base. If the asset genuinely traded at $0.00 (e.g., a delisted token), normalization is skipped. This is sensible but undocumented.
- **Missing asset handling**: If `first_val` is `None` (all values are zero or null), the asset is silently omitted from the performance chart. No warning or error is returned. The caller doesn't know why an expected asset is missing.

**Lines 181-207: `get_log_return_stats`**
```rust
    let variance: f64 = log_returns.iter()
        .map(|&x| { let diff = x - mean; diff * diff })
        .sum::<f64>() / log_returns.len() as f64;
```
- **Population variance (line 204)**: Uses `N` (not `N-1`). This is inconsistent with `analysis.rs` `calculate_bollinger_bands` (population) and `calculate_orderbook_metrics` (sample). Should document which convention is used. For log returns, population variance is common in finance, but the inconsistency should be explicit.

**Lines 209-293: `plot_risk_return`**
- **Stub function (line 222)**: `if let Some((risk, ret)) = get_log_return_stats(series)` — if `get_log_return_stats` returns `None` (insufficient data), the asset is silently skipped. No aggregate warning. The scatter plot may have fewer points than expected assets.
- **Y-axis label (line 271)**: `"Return (Mean %)"`. The values from `get_log_return_stats` are log returns in percentage form (multiplied by 100 on line 190). Correct.

**Lines 295-402: `plot_efficient_frontier`**
- **Color inconsistency**: `simulated_color` is `RGBColor(180, 190, 200).mix(0.5)` on line 363. `.mix(0.5)` with `WHITE`? Actually `RGBColor` has `.mix(alpha)`. This produces a semi-transparent gray. The alpha is ignored by `BitMapBackend` (no transparency support in BMP). The color is just gray.
- **Portfolio points (lines 370-393)**: Green for min vol, red for max Sharpe. Standard convention. Good.

**Lines 404-493: `plot_backtest_equity`**
```rust
    let padding = if y_max != y_min {
        (y_max - y_min) * 0.1
    } else {
        1000.0
    };
    let y_min = (y_min - padding).max(0.0);
    let y_max = y_max + padding;
```
- **Padding fallback (line 436)**: When `y_max == y_min` (flat equity curve), padding is `1000.0`. This means a flat curve at $10,000 gets y-axis 0 to 20,000, which is misleading. Should use a relative padding like `y_max * 0.1` or a smaller absolute value.

**Missing features**
- **No tests**: `plot.rs` has zero tests. There are `#[cfg(test)]` modules in other files but not here. Plotting functions are hard to unit test, but at least `get_distinct_color`, `hsl_to_rgb`, and `get_log_return_stats` could be tested.
- **No error context**: Functions return `Result<()>` but errors from `df.column()`, `std::fs::create_dir_all()`, `BitMapBackend::new()` propagate without `.with_context()`. Callers cannot tell which step failed.

### Phase 7 Summary
- `plot.rs` is functional but has fragility: `hsl_to_rgb` handles out-of-range `h` poorly. `get_distinct_color` returns black for single-series plots (invisible on white). `plot_line_chart` uses brittle defaults for null columns and overcrowded x-axis labels. `plot_performance` silently skips all-zero assets. `get_log_return_stats` uses population variance without documenting inconsistency with other modules. No tests for any function.

---

# Phase 8a: api/coingecko.rs

## File: src/api/coingecko.rs (12122 bytes, 356 lines)

### Line-by-Line Issues

**Lines 14-56: `CoinGeckoClient::new`**
```rust
    pub fn new(cache: Arc<dyn CacheBackend>) -> Result<Self> {
        let mut base_url = "https://api.coingecko.com/api/v3".to_string();
        let mut demo_key = std::env::var("COINGECKO_DEMO_API_KEY").ok();
        let mut pro_key = std::env::var("COINGECKO_PRO_API_KEY").ok();

        if demo_key.is_none() && pro_key.is_none() {
            if let Ok(key) = std::env::var("COINGECKO_API_KEY") {
                let key_type = std::env::var("COINGECKO_API_KEY_TYPE").unwrap_or_default();
                if key_type.to_lowercase() == "pro" {
                    pro_key = Some(key);
                } else {
                    demo_key = Some(key);
                }
            }
        }

        if pro_key.is_some() {
            base_url = "https://pro-api.coingecko.com/api/v3".to_string();
        }
```
- **Complex env var logic (lines 27-43)**: Three env vars (`COINGECKO_DEMO_API_KEY`, `COINGECKO_PRO_API_KEY`, `COINGECKO_API_KEY` + `COINGECKO_API_KEY_TYPE`). The fallback chain is hard to reason about. If `COINGECKO_DEMO_API_KEY` and `COINGECKO_API_KEY` are both set, `COINGECKO_DEMO_API_KEY` wins. This is surprising. No documentation of precedence.
- **Missing error on invalid key type (line 33)**: If `COINGECKO_API_KEY_TYPE` is set to something other than "pro" (e.g., "demo", "PRO", "unknown"), the key goes to `demo_key`. Only exact lowercase "pro" triggers pro mode. This is case-sensitive but documented only by code.

```rust
            client: Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
                .build()?,
```
- **Hardcoded user agent (line 47)**: `"Mozilla/5.0 (Windows NT 10.0; Win64; x64)"` — hardcoded Windows user agent. On Linux/macOS, this is misleading. Should use a generic user agent or make it configurable.

```rust
            ttl_secs: 300, // 5 minutes cache default
```
- **Hardcoded TTL (line 51)**: 300 seconds is reasonable for CoinGecko's rate limits, but it's not configurable except via `with_ttl`. If the caller forgets to call `with_ttl`, they get 5 minutes.

**Lines 73-154: `get_request`**
```rust
        let mut attempts = 0;
        let max_attempts = 4;
        let mut retry_delay = std::time::Duration::from_millis(10000);
```
- **Hardcoded retry parameters (lines 95-97)**: `max_attempts=4`, initial `retry_delay=10s`. Exponential backoff with factor 2 yields 10s, 20s, 40s. Total worst-case wait: 70 seconds before final failure. This is reasonable for rate limits but the values are magic numbers.

```rust
            if status == 429 {
                attempts += 1;
                if attempts >= max_attempts {
                    return Err(anyhow!(
                        "CoinGecko API Rate Limit Exceeded (429) after {} attempts",
                        max_attempts
                    ));
                }
                ...
                tokio::time::sleep(retry_delay).await;
                retry_delay *= 2;
                continue;
            }

            if !status.is_success() {
                return Err(anyhow!("CoinGecko API returned error status: {}", status));
            }
```
- **Retry only handles 429 (lines 113-139)**: 5xx errors (502, 503, 504) and network errors (reqwest::Error) are NOT retried. CoinGecko's pro API can return 502/503 during maintenance. The retry logic should also handle 5xx and transient network errors.
- **Missing timeout**: `self.client.get(...).send().await?` has no explicit timeout. If the HTTP connection stalls, the future hangs until TCP timeout (potentially minutes). Should use `reqwest::Client::timeout()`.

```rust
            let body = response.text().await?;

            // Validate response body parses as JSON before storing in cache
            if let Err(e) = serde_json::from_str::<serde_json::Value>(&body) {
                return Err(anyhow::anyhow!("Invalid JSON response from CoinGecko: {}, body: {}", e, body));
            }
```
- **Error message includes full body (line 145)**: If the response is large (e.g., 10MB), the error message includes the entire body. This can cause memory/display issues. Should truncate or omit the body in production.

**Lines 168-171: `get_coins_list`**
- **Expensive cache dependency**: `get_coins_list` calls `/coins/list` with `use_cache=true`. CoinGecko's coins list is ~25KB and rarely changes. 5-minute TTL means it's re-fetched 12 times per hour. A 24-hour TTL would be more appropriate.

**Lines 275-355: `check_coin_id`**
```rust
    pub async fn check_coin_id(&self, input: &str) -> Result<Option<Vec<CoinSuggestion>>> {
        let coin_list = self.get_coins_list().await?;
```
- **Performance bug (line 276)**: Calls `get_coins_list()` every time. This fetches the entire coins list from cache or network. For a CLI tool where the user might call `check_coin_id` multiple times in a session, this is inefficient. Should cache the coins list in memory for the lifetime of the `CoinGeckoClient`.

```rust
        if exact_id_found {
            return Ok(None);
        }
```
- **Inverted API semantics (line 291)**: Returns `Ok(None)` when exact ID is found, `Ok(Some(suggestions))` when input is ambiguous. The caller must know that `None` means "exact match found, no suggestions needed", while `Some` means "no exact match, here are suggestions". This is confusing. A clearer API would be `Result<CoinMatch>` with variants `Exact`, `Ambiguous(Vec<CoinSuggestion>)`, `NotFound`.

```rust
            let id = coin
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
```
- **Null poisoning (lines 299-313)**: `unwrap_or("")` replaces null JSON fields with empty strings. If CoinGecko returns a coin with null `id`, `symbol`, or `name`, it becomes an empty-string suggestion. The caller cannot distinguish "real empty string" from "null field".

```rust
        if suggestions.len() < 10 {
            for coin in &coin_list {
                ...
                if suggestions.iter().any(|s| s.id == id) {
                    continue;
                }

                if id.to_lowercase().contains(&input_lower)
                    || name.to_lowercase().contains(&input_lower)
                {
                    suggestions.push(CoinSuggestion { id, symbol, name });
                    if suggestions.len() >= 10 {
                        break;
                    }
                }
            }
        }
```
- **Substring matching is O(n*m) (lines 321-352)**: For each coin in `coin_list`, calls `id.to_lowercase().contains(&input_lower)` and `name.to_lowercase().contains(&input_lower)`. For a list of ~10,000 coins, this is 20,000 lowercase allocations. Should pre-lowercase the coin list once or use a case-insensitive substring library.

### Phase 8a Summary
- `CoinGeckoClient::new`: Complex env var precedence for API keys. Hardcoded Windows user agent. Hardcoded 5-minute TTL.
- `get_request`: No timeout on HTTP client. Retry only handles 429, not 5xx/network errors. Error messages include full response body.
- `get_coins_list`: 5-minute TTL is too aggressive for a rarely-changing list.
- `check_coin_id`: Calls `get_coins_list` every time (performance bug). Inverted return semantics (`Ok(None)` = exact match). `unwrap_or("")` on JSON fields. O(n*m) substring matching with per-iteration lowercase allocation.

---

# Phase 8b: api/yahoo.rs

## File: src/api/yahoo.rs (5450 bytes, 153 lines)

### Line-by-Line Issues

**Lines 14-25: `YahooClient::new`**
```rust
            client: Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
                .build()?,
            ...
            base_url: "https://query2.finance.yahoo.com/v8/finance/chart".to_string(),
            ttl_secs: 300, // 5 minutes cache default
```
- **Hardcoded user agent (line 18)**: Same Windows-specific user agent as `coingecko.rs`. Misleading on non-Windows platforms.
- **Hardcoded base_url (line 21)**: Yahoo's chart endpoint has changed multiple times (`query1.finance.yahoo.com`, `query2.finance.yahoo.com`). Hardcoding makes the client fragile to Yahoo API migrations.
- **Hardcoded TTL (line 22)**: 5 minutes. Same comment as coingecko.rs.

**Lines 42-145: `fetch_ticker_chart`**
```rust
        let url = format!(
            "{}/{}?period1={}&period2={}&interval=1d",
            self.base_url, ticker, from_timestamp, to_timestamp
        );
```
- **URL construction (lines 48-51)**: Uses string formatting instead of `reqwest::Url` builder. If `ticker` contains URL-special characters (e.g., `^`, `&`, `=`), the resulting URL is malformed. Should use `reqwest::Url::parse` + `.query_pairs_mut()` for proper percent-encoding.
- **Cache key inconsistency**: The cache key is the raw, unencoded URL. If the same ticker is requested through different code paths that encode differently, cache misses occur.

```rust
        let mut attempts = 0;
        let max_attempts = 4;
        let mut retry_delay = std::time::Duration::from_millis(2000);
```
- **Hardcoded retry params (lines 57-59)**: `max_attempts=4`, initial `retry_delay=2s`. Magic numbers.

```rust
                    if status == 429 || status.is_server_error() {
```
- **Retry on 429 + 5xx (line 68)**: **Better than `coingecko.rs`** which only retries 429. This handles 502/503/504 correctly.

```rust
                Err(e) => {
                    attempts += 1;
                    ...
                    tokio::time::sleep(retry_delay).await;
                    retry_delay *= 2;
                }
```
- **Network error retry (lines 118-142)**: Retries on `reqwest::Error`. Good.
- **Missing jitter**: All retry delays are deterministic (`2s`, `4s`, `8s`). If multiple requests fail simultaneously, they retry in lockstep (thundering herd). Should add random jitter.

```rust
                        body
                    )
```
- **Full body in error (lines 107-112)**: Same antipattern as `coingecko.rs` — includes entire response body in error message.

**Lines 147-152: `ping`**
```rust
    pub async fn ping(&self) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        let rounded_now = (now / 3600) * 3600;
        self.fetch_ticker_chart("^GSPC", rounded_now - 86400, rounded_now).await?;
        Ok(())
    }
```
- **Hardcoded `^GSPC` (line 150)**: If Yahoo blocks S&P 500 data or returns an error for this ticker, `ping` fails even if other tickers work. Should use a more reliable ticker or make it configurable.
- **Round-down to hour (line 149)**: `rounded_now` truncates to the hour. Yahoo chart API typically accepts daily intervals, so this is fine.

### Phase 8b Summary
- `YahooClient::new`: Hardcoded Windows user agent, hardcoded base_url (fragile to Yahoo API changes), hardcoded 5-minute TTL.
- `fetch_ticker_chart`: URL built via string formatting (no percent-encoding for ticker). Cache key uses raw unencoded URL. Retry logic is actually better than `coingecko.rs` (handles 429 + 5xx + network errors) but lacks jitter and `Retry-After` header handling. No HTTP timeout. Full response body included in error messages.
- `ping`: Hardcoded `^GSPC` ticker. No tests.

---

# Phase 9: ui.rs

## File: src/ui.rs (16464 bytes, 417 lines)

### Line-by-Line Issues

**Lines 4-20: `clear_terminal`**
```rust
#[cfg(windows)]
pub fn clear_terminal() {
    if std::process::Command::new("cmd")
        .args(["/c", "cls"])
        .status()
        .is_err()
    {
        print!("\x1B[2J\x1B[1;1H");
        let _ = std::io::Write::flush(&mut std::io::stdout());
    }
}

#[cfg(not(windows))]
pub fn clear_terminal() {
    print!("\x1B[2J\x1B[1;1H");
    let _ = std::io::Write::flush(&mut std::io::stdout());
}
```
- **Heavyweight Windows clear (lines 6-14)**: Spawns `cmd /c cls` as a subprocess just to clear the screen. On modern Windows terminals (Windows Terminal, VS Code terminal), ANSI escape codes work natively. The subprocess approach is slow and unnecessary.
- **Inconsistent fallback**: The Windows version only falls back to ANSI if `cmd` fails, but `cmd` almost never fails on Windows. The non-Windows version always uses ANSI. This means Windows users get the slow subprocess approach by default.

**Lines 22-30: `wait_for_back`**
```rust
pub fn wait_for_back() {
    println!();
    let options = &["[Back]"];
    let _ = dialoguer::Select::new()
        .with_prompt("Press enter/select option to go back")
        .default(0)
        .items(options)
        .interact_opt();
}
```
- **Overkill UI (lines 24-29)**: Uses `dialoguer::Select` with a single option "[Back]" just to wait for user input. A simple `read_line` or `press_any_key` would be more appropriate.
- **Result ignored**: `interact_opt()` returns `Option<usize>`, but the result is discarded with `let _ =`. If the user presses Ctrl+C, the behavior depends on dialoguer's terminal state handling.

**Lines 32-417: `run_interactive_menu`**
- **God function (385 lines)**: Entire interactive CLI logic in one function. Should be broken into smaller functions per menu action.

**Lines 117-129: `default_concurrency`**
```rust
                let default_concurrency = {
                    let is_pro = std::env::var("COINGECKO_PRO_API_KEY").is_ok()
                        || (std::env::var("COINGECKO_API_KEY").is_ok()
                            && std::env::var("COINGECKO_API_KEY_TYPE")
                                .unwrap_or_default()
                                .to_lowercase()
                                == "pro");
                    if is_pro {
                        3
                    } else {
                        1
                    }
                };
```
- **Duplicated env var logic (lines 117-129)**: Exact same API-key detection logic from `coingecko.rs::new` (lines 27-43). If precedence rules change, both locations must be updated. Should extract to a shared helper.
- **Magic numbers**: `3` for pro, `1` for free tier. Not documented why these values.

**Lines 153-199: Backtest defaults branching**
```rust
                let (strategy, fee, slippage, rebalance_frequency) = if backtest {
                    ...
                } else {
                    ("rsi".to_string(), 0.001, 0.0005, "daily".to_string())
                };
```
- **Hidden defaults when backtest=false (lines 198-199)**: Even when backtesting is disabled, `strategy`, `fee`, `slippage`, and `rebalance_frequency` are set to hardcoded defaults and passed to `pipeline::run_pipeline_flow`. These values are meaningless if `backtest=false`. Should use `Option` types or only set these fields when `backtest=true`.

**Lines 202-226: Pipeline config construction**
```rust
                if let Err(e) = pipeline::run_pipeline_flow(pipeline::PipelineConfig {
                    coin: &coin,
                    currency: &currency,
                    strategy: &strategy,
                    rebalance_frequency: &rebalance_frequency,
                    ...
                })
```
- **String reference lifetime risk (lines 202-222)**: `PipelineConfig` stores `&str` references to local variables (`coin`, `strategy`, `rebalance_frequency`). This works because the config is consumed immediately by `run_pipeline_flow`, but if the struct lifetime changes, this becomes a dangling reference bug. The API should accept `String` and let `PipelineConfig` own its data.

**Lines 240-271: "List Supported Coins"**
```rust
                            let price = c
                                .get("current_price")
                                .and_then(|v| v.as_f64())
                                .unwrap_or(0.0);
                            let market_cap =
                                c.get("market_cap").and_then(|v| v.as_f64()).unwrap_or(0.0);
```
- **Null poisoning (lines 253-258)**: `unwrap_or(0.0)` replaces null prices and market caps with 0.0. The printed table shows "$0.00" and "0" for missing data, which is misleading.

**Lines 341-369: "Check Coin ID"**
```rust
                    Ok(Some(suggestions)) => {
                        println!("Error: '{}' is not a valid CoinGecko ID.", coin_to_check);
```
- **Misleading user message (lines 351-352)**: When `check_coin_id` returns suggestions, the UI prints "Error: ... is not a valid CoinGecko ID." Suggestions are not errors — they're helpful hints for ambiguous input. Calling them "Error" is poor UX.

**Lines 370-404: "Settings" loop**
```rust
                    _ => unreachable!(),
```
- **Unnecessary `unreachable!()` (line 402)**: After the exhaustive match on `settings_choice` (which only has 2 variants), `unreachable!()` is technically correct but adds noise.

**Lines 405-410: Exit arm**
```rust
            _ => unreachable!(),
```
- **Unnecessary `unreachable!()` (line 409)**: Same as above. The outer `match choice` covers all 8 options from the array.

**Lines 412-414: Post-action wait logic**
```rust
        if choice != "Exit" && choice != "Settings" {
            wait_for_back();
        }
```
- **Inconsistent wait (lines 412-414)**: After returning from "Settings", the menu immediately redraws without waiting. After other actions, it waits. This inconsistency is minor but surprising.

**No input validation**
- `days` (line 87) is `u32` with default 90, but no upper bound check. User can enter `999999999`.
- `fee` (line 168) and `slippage` (line 173) are `f64` with no validation that they're positive or less than 1.0.
- `seed_str.parse::<u64>().ok()` (line 114) silently fails on invalid input, returning `None` without warning the user.
- `ann_factor_str.parse::<f64>().ok()` (line 145) silently fails, returning `None` without warning.

**No tests**: `ui.rs` has zero tests.

### Phase 9 Summary
- `clear_terminal`: Spawns `cmd /c cls` subprocess on Windows instead of using ANSI codes. Slow and unnecessary.
- `run_interactive_menu`: 385-line god function. Duplicated API-key detection logic from `coingecko.rs`. Passes hardcoded backtest defaults even when backtest is disabled. Stores borrowed string references in `PipelineConfig`.
- `wait_for_back`: Uses `dialoguer::Select` with a single item — overkill for "press enter".
- Input prompts: No validation on numeric inputs. Silent parse failures.
- UI messages: Calls suggestions "Error" in `Check Coin ID`.
- No tests.

---

# Phase 10a: pipeline.rs — Config, Setup, Data Fetching (lines 1-300)

## File: src/pipeline.rs (49668 bytes, 770 lines) — Part 1 of 3

### Line-by-Line Issues

**Lines 5-25: `PipelineConfig`**
```rust
pub struct PipelineConfig<'a> {
    pub coin: &'a str,
    pub currency: &'a str,
    ...
    pub strategy: &'a str,
    pub rebalance_frequency: &'a str,
}
```
- **Lifetime parameter (line 5)**: `PipelineConfig<'a>` forces callers to manage string lifetimes. `ui.rs` works around this by passing `&coin` etc. from local variables (lines 202-222). This is fragile and forces `ui.rs` to keep all input strings alive until the config is consumed. Should accept `String` and own its data.

**Lines 27-63: `run_pipeline_flow`**
```rust
pub async fn run_pipeline_flow(mut config: PipelineConfig<'_>) -> Result<()> {
    if config.light {
        config.days = 30;
    }

    let coin = config.coin;
    let currency = config.currency;
    ...
```
- **Unnecessary destructuring (lines 33-63)**: Copies every field from `config` into local variables. `config` is already `mut` and accessible. These locals add boilerplate without benefit.

```rust
    let concurrency = config.concurrency.unwrap_or_else(|| {
        let is_pro = std::env::var("COINGECKO_PRO_API_KEY").is_ok()
            || (std::env::var("COINGECKO_API_KEY").is_ok()
                && std::env::var("COINGECKO_API_KEY_TYPE")
                    .unwrap_or_default()
                    .to_lowercase()
                    == "pro");
        if is_pro {
            3
        } else {
            1
        }
    });
```
- **Triplicated API key logic (lines 46-57)**: This is the THIRD copy of the CoinGecko API key detection logic. Appears in `coingecko.rs::new` (lines 27-43), `ui.rs::run_interactive_menu` (lines 117-129), and now here. Should be extracted to a single helper function or constant.

**Lines 65-71: Directory creation**
```rust
    let run_dir = format!("{}/run_{}", output_dir, timestamp);
    crate::utils::validate_safe_path(&run_dir)?;
    std::fs::create_dir_all(&run_dir)?;
```
- **Missing error context**: `create_dir_all` failure (disk full, permission denied) propagates without context. Caller cannot tell which directory creation failed.

**Lines 95-107: Ping handling**
```rust
    match cg_client.ping().await {
        Ok(_) => println!("CoinGecko API Connection: OK"),
        Err(e) => println!("Warning: CoinGecko API Connection Failed: {}", e),
    }
```
- **Ping failure is non-fatal**: If CoinGecko is unreachable, the pipeline continues. This might be intentional (to allow Yahoo-only mode), but if the user requested only CoinGecko coins, the pipeline will fail later with a less clear error.

**Lines 112-124: `fetch_days` logic**
```rust
    let fetch_days = if backtest {
        if days <= 40 {
            90
        } else if days <= 130 {
            180
        } else if days <= 315 {
            365
        } else {
            days + 50
        }
    } else {
        days
    };
```
- **Magic numbers (lines 113-121)**: `40 -> 90`, `130 -> 180`, `315 -> 365`. No explanation of why these thresholds exist. Likely related to indicator warmup periods (e.g., need extra days for 200-day SMA), but this should be documented or computed from the longest indicator period.

**Lines 148-200: Coin validation loop**
```rust
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("404") {
                    println!(
                        "Error: Coin '{}' is not a valid CoinGecko ID. Use '--check-coin {}' to search for suggested IDs. Skipping.",
                        c, c
                    );
                } else {
                    println!(
                        "Warning: Failed to fetch tickers for {}: {}. Keeping in pipeline.",
                        c, e
                    );
                    coins.push(c.to_string());
                }
            }
```
- **Asymmetric error handling (lines 184-199)**: 404 errors cause the coin to be skipped (not added to `coins`). All other errors keep the coin in the pipeline. This means transient network errors are treated as "coin is valid, keep going", while 404s are treated as "coin is invalid, skip". The behavior is inconsistent.

**Lines 217-280: Async task spawning**
```rust
    for (c, curr) in tasks {
        let sem = semaphore.clone();
        let client = cg_client_arc.clone();
        let ohlcv_dir_clone = ohlcv_dir.clone();
        let days_str = fetch_days.to_string();
        let raw_format_clone = raw_format.to_string();

        join_set.spawn(async move {
            let _permit = sem.acquire().await?;

            let cg_val = client
                .get_coin_market_chart_range(&c, &curr, from_timestamp, to_timestamp)
                .await?;
            let cg_json_str = serde_json::to_string(&cg_val)?;

            let c_safe = crate::utils::sanitize_name(&c);
            let curr_safe = crate::utils::sanitize_name(&curr);
            crate::utils::validate_safe_path(&c_safe)?;
            crate::utils::validate_safe_path(&curr_safe)?;
```
- **Cloning overhead (lines 228-233)**: Clones `String` for each task. For 50 tasks, this is 100 small allocations. Acceptable but could be reduced by borrowing.
- **`from_timestamp` and `to_timestamp`**: These are captured by reference from the outer scope. Since they're `i64` (Copy), they're implicitly copied into the async block. Correct.

```rust
            let ohlc_val = client.get_coin_ohlc(&c, &curr, &days_str).await?;

            if raw_format_clone == "json" {
                let ohlc_json_pretty = serde_json::to_string_pretty(&ohlc_val)?;
                let json_file_path = format!("{}/{}_{}.json", ohlcv_dir_clone, c_safe, curr_safe);
                crate::utils::validate_safe_path(&json_file_path)?;
                std::fs::write(&json_file_path, &ohlc_json_pretty)?;
            } else if raw_format_clone == "csv" {
                let csv_file_path = format!("{}/{}_{}.csv", ohlcv_dir_clone, c_safe, curr_safe);
                crate::utils::validate_safe_path(&csv_file_path)?;
                let mut wtr_ohlcv = std::fs::File::create(&csv_file_path)?;
                writeln!(wtr_ohlcv, "timestamp,open,high,low,close")?;
                for row in &ohlc_val {
                    if row.len() >= 5 {
                        writeln!(
                            wtr_ohlcv,
                            "{},{},{},{},{}",
                            row[0], row[1], row[2], row[3], row[4]
                        )?;
                    }
                }
            }
```
- **File I/O inside async task (lines 253-272)**: Performs synchronous file writes inside async tasks. Under high concurrency, this can block async executor threads. Should use `tokio::fs` or offload to `spawn_blocking`.

```rust
            let df = analysis::align_datasets(&df_market, &[df_ohlc], false)?;
            Ok::<(polars::prelude::DataFrame, String), anyhow::Error>((df, price_col_name))
```
- **`align_datasets` with `drop_weekends=false` (line 277)**: Always passes `false`. The `drop_weekends` config from `PipelineConfig` is ignored here. The `false` is hardcoded.

**Lines 285-299: Result collection**
```rust
    while let Some(res) = join_set.join_next().await {
        match res {
            Ok(Ok((df, col_name))) => {
                pb.set_message(format!("Loaded {} rows for {}", df.height(), &col_name));
                currency_dfs.push(df);
                currency_cols.push(col_name);
            }
            Ok(Err(e)) => {
                return Err(e);
            }
            Err(e) => {
                return Err(anyhow!("Join error: {}", e));
            }
        }
    }
```
- **All-or-nothing failure (lines 292-298)**: If ANY task fails, the entire pipeline returns `Err`. One bad coin kills all others. Should collect partial results and report which coins failed.

### Phase 10a Summary
- `PipelineConfig<'a>`: Borrowed string lifetimes force fragile patterns in `ui.rs` and `main.rs`. Should own its strings.
- `run_pipeline_flow`: Unnecessary field destructuring. Triplicated API-key detection logic.
- Ping failure is non-fatal, which may hide CoinGecko unavailability until later.
- `fetch_days` uses magic-number thresholds for backtest data padding.
- Coin validation has asymmetric error handling (404 = skip, other errors = keep).
- Async tasks perform synchronous file I/O.
- `align_datasets` hardcodes `drop_weekends=false`, ignoring the pipeline config.
- Task collection uses all-or-nothing failure semantics.

---

# Phase 10b: pipeline.rs — Merge, Yahoo, Indicators, Export, Plot, Optimization, Backtest Setup (lines 300-500)

## File: src/pipeline.rs (49668 bytes, 770 lines) — Part 2 of 3

### Line-by-Line Issues

**Lines 301-310: Merge currency DataFrames**
```rust
    let mut main_df = currency_dfs[0].clone();
    if currency_dfs.len() > 1 {
        main_df = analysis::align_datasets(&main_df, &currency_dfs[1..], false)?;
    }
```
- **Hardcoded `drop_weekends=false` (line 309)**: Even though `drop_weekends` is a pipeline config option, the first merge hardcodes `false`. The later merge on line 345 correctly uses the config variable. This inconsistency means multi-currency pipelines always skip weekend dropping during the initial merge.

**Lines 312-341: Fetch Yahoo Benchmarks**
```rust
        let bench_tickers = vec!["^GSPC", "^DJI", "^IXIC", "^HSI", "^BVSP", "^TNX"];
```
- **Hardcoded tickers (line 318)**: Six specific Yahoo Finance tickers are baked into the pipeline. If Yahoo changes a ticker symbol, delists an index, or the user wants different benchmarks, the only way to change this is to edit source code. Should be configurable.
- **Silent failure per ticker (lines 333-339)**: If a benchmark fetch fails, `pb.println` prints the error and continues. If ALL benchmarks fail, `other_dfs` is empty and `align_datasets` on line 345 succeeds trivially. The user gets no summary of how many benchmarks succeeded.

**Lines 343-357: Align and compute indicators**
```rust
    let aligned_df = analysis::align_datasets(&main_df, &other_dfs, drop_weekends)?;
```
- **Uses config for `drop_weekends` (line 345)**: Correct. But inconsistent with line 309.
- **Indicator computation loop (lines 354-356)**: Calls `compute_returns_and_indicators` for each coin. Each call internally does `df.hstack(&new_cols)?`, which appends columns. After N coins, the DataFrame has N * M indicator columns plus original columns. This is correct but means each iteration reallocates the column array. For many coins, this is O(N^2) reallocations.

**Lines 365-373: Export datasets**
```rust
    export::export_csv(&mut final_df, &csv_path)?;
    export::export_parquet(&mut final_df, &parquet_path)?;
```
- **Unnecessary `mut` (lines 370, 373)**: `export_csv` and `export_parquet` only read from `final_df`. Passing `&mut` forces mutable borrow, blocking concurrent reads. Should accept `&DataFrame`.

**Lines 375-420: Plotting**
```rust
            let returns_cols_refs: Vec<&str> = returns_cols.iter().map(|s| s.as_str()).collect();
```
- **Unnecessary Vec allocation (line 382)**: `plot_line_chart` takes `&[&str]`. The caller allocates a `Vec<&str>` just to pass a slice. If `plot_line_chart` accepted `&[String]` or `&[&str]` through a generic, this allocation would be unnecessary.

**Lines 424-477: Portfolio Optimization**
```rust
        match crate::optimization::run_monte_carlo(&final_df, &currency_cols, final_ann_factor, 10000, seed) {
```
- **Hardcoded 10000 simulations (line 433)**: Not configurable. 10,000 is reasonable but should be a parameter.

**Lines 479-500: Backtest setup**
```rust
        let mut backtest_metrics = Vec::new();
        let mut custom_configs = Vec::new();
        let mut strats = Vec::new();
        if strategy.ends_with(".json") {
            let configs = backtest::load_custom_strategies(strategy)?;
            for cfg in configs {
                strats.push(cfg.name.clone());
                custom_configs.push(Some(cfg));
            }
        } else if strategy.to_lowercase() == "all" {
            strats = vec!["rsi".to_string(), "macd".to_string(), "bollinger".to_string()];
            custom_configs = vec![None, None, None];
        } else {
            strats = vec![strategy.to_lowercase()];
            custom_configs = vec![None];
        }
```
- **Zipping two parallel Vecs (lines 486-501)**: `strats` and `custom_configs` are maintained in parallel. If the lengths ever diverge (e.g., a future "all" mode accidentally pushes 4 elements to one but 3 to the other), `zip` silently truncates. A single struct or enum would be safer.

```rust
        let mut asset_bh_caches: std::collections::HashMap<String, Option<backtest::BhCache>> = std::collections::HashMap::new();
```
- **Verbose HashMap (line 507)**: `std::collections::HashMap::new()` is verbose. Use `HashMap::new()` with type inference.

### Phase 10b Summary
- `align_datasets`: First merge hardcodes `drop_weekends=false`, second uses config. Inconsistent.
- Yahoo benchmark fetch: Hardcoded tickers, silent per-ticker failure with no summary.
- Indicator loop: Correct but O(N^2) column reallocations for many coins.
- Export: Unnecessary `&mut` borrow on DataFrames.
- Plotting: Unnecessary Vec allocation for column references.
- Optimization: Hardcoded 10,000 simulation count.
- Backtest setup: Parallel Vecs for `strats`/`custom_configs` are fragile.

---

# Phase 10c: pipeline.rs — Backtest Loop, Reports, run_ohlcv_flow, run_standalone_backtest (lines 500-770)

## File: src/pipeline.rs (49668 bytes, 770 lines) — Part 3 of 3

### Line-by-Line Issues

**Lines 509-552: Backtest individual assets**
```rust
            for (strat, custom_cfg) in strats.iter().zip(custom_configs.iter()) {
                let cache_entry = asset_bh_caches.entry(col.to_string()).or_insert(None);
                match backtest::run_backtest_for_asset(
                    &final_df,
                    col,
                    strat,
                    custom_cfg.as_ref(),
                    fee,
                    slippage,
                    final_ann_factor,
                    days as usize,
                    cache_entry,
                ) {
                    Ok((metrics, equity, bh_equity)) => {
                        backtest_metrics.push(metrics);

                        // Save PNG plot
                        let dates: Vec<String> = final_df
                            .column("date")?
                            .str()?
                            .into_iter()
                            .map(|opt| opt.unwrap_or("").to_string())
                            .collect();
                        let plot_path = format!("{}/{}_{}_backtest.png", backtest_dir, col, strat);
                        let active_dates = dates[dates.len() - equity.len()..].to_vec();
```
- **Redundant date extraction (lines 528-535)**: For every asset-strategy combination, the code extracts the entire `date` column from `final_df`, converts all rows to `String`, then slices to `active_dates`. If there are 5 coins and 3 strategies, this does 15 full column extractions. `plot_backtest_equity` already receives `bh_equity` and `equity`, which have the same length as the active dates. The dates could be extracted once per coin outside the strategy loop.

**Lines 554-636: US Treasury benchmark**
```rust
        if final_df.column("^TNX").is_ok() {
            let n_rows = final_df.height();
            ...
            let treasury_metrics = backtest::BacktestMetrics {
                coin: "US_TREASURY".to_string(),
                currency: "10Y".to_string(),
                strategy: "B&H".to_string(),
                ...
                prediction_accuracy: 1.0,
                prediction_r2: 0.0,
                active_win_rate: 1.0,
                prediction_rating: "N/A".to_string(),
                strategy_rating: "good".to_string(),
                ...
            };
```
- **Placeholder metrics (lines 614-634)**: Treasury benchmark uses meaningless placeholder values (`prediction_accuracy: 1.0`, `active_win_rate: 1.0`, `prediction_rating: "N/A"`). These fields don't apply to a buy-and-hold benchmark. Populating them with fake values pollutes the report.
- **Hardcoded "good" rating (line 628)**: Treasury always gets `strategy_rating: "good"` regardless of actual return. If the treasury lost money, it would still be rated "good".

**Lines 638-712: Portfolio backtests**
```rust
            match backtest::backtest_portfolio(
                &final_df,
                &currency_cols,
                &opt_res.max_sharpe.weights,
                "max_sharpe",
                final_ann_factor,
                days as usize,
                fee,
                slippage,
                rebalance_frequency,
            ) {
```
- **Passes `rebalance_frequency` as `&str` (line 657)**: `backtest_portfolio` takes `rebalance_frequency: &str`. This is a borrowed string that must outlive the call. In this context it's fine because `rebalance_frequency` is owned by `config`, but it adds lifetime complexity.
- **Same plot path issue (lines 662-705)**: Extracts dates again for each portfolio backtest. Could be extracted once.

**Lines 714-767: Report export**
```rust
            let mut report_map = std::collections::HashMap::new();
            for m in &backtest_metrics {
                report_map.insert(format!("{}_{}", m.coin, m.strategy), m.clone());
            }
```
- **HashMap insertion overwrites duplicates (line 756)**: If two metrics have the same `coin_strategy` key (e.g., if a custom strategy name collides with a built-in one), the later one silently overwrites the earlier. No warning.
- **JSON write silently ignored (line 763)**: `std::fs::write(...).is_ok()` returns `true` on success. If write fails, the `if` body is skipped and no error is printed. The user never knows the JSON report wasn't saved.

```rust
    println!("CDG data pipeline completed successfully!");
    Ok(())
```
- **Unconditional success message (line 770)**: Even if backtests failed, plots failed, or reports failed to export, the final message says "completed successfully". Should reflect actual status.

**Lines 774-892: `run_ohlcv_flow`**
```rust
    let sanitized_coin = crate::utils::sanitize_name(coin);
    let sanitized_currency = crate::utils::sanitize_name(currency);
    crate::utils::validate_safe_path(&sanitized_coin)?;
    crate::utils::validate_safe_path(&sanitized_currency)?;
```
- **Sanitization before validation (lines 784-787)**: Sanitizes coin/currency, then validates the sanitized names. If sanitization changes the string significantly (e.g., removes special chars), the validation is checking a different string than what was requested.

```rust
    if raw_format == "json" {
        ...
    } else if raw_format == "csv" {
        ...
    }
```
- **Duplicated file-writing logic (lines 803-828)**: This JSON/CSV export block is nearly identical to the one in `run_pipeline_flow` lines 253-272. Duplicated across two functions. Should extract to a helper.

```rust
        _ => {
            // stdout
            println!(
                "{:<20} | {:<10} | {:<10} | {:<10} | {:<10}",
                "Timestamp", "Open", "High", "Low", "Close"
            );
```
- **Magic truncation (line 874)**: `for row in ohlc_data.iter().take(50)` silently truncates to 50 rows for stdout. No warning that data was truncated.

**Lines 894-1174: `run_standalone_backtest`**
```rust
pub async fn run_standalone_backtest(
    db_path: &str,
    output_dir: &str,
    _output_prefix: &str,
    cache_ttl: i64,
    coin: &str,
    currency: &str,
    days: u32,
    strategy: &str,
    fee: f64,
    slippage: f64,
    _rebalance_frequency: &str,
) -> Result<()> {
```
- **Unused parameter (line 905)**: `_rebalance_frequency: &str` is accepted but never used in the function body. The parameter is prefixed with `_` to suppress warnings, but it suggests the API was designed with portfolio backtesting in mind and then simplified. Should either use it or remove it.
- **Copy-paste of pipeline backtest block (lines 989-1120)**: The entire backtest loop, treasury benchmark, and report generation is duplicated from `run_pipeline_flow` lines 480-768. This is a maintenance trap—if backtest logic changes, both copies must be updated.

```rust
            let final_treasury_return = (cum_yield - 1.0) * 100.0;
            let treasury_metrics = backtest::BacktestMetrics {
                ...
                prediction_accuracy: 1.0,
                prediction_r2: 0.0,
                active_win_rate: 1.0,
                prediction_rating: "N/A".to_string(),
                strategy_rating: "good".to_string(),
                ...
            };
```
- **Same placeholder metrics as pipeline (lines 1096-1116)**: Treasury benchmark populates prediction fields with meaningless values.

**Lines 1007-1010: run_dir creation**
```rust
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let run_dir = format!("{}/backtest_run_{}", output_dir, timestamp);
```
- **Clock skew risk**: Uses `chrono::Local::now()`. If the system clock changes between `run_pipeline_flow` and `run_standalone_backtest`, the timestamps differ. Both should use `Utc::now()` or a monotonic source.

### Phase 10c Summary
- Backtest loop: Redundant full date column extraction per coin-strategy pair.
- Treasury benchmark: Hardcoded placeholder metrics with fake ratings.
- Portfolio backtest: Borrowed string lifetimes, redundant date extraction.
- Report export: HashMap key collisions silently overwrite. JSON write failure silently ignored. Unconditional "success" message.
- `run_ohlcv_flow`: Sanitization-before-validation order. Duplicated file-writing logic. Stdout truncates to 50 rows silently.
- `run_standalone_backtest`: Unused `_rebalance_frequency` parameter. Near-complete copy-paste of backtest/report logic from `run_pipeline_flow`. Same placeholder treasury metrics.

---

# Phase 11: main.rs

## File: src/main.rs (19384 bytes, 554 lines)

### Line-by-Line Issues

**Lines 5-30: `Cli` struct**
```rust
#[derive(Parser, Debug, PartialEq)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
```
- **Duplicated CLI definitions**: `Cli` defines `RunPipeline` and `Backtest` subcommands with many overlapping fields (`coin`, `currency`, `days`, `strategy`, `fee`, `slippage`, `rebalance_frequency`). `Backtest` is essentially `RunPipeline` without most fields. This duplication means adding a new field requires updating both subcommands.
- **`output_prefix` logic duplicated elsewhere**: `main.rs` computes `db_path` and `output_prefix` from `output_dir` on lines 176-181. The same derivation logic appears in `ui.rs` and tests in `main.rs`. Should be a helper function.

**Lines 154-161: Ctrl+C handler**
```rust
    tokio::spawn(async {
        tokio::signal::ctrl_c().await.ok();
        println!("\nOperation cancelled by user.");
        std::process::exit(0);
    });
```
- **Critical bug (line 160)**: `std::process::exit(0)` immediately terminates the process without dropping async tasks, flushing write buffers, or releasing database connections. Under Tokio, this can corrupt the SQLite cache or leave temp files half-written. Should use a cancellation token (`CancellationToken`) to gracefully shutdown async tasks.

**Lines 165-174: `raw_format` validation**
```rust
    let raw_format = args.raw_format.to_lowercase();
    if raw_format != "json" && raw_format != "csv" {
        return Err(anyhow::anyhow!(
            "Invalid raw format: '{}'. Must be 'json' or 'csv'.",
            raw_format
        ));
    }
```
- **Good**: Validates input early.

**Lines 176-181: Path derivation**
```rust
    let db_path = args
        .db_path
        .unwrap_or_else(|| format!("{}/cache.db", output_dir));
    let output_prefix = args
        .output_prefix
        .unwrap_or_else(|| format!("{}/output", output_dir));
```
- **Duplicated logic**: Same derivation appears in `test_dynamic_path_resolution` (lines 427-476) and `ui.rs`. Should be a function like `resolve_paths(output_dir, db_path_opt, output_prefix_opt)`.

**Lines 200-220: `RunPipeline` arm**
```rust
            pipeline::run_pipeline_flow(pipeline::PipelineConfig {
                coin: &coin,
                currency: &currency,
                ...
                strategy: &strategy,
                fee,
                slippage,
                rebalance_frequency: &rebalance_frequency,
            })
```
- **Borrowed string lifetimes**: `PipelineConfig<'a>` requires all string fields to be borrowed. `main.rs` works around this by passing references to local variables. This is fragile and tightly couples `main.rs` to `PipelineConfig`'s lifetime parameter.

**Lines 247-265: `Ping` arm**
```rust
            match cg_client.ping().await {
                Ok(val) => println!("CoinGecko Connection: OK, response: {:?}", val),
                Err(e) => println!("CoinGecko Connection Failed: {}", e),
            }
```
- **Different output format**: CoinGecko ping prints the response value, while Yahoo ping (line 262) prints only "OK". Inconsistent output format.

**Lines 342-368: `CheckCoin` arm**
```rust
                Ok(Some(suggestions)) => {
                    println!("Error: '{}' is not a valid CoinGecko ID.", coin);
```
- **Misleading message**: Calls suggestions an "Error". Same UX issue as `ui.rs` line 351.

**Lines 369-378: Interactive mode fallback**
```rust
        None => {
            ui::run_interactive_menu(
                &db_path,
                &output_dir,
                &output_prefix,
                &raw_format,
                args.cache_ttl,
            )
            .await?;
        }
```
- **Unused subcommand fallback**: When no subcommand is provided, falls back to interactive TUI. This is reasonable but means `cdg` with no args starts an interactive session, which is surprising for a CLI-first tool. Should be explicit in help text.

**Lines 384-553: Tests**
```rust
    #[test]
    fn test_dynamic_path_resolution() {
```
- **Duplicated test logic (lines 427-476)**: Reimplements the same path derivation logic as `main.rs` lines 176-181. If derivation logic changes, both locations must be updated. Tests should call a shared function.

```rust
    #[test]
    fn test_default_concurrency_resolution() {
```
- **Env var pollution (lines 497-553)**: Manually saves/restores env vars. If the test panics between save and restore, the env vars leak. Should use a helper that guarantees restoration (e.g., `temp_env` crate or RAII guard).

**Structural issues**
- **God-function `main`**: 227 lines of `match` arms. Each arm repeats cache/client initialization. Should extract to a helper like `fn init_clients(db_path: &str, ttl: i64) -> Arc<dyn CacheBackend + Send + Sync>`.

### Phase 11 Summary
- `Cli`: Duplicated fields across subcommands. Path derivation logic duplicated 3 times.
- `main`: 227-line match god-function. Ctrl+C handler uses `std::process::exit(0)`, abandoning async cleanup.
- `CheckCoin` UX: Calls suggestions "Error".
- Tests: Env var pollution risk. Reimplements path logic instead of sharing it.

---

