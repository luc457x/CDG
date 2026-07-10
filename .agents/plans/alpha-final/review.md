# Alpha Final Code Review — CDG

**Reviewer:** Kilo (automated QA)  
**Date:** 2026-07-10  
**Scope:** Full `src/` audit + test execution + dependency/GCP/headless compatibility check  
**Verdict:** **NOT ALPHA-READY** — 5 critical blockers remain unfixed.

---

## 1. Verified Summary

| Check | Result |
|-------|--------|
| `cargo build` | Clean, 0 warnings |
| `cargo test` | 113 pass / 0 fail |
| `unsafe` in `src/` | None (only in `target/` libsqlite3-sys bindgen) |
| `.env` tracked in git | **False** — ignored and absent from worktree/index |
| CI/CD present | **False** — no `.github/workflows/` |
| Dockerfile present | **False** — none found |
| `cargo audit` installed | **False** — not installed |

Earlier review drafts contained unverified claims about `.env` tracking and a Dockerfile. Those were incorrect. The findings below are based on direct file inspection and command output.

---

## 2. Critical Blockers (P0) — Must Fix Before Alpha Tag

### B1: Treasury benchmark emits `NaN` and `"n/a"` in reports
**File:** `src/backtest.rs:1049-1069`

```rust
metrics.push(BacktestMetrics {
    coin: "US_TREASURY".to_string(),
    currency: "10Y".to_string(),
    strategy: "B&H".to_string(),
    prediction_accuracy: f64::NAN,   // <-- breaks JSON/CSV parsers
    prediction_r2: f64::NAN,         // <-- breaks ML ingestion
    active_win_rate: f64::NAN,       // <-- breaks numeric schemas
    prediction_rating: "n/a".to_string(), // <-- string in numeric field
    ...
});
```

**Impact:** Any downstream data pipeline (BigQuery, Vertex AI, pandas) ingesting `backtest_report.json` will fail on `NaN` floats. The `"n/a"` string violates the otherwise-numeric `prediction_rating` schema.

**Fix:** Replace `f64::NAN` with `0.0` or `None` via a custom serializer. Replace `"n/a"` with `"none"` or empty string.

---

### B2: No CI/CD pipeline
**Finding:** No `.github/workflows/` directory exists.

**Impact:** No automated test gate, no dependency audit, no reproducible release process. For a financial ML tool, shipping without CI is unacceptable.

**Fix:** Add `.github/workflows/ci.yml` running `cargo test`, `cargo fmt -- --check`, `cargo clippy`, and `cargo deny` or `cargo audit`.

---

### B3: `native-tls` prevents hermetic GCP container builds
**File:** `Cargo.toml:8`

```toml
reqwest = { version = "0.12", features = ["json", "native-tls"] }
```

**Impact:** `native-tls` requires system OpenSSL/LibreSSL. Minimal GCP base images (distroless, alpine without `openssl-dev`) will fail to build. This violates the AGENTS.md GCP compatibility directive.

**Fix:** Switch to `rustls` feature, or enable `native-tls` with `vendored` to bundle OpenSSL.

---

### B4: No headless/CI guard for interactive menu
**File:** `src/main.rs:422-431`

When no subcommand is given, the app calls `ui::run_interactive_menu()`, which uses `dialoguer::Select` and `dialoguer::Input`. Without a TTY (CI, Cloud Run, SSH), this hangs.

**Impact:** The default invocation is unusable in headless environments. There is no `--non-interactive` flag, no `CDG_NONINTERACTIVE` env check, and no stdin TTY detection.

**Fix:** Add `--non-interactive` / `CDG_NONINTERACTIVE` check. If headless and no subcommand, print help and exit with code 1 instead of launching interactive menu.

---

### B5: Hardcoded SQLite prevents Cloud SQL migration
**Files:** `Cargo.toml:11`, `src/cache.rs:37-42`, `.sqlx/`

```toml
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }
```

```rust
let options = SqliteConnectOptions::new()
    .filename(db_path)
    .create_if_missing(true)
    .journal_mode(SqliteJournalMode::Wal)
    .synchronous(SqliteSynchronous::Normal);
```

`.sqlx/` contains 4 query JSON files locked to SQLite.

**Impact:** AGENTS.md explicitly requires GCP/Cloud SQL compatibility. There is no abstraction layer to switch to PostgreSQL or MySQL.

**Fix:** Add `CacheBackend` implementations for Postgres/MySQL. Introduce a `--db-url` flag supporting `postgres://`, `mysql://`, and local SQLite paths. Gate SQLite-specific features behind a feature flag.

---

## 3. Major Issues (P1) — Should Fix Before Beta

### M1: Weak path sanitization
**File:** `src/utils.rs:16-19`

```rust
pub fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|ch| if ch == '^' || ch == '-' { '_' } else { ch })
        .collect()
}
```

Only `^` and `-` are replaced. `validate_safe_path` only checks for `ParentDir` components. A coin ID like `bitcoin/../tmp` becomes `bitcoin_../tmp` after sanitization, then `validate_safe_path` passes it because there is no literal `..` component.

**Impact:** Filesystem write risk. Malicious or malformed coin IDs could write outside intended directories.

**Fix:** Rewrite `sanitize_name` as a whitelist: `[a-zA-Z0-9_-]` only. Reject or strip everything else.

---

### M2: Synchronous plotting blocks async runtime
**File:** `src/plot.rs`

`plotters::BitMapBackend` is synchronous. Called from async `pipeline.rs`, PNG generation blocks the Tokio runtime during multi-coin processing.

**Impact:** Latency spikes when generating candlestick, returns, performance, risk/return, and efficient-frontier plots for multiple coins.

**Fix:** Offload plot generation to `tokio::task::spawn_blocking`.

---

### M3: `chrono::Local` for run directories is non-deterministic
**File:** `src/pipeline.rs:114`, `src/pipeline.rs:1173`

```rust
let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
```

**Impact:** Timezone-dependent paths. Second-level collisions if two GCP runners start simultaneously. Non-reproducible artifact paths.

**Fix:** Use `chrono::Utc::now()` plus a monotonic counter or UUID.

---

### M4: Windows-centric user-agent
**File:** `src/api/coingecko.rs:48`, `src/api/yahoo.rs:18`

```rust
.user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
```

**Impact:** Hardcoded Windows UA in a cross-platform Rust app. Some GCP egress filters or API rate limiters treat Windows UAs differently.

**Fix:** Use a configurable user-agent string, defaulting to something like `cdg/<version> (https://github.com/yourorg/cdg)`.

---

### M5: No GCS/BigQuery/Vertex AI integration
**Finding:** Despite AGENTS.md mandating GCP compatibility, the app only writes local CSV/Parquet. No GCS upload, no BigQuery export, no Vertex AI endpoint.

**Impact:** Users cannot plug CDG into Vertex AI pipelines without manual export/import. The "ML-primary" requirement is unmet for GCP-native workflows.

**Fix:** Add optional `gcs` feature with `google-cloud-storage` SDK. Add BigQuery export via `google-cloud-bigquery`. Document Vertex AI inference via exported Parquet/CSV conventions.

---

## 4. Financial Correctness — Audited & Passing

| Indicator | Status | Evidence |
|-----------|--------|----------|
| RSI bounded `[0, 100]` | PASS | `test_returns_and_indicators_computation` asserts bounds |
| Bollinger upper >= lower | PASS | Same test asserts `u >= l` for all finite pairs |
| MACD histogram = line - signal | PASS | Same test asserts `|hist - (line - signal)| < 1e-9` |
| ADX bounded `[0, 100]` | PASS | `test_adx_golden` asserts bounds from index 26 onward |
| OBV monotonic on flat price | PASS | `test_obv_monotonic_flat_price` asserts constant OBV |
| Sharpe / max drawdown / covariance | PASS | Inspected; standard implementations, no arithmetic bugs found |

No arithmetic bugs found in core finance code.

---

## 5. Headless Compatibility — Partial

| Component | Headless Status |
|-----------|-----------------|
| CLI subcommands (`run-pipeline`, `backtest`, etc.) | PASS |
| `indicatif` progress bars | PASS (`cfg!(test)` hides them) |
| `plotters` PNG output | PASS (file-only, no GUI) |
| Default no-arg invocation | **FAIL** — drops to `dialoguer` interactive menu |
| TTY detection / `--non-interactive` | **MISSING** |

---

## 6. Recommended Action Plan

| Priority | Action | Owner | Est. Effort |
|----------|--------|-------|-------------|
| P0 | Fix treasury NaN metrics (`backtest.rs:1059-1062`) | Backend | Small |
| P0 | Add `--non-interactive` / TTY guard for UI | CLI | Small |
| P0 | Switch `reqwest` to `rustls` or vendored TLS | Infra | Small |
| P0 | Add `.github/workflows/ci.yml` | Infra | Small |
| P0 | Harden `sanitize_name` to whitelist | Security | Small |
| P1 | Abstract `CacheBackend` for Postgres/MySQL | Backend | Medium |
| P1 | Offload `plotters` to `spawn_blocking` | Backend | Small |
| P1 | Replace `chrono::Local` with UTC + monotonic run dirs | Infra | Small |
| P1 | Configurable user-agent | API | Small |
| P2 | Add GCS/BigQuery/Vertex AI adapters | GCP | Medium |
| P2 | Install `cargo audit` / `cargo deny` in CI | Security | Small |

---

## 7. Conclusion

The core math is solid and the test suite is green (113/113). However, the codebase cannot ship as an alpha for a financial ML product because:

1. Reports contain `NaN` values that break downstream ingestion
2. There is no CI/CD gate
3. The TLS stack is incompatible with minimal GCP containers
4. The default invocation hangs in headless environments
5. The cache layer is locked to SQLite, violating the GCP/Cloud SQL requirement

Fix all P0 items, then re-run this review. After P0 clearance, tag `v0.1.0-alpha`.
