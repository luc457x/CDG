# Module Review: `pipeline.rs` + `main.rs`

**Files:** `src/pipeline.rs`, `src/main.rs`  
**Tests passing:** integration tests green

## What is implemented well
- CancellationToken (07): Propagated to all JoinSet tasks; clean `main` return.
- E2E integration tests (08): `wiremock` happy path, 404, missing ^TNX, cache hits.
- Report extraction (09): `generate_backtest_report` + `append_treasury_benchmark` centralized.
- Annualization (10): `--drop-weekends` threaded to standalone backtest; `252` vs `365` logic correct.
- Toggles (14/15): `plots`/`optimize` flags + env vars + gates.
- Light conflicts (16): `warn_light_conflicts` + test.
- Currency guard (25): Empty `currency_cols` returns `Err`.
- Menu (29): Concurrency + annualization_factor passed from interactive menu.

## Remaining gaps / risks

| # | Severity | Gap | Evidence |
|---|----------|-----|----------|
| C1 | P1 | Final success message unconditional | `pipeline.rs:865` — `"CDG data pipeline completed successfully!"` regardless of skipped plots/optimization/backtest |
| C2 | P2 | `run_pipeline_flow` not reduced to <400 LOC | Currently ~808 LOC; plan 09 target not met |
| C3 | P2 | Standalone backtest lacks `annualization_factor: Option<f64>` param | `pipeline.rs:1013` — only `drop_weekends: bool`; custom factor unavailable in standalone mode |
| C4 | P3 | `run_pipeline_flow` has 12-arg signature | `main.rs:265` — clippy suppressed; `PipelineConfig` exists but not used at all call sites |

## Recommendations
1. Make final status message reflect actual outcomes (C1).
2. Continue extracting orchestration helpers to reach <400 LOC target (C2).
3. Add `annualization_factor` override to standalone if users need it (C3).
