---
name: request-refactor-plan
description: Create detailed refactor plan with tiny commits via user interview, then file as GitHub issue
when_to_use: user wants to plan refactor, create refactoring RFC, or break refactor into safe incremental steps.
metadata:
  category: workflow
---
# Request Refactor Plan

## When to Use

Invoked when user wants to create refactor request. Go through steps below. May skip steps if not necessary.

## Steps

1. Ask user for long, detailed description of problem and any potential ideas for solutions.

2. Explore repo to verify assertions and understand current state of codebase.

3. Ask whether other options considered, and present alternatives.

4. Interview user about implementation. Be extremely detailed and thorough.

5. Hammer out exact scope. Work out what to change and what not to change.

6. Check codebase for test coverage in this area. If insufficient, ask user about testing plans.

7. Break implementation into plan of tiny commits. Remember Martin Fowler: "make each refactoring step as small as possible, so you can always see program working."

8. Create GitHub issue with refactor plan using this template:

```md
## Problem Statement

Problem developer facing, from developer's perspective.

## Solution

Solution to problem, from developer's perspective.

## Commits

Long, detailed implementation plan. Write in plain English, breaking into tiniest commits possible. Each commit must leave codebase in working state.

## Decision Document

List of implementation decisions made. Can include:

- Modules built/modified
- Interfaces of those modules modified
- Technical clarifications from developer
- Architectural decisions
- Schema changes
- API contracts
- Specific interactions

Do NOT include specific file paths or code snippets. May go stale quickly.

## Testing Decisions

List of testing decisions made. Include:

- What makes a good test (only test external behavior, not implementation details)
- Which modules will be tested
- Prior art for tests (similar types of tests in codebase)

## Out of Scope

Things out of scope for this refactor.

## Further Notes (optional)

Any further notes about refactor.
```
