---
name: stdd
description: >
  Spec & Test-Driven Development. Define contracts/schemas/types first (SDD phase),
  then validate via red-green-refactor (TDD phase). Use when building new features,
  modules, or data layers. Covers full cycle: spec → test → implement → refactor.
---

# Spec & Test-Driven Development (STDD)

## Philosophy

**Core principle**: Define what system _is_ before proving it _works_. Specifications (types, schemas, interfaces) come first. Tests validate spec. Implementation satisfies tests.

STDD = **SDD phase** (define contracts) → **TDD phase** (red-green-refactor against those contracts).

Why spec-first:

- Types/schemas catch structural errors at compile time — cheaper than test failures
- Contracts serve as living documentation for humans and AI agents
- TDD without spec tends to drift — test what _built_, not what _needed_
- Spec-first forces boundary thinking before touching implementation

**Good tests** are integration-style: exercise real code paths through public APIs. Describe _what_ system does, not _how_. Survive refactors — don't care about internal structure.

**Bad tests** are coupled to implementation. Mock internal collaborators, test private methods, or verify through external means. Warning sign: test breaks when refactoring, but behavior hasn't changed.

See [tests.md](./tests.md) for examples and [mocking.md](./mocking.md) for mocking guidelines.

## Anti-Patterns

### Spec Without Teeth

**DO NOT treat specs as documentation-only.** Spec not enforced by types or validated by tests = wish, not contract.

Bad: write doc describing API shape, then implement without referencing it.
Good: define TypeScript interfaces / Python dataclasses / Rust structs, write tests that use those types, then implement.

### Over-Specification

**DO NOT spec implementation details.** Spec _what_ (inputs, outputs, constraints), not _how_ (algorithms, internal data flow).

```text
WRONG: "OrderService uses a priority queue internally to sort by date"
RIGHT: "OrderService.list() returns orders sorted by date descending"
```

### Horizontal Slices

**DO NOT write all tests first, then all implementation.** "Horizontal slicing" — treating RED as "write all tests" and GREEN as "write all code."

Produces bad tests:

- Tests written in bulk test _imagined_ behavior, not _actual_ behavior
- Test _shape_ of things (data structures, signatures) rather than user-facing behavior
- Tests become insensitive to real changes
- Outrun headlights, commit to test structure before understanding implementation

**Correct approach**: Vertical slices via tracer bullets. One test → one implementation → repeat.

```text
WRONG (horizontal):
  RED:   test1, test2, test3, test4, test5
  GREEN: impl1, impl2, impl3, impl4, impl5

RIGHT (vertical):
  RED→GREEN: test1→impl1
  RED→GREEN: test2→impl2
  RED→GREEN: test3→impl3
  ...
```

## Workflow

### Phase 1: SDD — Define Contracts

Before any test or implementation code:

- Define data models / schemas / type contracts
- Define public interfaces (function signatures, API endpoints, DTOs)
- Define validation rules and constraints (from SPEC.md business rules)
- Get user approval on contracts

See [spec_phase.md](./spec_phase.md) for guidelines and examples.

**Output**: compilable/parseable type definitions and interface contracts. Not docs — code.

### Phase 2: TDD — Planning

When exploring codebase, use project's domain glossary so test names and interface vocabulary match project's language, and respect ADRs in area being touched.

Before writing any tests:

- Confirm with user what interface changes needed
- Confirm which behaviors to test (prioritize)
- Identify opportunities for [deep modules](./deep_modules.md) (small interface, deep implementation)
- Design interfaces for [testability](../design_an_interface/SKILL.md)
- List behaviors to test (not implementation steps)
- Get user approval on plan

Ask: "What should public interface look like? Which behaviors most important to test?"

**Can't test everything.** Confirm with user exactly which behaviors matter most. Focus on critical paths and complex logic, not every edge case.

### Phase 3: TDD — Tracer Bullet

Write ONE test that confirms ONE thing about system:

```text
RED:   Write test for first behavior → test fails
GREEN: Write minimal code to pass → test passes
```

Tracer bullet — proves path works end-to-end.

### Phase 4: TDD — Incremental Loop

For each remaining behavior:

```text
RED:   Write next test → fails
GREEN: Minimal code to pass → passes
```

Rules:

- One test at a time
- Only enough code to pass current test
- Don't anticipate future tests
- Keep tests focused on observable behavior

### Phase 5: TDD — Refactor

After all tests pass, look for refactor candidates:

- Extract duplication
- Deepen modules (move complexity behind simple interfaces)
- Apply SOLID principles where natural
- Consider what new code reveals about existing code
- Run tests after each refactor step

**Never refactor while RED.** Get to GREEN first.

### Contract Evolution

Contracts may need to change during TDD. Rules:

- If test reveals contract wrong → **update contract first**, then test, then implementation
- Never silently deviate from contract in implementation
- Contract changes must be intentional and documented (commit message or PROGRESS.md)

## Checklist Per Feature

```md
SDD Phase:
[ ] Types/schemas defined and compilable
[ ] Public interfaces specified (inputs, outputs, errors)
[ ] Constraints from SPEC.md captured in types or validation
[ ] User approved contracts

TDD Phase:
[ ] Tests reference defined contracts/types
[ ] Vertical slices (not horizontal)
[ ] All tests pass
[ ] Refactor complete under green

Integration:
[ ] Contract changes documented if any occurred
[ ] No spec drift (implementation matches contract)
```

## Checklist Per TDD Cycle

```md
[ ] Test describes behavior, not implementation
[ ] Test uses public interface only
[ ] Test would survive internal refactor
[ ] Code is minimal for this test
[ ] No speculative features added
```
