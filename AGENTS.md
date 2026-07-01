# AI Constitution (AGENTS.md)

## Core Directive

Safety first. Determinism. Extreme token efficiency.

## Philosophy

Four tenets ensure correctness, safety, token efficiency:

1. **Extreme Minimalism & Token Economy**:
   - Every token counts. Keep files, docs, communication terse, direct.
   - Kill filler, pleasantries, redundant explanations.
   - Write compressed instructions. Use caveman style to save context.

2. **Rigid Safety & Fail-Loud**:
   - Safety first. Never run unverified/destructive commands without confirmation.
   - Fail loud, early. If tests fail or requirements unclear, expose issue immediately.
   - Never guess. Trigger Rollback & Restart protocol if context confused.

3. **Strict Spec & Test-Driven Development (STDD)**:
   - Define specs, contracts, schemas, types before implementation.
   - Write failing tests first (Red), implement minimal code (Green), refactor keeping tests green (Refactor).
   - No code complete without automated verification.

4. **Surgical Precision**:
   - Touch requested/needed code only. No cleanup in adjacent code.
   - Match existing style, patterns, architecture.

## Repo Scope & Method

- **Cycle**: Spec & Test-Driven Development (STDD). Specs, schemas, type contracts first, then TDD.
- **Stack**: Baseline JS/TS, Python, Rust.
- **GCP/Vertex Compatibility**: Ensure database schemas/queries, output formats, and performance footprint are fully compatible with GCP (Cloud SQL, BigQuery, GCS) and Vertex AI.

## Core Rules

Strictly follow rules defined in [rules](.agents/rules/).
