---
trigger: always_on
---

# Engineering Rules

1. **Spec-Driven**: Task must be in `SPEC.md` before build. If missing, move to `BACKLOG.md`. Do not build.
2. **Surgical**: Touch only what needed. No style/comment cleanups in adjacent code. Match style.
3. **Minimal**: Only requested code. No extra features/optimizations.
4. **Pragmatic**: Simplest solution. Shortest path. No over-engineering.
5. **Read First**: Review exports, callers, utilities before editing. Ambiguity → write ADR in `.agents/adr/`. Blocked → follow [workflow rules](../rules/workflow.md#unresolvable-blockers).
6. **No Blend**: Choose newest/most tested pattern. Explain choice. Flag old for cleanup.
7. **Relative Paths**: Always use relative paths (e.g., `../.agents/file.md`) instead of absolute paths.