---
name: adr
description: Decide when architecture decision record needed, then author it in .agents/docs/ADRs/ using bundled template.
when_to_use: proposing hard-to-reverse, surprising, or trade-off architecture decisions, or user asks to write/record ADR.
metadata:
  category: documentation
---
# Architecture Decision Records (ADR)

Author ADRs in `.agents/docs/ADRs/` with sequential numbering `001-slug.md`, `002-slug.md`. Next number = highest existing + 1.

## Philosophy

Code explains _what_. ADR explains _why_ decision made, so future reader not wonder "why done this way?". Write only when decision carries weight. Do not write ADR for obvious or reversible choices.

## When To Write (gate)

Write ADR only if decision meets ALL of:

1. **Hard to reverse**: cost of changing later high (tech stack lock-in, data model, integration pattern).
2. **Surprising**: code reader would question choice versus obvious path.
3. **Real trade-off**: valid alternatives existed with distinct trade-offs.

Qualifies:
- Core architecture / integration patterns.
- Stacks with significant lock-in.
- Deliberate deviation from standard pattern.
- Constraint invisible in code (compliance, partner SLA).

Skip if reversible, obvious, or no alternative considered. Log minor decision in PROGRESS.md instead.

## Workflow

1. **Detect trigger**: during plan or build, spot decision meeting gate above.
2. **Name + number**: pick `NNNN-kebab-slug.md`. Check existing files for next seq.
3. **Fill template**: copy `Template` below, fill all fields.
4. **Status**: set `proposed` first. Promote `accepted` after user approval. Mark `deprecated`/`superseded by ADR-NNNN` when replaced.
5. **Date**: `YYYY-MM-DD` of authoring.
6. **Place file**: write to `.agents/docs/ADRs/`.

## Template

```markdown
# {Title}

**Status:** {proposed | accepted | deprecated | superseded by ADR-NNNN}
**Date:** {YYYY-MM-DD}

### Context & Problem Statement
{Briefly describe context and problem being solved.}

### Decision
Decided to **{decision}** because:
1. **{Reason 1}**
2. **{Reason 2}**

### Consequences (Optional)
- **Good:** {Positive impacts}
- **Bad:** {Negative impacts or trade-offs}
```

## Checklist

```md
[ ] Decision meets all 3 gate rules (hard-to-reverse, surprising, trade-off)
[ ] File named NNNN-kebab-slug.md with correct next sequence number
[ ] Status set (proposed → accepted after approval)
[ ] Date in YYYY-MM-DD
[ ] Context, Decision, Consequences filled
[ ] Written to .agents/docs/ADRs/
```
