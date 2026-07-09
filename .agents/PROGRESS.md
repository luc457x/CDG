# Progress (PROGRESS.md)

## Status

- State: Phases 01–29 implemented; ADR + SPEC alignment rules refactored into `adr` and `spec-align` skills as single source of truth.
- Last: Created `adr` skill (template + decision-gate) and `spec-align` skill; removed duplicated ADR gate from `grill-me`; deleted loose `000_TEMPLATE.md`.

## Log

Old sessions: [PROGRESS_ARCHIVE.md](./PROGRESS_ARCHIVE.md).

### Session 47: Refactor ADR & Spec Rules into Skills

- Date: 2026-07-09
- Agent: Antigravity
- Goal: Move ADR decision-gate/template and SPEC.md alignment rules into dedicated skills as single source of truth.
- Constraints: None.
- Done:
  - Created `.agents/skills/adr/SKILL.md` — embedded ADR template + decision-gate (hard-to-reverse, surprising, real trade-off); canonical source for when to write ADR in `.agents/docs/ADRs/`.
  - Deleted `.agents/docs/ADRs/000_TEMPLATE.md` — template content moved into `adr` skill.
  - Updated `.agents/skills/grill-me/SKILL.md` — replaced duplicated ADR gate (old lines 54-62) with pointer to `adr` skill; removed drift risk.
  - Created `.agents/skills/spec-align/SKILL.md` — planning/SDD gate: aligned tasks → implement via `stdd`; unaligned → `spec-triage` (backlog/spec). References existing `spec-triage`, no routing logic duplicated.
- Blocked: None.
- Risk: None — skill/docs only; `spec-triage` remains routing source of truth.
- Artifact: `.agents/skills/adr/SKILL.md`, `.agents/skills/spec-align/SKILL.md`, `.agents/skills/grill-me/SKILL.md`.
- Verification: `python .agents/skills/audit-skills/scripts/audit_skills.py --skill adr` and `--skill spec-align` — both pass clean (0 structural issues, 0 broken links).
- Pending: None.

### Session 46: Fix Stale Path References

- Date: 2026-07-09
- Agent: Antigravity
- Goal: Fix stale references after `.agents/` folder restructure (`adr/` → `docs/ADRs/`, `etc/` → `plans/`, `personas/` → `agents/`).
- Constraints: None.
- Done:
  - Updated `.agents/docs/ADRs/0000_TEMPLATE.md` — changed `adr/` → `docs/ADRs/`
  - Updated `.agents/skills/grill-me/SKILL.md` — changed 2 `adr/` → `docs/ADRs/` refs
  - Updated `.agents/rules/engineering.md` — changed 1 `adr/` → `docs/ADRs/` ref
  - Updated `.agents/rules/structure.md` — changed folder listing (`adr/` → `docs/ADRs/`, `etc/` → `plans/`, `personas/` → `agents/`)
  - Updated `.agents/plans/alpha_plan/post_review/REMAINING_GAPS.md` — changed `etc/post_review/` → `plans/alpha_plan/post_review/`
  - Updated `.agents/PROGRESS_ARCHIVE.md` — changed 4 old-path refs to new paths
- Blocked: None.
- Risk: None — doc-only changes.
- Artifact: `.agents/docs/ADRs/0000_TEMPLATE.md`, `.agents/skills/grill-me/SKILL.md`, `.agents/rules/engineering.md`, `.agents/rules/structure.md`, `.agents/plans/alpha_plan/post_review/REMAINING_GAPS.md`, `.agents/PROGRESS_ARCHIVE.md`.
- Verification: `Select-String` across repo — no remaining `.agents/adr/`, `.agents/etc/alpha_plan/`, `.agents/etc/post_review/` refs; 3 intentional `.agents/etc/cdg-lib-migration-candidates.md` refs remain in archive (file deleted, no replacement).
- Pending: None.

### Session 45: Complete Alpha Phases 26-29

- Date: 2026-07-09
- Agent: Antigravity
- Goal: Implement alpha phases 26 to 29.
- Constraints: None.
- Done:
  - **Phase 26**: [src/optimization.rs](file:///C:/Users/lucas/Code/CDG/src/optimization.rs) — (1) `r_f_annual` silent fallback replaced: now emits `eprintln!` warning and uses documented constant `0.04` when `^TNX` absent/empty. (2) Added `MAX_SIMULATIONS_DEFAULT` (10k) and `MAX_SIMULATIONS_HARD_CAP` (50k) consts; `num_simulations` clamped at top of `run_monte_carlo`. (3) `+ 0.01` min-weight hack documented with Dirichlet-bias rationale. (4) Progress bar update moved post-parallel-collect with explanatory comment. (5) Added `test_covariance_no_zero_weights` (both weights > 0 with 10-row dataset) and `test_sharpe_formula_and_weights_sum` (weights sum to 1.0; hard cap respected).
  - **Phase 27**: [tests/pipeline_tests.rs](file:///C:/Users/lucas/Code/CDG/tests/pipeline_tests.rs) — Added `test_path_injection_blocked_by_sanitize_and_validate`: verifies `"../../etc/passwd"` coin id → `validate_safe_path` returns `Err("Path traversal detected")`. Runtime guards (`sanitize_name` + `validate_safe_path`) were already wired in [src/pipeline.rs](file:///C:/Users/lucas/Code/CDG/src/pipeline.rs) and [src/api/coingecko.rs](file:///C:/Users/lucas/Code/CDG/src/api/coingecko.rs) in prior phases.
  - **Phase 28**: Already done. No unconditional `sleep` found in [src/api/coingecko.rs](file:///C:/Users/lucas/Code/CDG/src/api/coingecko.rs) — only retry-path backoff sleeps.
  - **Phase 29**: Already done. [src/ui.rs](file:///C:/Users/lucas/Code/CDG/src/ui.rs) already prompts user for concurrency and annualization factor and passes both through `PipelineConfig`.
- Blocked: None.
- Risk: None.
- Artifact: [src/optimization.rs](file:///C:/Users/lucas/Code/CDG/src/optimization.rs), [tests/pipeline_tests.rs](file:///C:/Users/lucas/Code/CDG/tests/pipeline_tests.rs).
- Verification: `cargo test` — 111 tests passed.
- Pending: None.
