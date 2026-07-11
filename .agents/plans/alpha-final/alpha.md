# Alpha Final Code Review — Alpha Release Blockers

**Reviewer:** Kilo (automated QA)  
**Date:** 2026-07-10  
**Scope:** Full `src/` audit + test execution + dependency/GCP/headless compatibility check  
**Verdict:** **NOT ALPHA-READY** — 5 critical blockers remain unfixed.

---

## 1. Critical Blockers (P0) — Must Fix Before Alpha Tag

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

## 2. Recommended Action Plan (P0)

| Priority | Action | Owner | Est. Effort |
|----------|--------|-------|-------------|
| P0 | Fix treasury NaN metrics (`backtest.rs:1059-1062`) | Backend | Small |
| P0 | Add `--non-interactive` / TTY guard for UI | CLI | Small |
| P0 | Switch `reqwest` to `rustls` or vendored TLS | Infra | Small |
| P0 | Add `.github/workflows/ci.yml` | Infra | Small |
| P0 | Harden `sanitize_name` to whitelist | Security | Small |

---

## 3. Alpha Conclusion

The core math is solid and the test suite is green (113/113). However, the codebase cannot ship as an alpha for a financial ML product because:

1. Reports contain `NaN` values that break downstream ingestion
2. There is no CI/CD gate
3. The TLS stack is incompatible with minimal GCP containers
4. The default invocation hangs in headless environments
5. The cache layer is locked to SQLite, violating the GCP/Cloud SQL requirement

Fix all P0 items, then re-run this review. After P0 clearance, tag `v0.1.0-alpha`.
