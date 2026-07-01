# Architecture Decision Records (ADRs)

ADRs live in `.agents/adr/` with sequential numbering: `0001-slug.md`, `0002-slug.md`, etc.

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

## When to Write

Write ADR only when decision is:
1. **Hard to reverse:** Cost of changing decision later is high.
2. **Surprising:** Code readers would wonder "why was it done this way?"
3. **Real trade-off:** Valid alternatives existed with distinct trade-offs.

## What Qualifies

- Core architectural and integration patterns.
- Technology stacks with significant lock-in.
- Deliberate deviations from obvious paths or standard patterns.
- Constraints not visible in code (compliance, partner SLAs).
