---
name: grill_me
description: Interview user relentlessly about plan or design until reaching shared understanding. Challenges against SPEC.md and sharpens terminology. Use when user wants to stress-test plan, get grilled on design, or mentions "grill me".
---

# Grill Me

## When to Use

Interview relentlessly about every aspect of plan until shared understanding reached. Walk down each branch of design tree, resolving dependencies between decisions one-by-one. For each question, provide recommended answer.

Ask questions one at a time, waiting for feedback before continuing.

If question can be answered by exploring codebase, explore codebase instead.

## Steps

1. **Codebase Check**: Before asking any question, search codebase using native tools (`grep_search`, `list_dir`, `view_file`) to check if answer already documented or implemented.
2. **Sequential Questioning**: Ask questions **one at a time**. Do not dump multiple questions in single response.
3. **Provide Recommendations**: For every question, present clear recommended choice/answer based on best practices and repository conventions.
4. **Tree Resolution**: Walk down each branch of design tree. Resolve upstream decisions before asking about downstream implementations.
5. **Stop Condition**: End interview only when complete, shared understanding achieved and all decision paths resolved.

## Domain Awareness

During codebase exploration, review project docs in `.agents/`:

- `SPEC.md` — specifications, business rules, tech stack
- `BACKLOG.md` — ideas and unvalidated features
- `TASKS.md` — actionable tasks roadmap
- `.agents/adr/` — (optional) architecture decision records

### Challenge Against SPEC.md

Compare proposed plan against FRs, BRs, and scope defined in `.agents/SPEC.md`. If plan contradicts spec, call it out immediately: "Your plan specifies X, but SPEC.md Business Rule Y says Z — which is correct?"

### Sharpen Fuzzy Language

When user uses vague or overloaded terms, propose precise canonical terms to add to domain spec or business rules.

### Discuss Concrete Scenarios

When domain relationships or business rules discussed, stress-test with specific scenarios. Invent scenarios that probe edge cases and force precision.

### Cross-Reference with Code

When user states how something works, check whether code agrees. If contradiction found, surface it.

### Update SPEC.md Inline

When business rule, requirement, or scope boundary resolved during interview, update `.agents/SPEC.md` right there. Don't batch — capture as they happen.

## ADRs

Only offer to create ADR in `.agents/adr/` when all three are true:

1. **Hard to reverse** — cost of changing mind later is meaningful
2. **Surprising without context** — future reader will wonder "why did they do it this way?"
3. **Result of real trade-off** — genuine alternatives existed and one was picked for specific reasons

If any of the three missing, skip ADR.