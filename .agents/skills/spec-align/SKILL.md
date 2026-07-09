---
name: spec-align
description: Gate planning and work against SPEC.md; aligned tasks implement via STDD, unaligned ideas route to backlog or spec.
when_to_use: planning or working on tasks, verifying work matches SPEC.md, or before building feature.
metadata:
  category: workflow
---
# Spec Alignment Gate

Before build, confirm task matches `SPEC.md`. SDD first: spec defines what system is; tests prove it works.

## Philosophy

`SPEC.md` holds goals, scope, FR/BR/NFR. Code must serve spec. Work outside spec risks scope creep or unvalidated ideas. Gate catches misalignment early.

## Workflow

1. **Read SPEC.md**: scan Goals, Scope, FR/BR/NFR for task match.
2. **Align check**: does task map to existing FR/BR/NFR or in-scope goal?
3. **Branch**:
   - **Aligned** → implement via `stdd` skill now (define contracts, TDD). No extra doc.
   - **Not aligned** → not specced. Route via `spec-triage` skill. Unvalidated idea → `BACKLOG.md`. Small additive clarification → `SPEC.md`.
4. **Never implement** from backlog. Backlog holds ideas only.

## Alignment signals

Aligned if any true:
- Maps to FRnn / BRnn / NFRnn ID.
- Extends in-scope goal (Goal 1–6).
- Additive flag/field/log within existing FR.

Not aligned if any true:
- New behaviour absent from `SPEC.md`.
- Changes scope boundary (in-scope ↔ out-of-scope).
- Big/risky/hard-to-revert change.

## Checklist

```md
[ ] SPEC.md read and scanned
[ ] Task mapped to FR/BR/NFR or goal
[ ] Aligned → stdd started
[ ] Not aligned → spec-triage run; backlog or spec updated
[ ] No direct implementation from backlog
```
