---
name: grill-me
description: Interview user relentlessly about plan or design until reaching shared understanding. Challenges against SPEC.md and sharpens terminology
when_to_use: user wants to stress-test plan, get grilled on design, or mentions "grill me".
metadata:
  category: workflow
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

- `.agents/SPEC.md` — specifications, business rules, tech stack
- `.agents/BACKLOG.md` — ideas and unvalidated features
- `.agents/docs/ADRs/` — architecture decision records (optional)

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

Offer to create ADR in `.agents/docs/ADRs/` only when decision gate met. Gate rules live in `adr` skill — load it to decide and author. Do NOT duplicate gate here. If gate not met, skip ADR.