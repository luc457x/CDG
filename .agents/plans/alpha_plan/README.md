# Alpha Plan — Execution Phases

**Created:** 2026-07-08  
**Scope:** v0.1.0-alpha release gate + high-value pre-alpha work  
**Rule:** One deliverable per file. No batch commits.  
**Sources:** `alpha_readiness_2026-07-05.md`, `alpha_readiness_plan_2026-07-07.md`, `base_plan.md`, `line_by_line_review.md`, `code_analysis_2026-07-04.md`, `code_review.md`

---

## Execution Order

```
01 → 02 → 03          (analysis data-quality + tests — P0 gate)
04 → 05 → 06          (backtest correctness + tests — P0 gate)
07                   (Ctrl+C graceful shutdown — P0 gate)
08                   (E2E integration test — P0 gate)
09 → 10 → 11 → 12    (P1: reliability, API, correctness)
13 → 14 → 15         (P1: UX toggles)
16 → 17 → 18         (tests that lock behavior)
19 → 20 → 21 → 22 → 23 → 24 → 25  (quick wins / polish)
26 → 27 → 28 → 29  (code_review.md gaps: covariance/zero-weights, path-injection, 3s-cache-miss-delay, menu concurrency/annualization)
```

**Alpha gate:** 01–08 must all be merged, passing, and tagged `v0.1.0-alpha`.  
**Pre-alpha polish:** 09–29 should be merged immediately after or before the tag if time permits. Note: 26 is Critical.
**Backlog:** See `backlog.md`.

## How to Use This Folder

Read the phase file, implement exactly what it describes, write the red test first (STDD), implement the green fix, refactor if needed, run validation commands, then commit as the phase name. Do not batch unrelated changes into one phase.
