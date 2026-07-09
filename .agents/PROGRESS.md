# Progress (PROGRESS.md)

## Status

- State: Phases 01–29 implemented; `.agents/` restructure + ADR/SPEC skill refactor complete; leftover restructure edits committed.
- Last: Committed leftover session-46 edits (AGENTS.md Stack, structure.md plans, ADR 0001/0002 → 001/002 renames) as Session 48.

## Log

Old sessions: [PROGRESS_ARCHIVE.md](./PROGRESS_ARCHIVE.md).

### Session 49: Implement P1 Gaps from Post-Review

- Date: 2026-07-09
- Agent: Antigravity
- Goal: Fix all P1 gaps + G12 identified in `.agents/plans/alpha_plan/post_review/`.
- Done:
  - **G1** (`analysis.rs`): Changed `compute_returns_and_indicators` return to `Result<(DataFrame, Vec<String>)>`; null warnings (high/low/volume) now collected in `warnings` vec instead of `eprintln!`. Updated 3 `pipeline.rs` call sites, 4 `analysis.rs` test call sites, 1 `pipeline_tests.rs` call site.
  - **G2** (`analysis.rs`): Added `test_macd_golden` (histogram=line−signal invariant, post-warmup all-Some for constant prices) and `test_adx_golden` (None during warm-up 0..25, [0,100] from index 26+).
  - **G3** (`analysis.rs`): Upgraded `test_returns_and_indicators_computation` with 60-bar dataset + RSI∈[0,100] assertion, Bollinger upper≥lower, MACD hist=line−signal across all non-None triples.
  - **G4** (`backtest.rs`): Portfolio and treasury `BacktestMetrics` prediction placeholders changed from misleading `0.0` to `f64::NAN`; `prediction_rating` stays `"n/a"`.
  - **G5** (`pipeline.rs`): Final success message now conditional: prints skipped phases (plots/optimization/backtesting) in status line.
  - **G6** (`pipeline.rs`, `main.rs`): `run_standalone_backtest` gains `annualization_factor: Option<f64>` param; `Backtest` CLI subcommand wired with `--annualization-factor` / `ANNUALIZATION_FACTOR` env.
  - **G12** (`utils.rs`): `validate_safe_path` returns `Err` on empty string; test updated.
- Blocked: None.
- Risk: None — no logic change to indicators or backtest math.
- Artifact: `src/analysis.rs`, `src/backtest.rs`, `src/pipeline.rs`, `src/main.rs`, `src/utils.rs`, `tests/pipeline_tests.rs`.
- Verification: `cargo test` — 113 pass, 0 failures (5 suites).
- Pending: P2 gaps (G7–G13) and P3 (G14–G17) remain in REMAINING_GAPS.md.

### Session 48: Commit Leftover Restructure Edits

- Date: 2026-07-09
- Agent: Antigravity
- Goal: Commit leftover folder-restructure edits from session 46 (AGENTS.md Stack, structure.md plans, ADR numbering 0001/0002 → 001/002).
- Constraints: None.
- Done:
  - Updated `AGENTS.md` — Stack line: "Baseline JS/TS, Python, Rust" → "Rust, SQLite".
  - Updated `.agents/rules/structure.md` — plans description trimmed to "Execution plans".
  - Renamed ADRs: `0001-gcp-compatibility.md` → `001-gcp-compatibility.md`, `0002-native-polars-indicators.md` → `002-native-polars-indicators.md` (numbering normalized to 3 digits; line-ending normalized).
- Blocked: None.
- Risk: None — docs only; ADR contents unchanged.
- Artifact: `AGENTS.md`, `.agents/rules/structure.md`, `.agents/docs/ADRs/001-gcp-compatibility.md`, `.agents/docs/ADRs/002-native-polars-indicators.md`.
- Verification: `git show` index blobs vs new files — ADR content identical (CRLF/LF only); `git status` clean post-commit.
- Pending: None.

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


