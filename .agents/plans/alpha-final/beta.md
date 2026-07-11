# Alpha Final Code Review — Beta Requirements

**Reviewer:** Kilo (automated QA)  
**Date:** 2026-07-10  
**Scope:** Full `src/` audit + test execution + dependency/GCP/headless compatibility check  
**Predecessor:** All P0 items in `alpha.md` must be resolved before beta tagging begins.

---

## 1. Major Issues (P1) — Should Fix Before Beta

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
.user_agent("Mozilla/5.0 (Windows NT 10.0; Win64, x64)")
```

**Impact:** Hardcoded Windows UA in a cross-platform Rust app. Some GCP egress filters or API rate limiters treat Windows UAs differently.

**Fix:** Use a configurable user-agent string, defaulting to something like `cdg/<version> (https://github.com/yourorg/cdg)`.

---

### M5: No GCS/BigQuery/Vertex AI integration
**Finding:** Despite AGENTS.md mandating GCP compatibility, the app only writes local CSV/Parquet. No GCS upload, no BigQuery export, no Vertex AI endpoint.

**Impact:** Users cannot plug CDG into Vertex AI pipelines without manual export/import. The "ML-primary" requirement is unmet for GCP-native workflows.

**Fix:** Add optional `gcs` feature with `google-cloud-storage` SDK. Add BigQuery export via `google-cloud-bigquery`. Document Vertex AI inference via exported Parquet/CSV conventions.

---

## 2. Recommended Action Plan (P1)

| Priority | Action | Owner | Est. Effort |
|----------|--------|-------|-------------|
| P1 | Abstract `CacheBackend` for Postgres/MySQL | Backend | Medium |
| P1 | Offload `plotters` to `spawn_blocking` | Backend | Small |
| P1 | Replace `chrono::Local` with UTC + monotonic run dirs | Infra | Small |
| P1 | Configurable user-agent | API | Small |

---

## 3. Beta Conclusion

Fix all P1 items before shipping the beta release. P0 blockers must already be resolved per `alpha.md`. After P1 clearance, tag `v0.1.0-beta`.
