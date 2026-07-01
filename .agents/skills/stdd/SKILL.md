---
name: stdd
description: Spec and Test-Driven Development, defining contracts/schemas/types first, then validating via red-green-refactor.
when_to_use: building new features, modules, or data layers.
metadata:
  category: development
---
# Spec & Test-Driven Development (STDD)

## Philosophy

**Core**: Define what system _is_ before proving it _works_. Specifications (types, schemas, interfaces) come first. Tests validate spec. Implementation satisfies tests.

## Vocabulary

**Module**: Interface + implementation. **Interface**: Everything caller must know (types, invariants, errors, ordering). **Depth**: Small interface, lots of implementation behind it. **Seam**: Where interface lives; behaviour altered without editing there. **Adapter**: Concrete thing satisfying interface. **Leverage**: More behaviour per unit of interface learned. **Locality**: Change/bugs concentrated in one place.

## Anti-Patterns

### Spec Without Teeth
- **DO NOT** treat specs as documentation-only. Spec not enforced = wish, not contract.
- Bad: doc describing API shape, then implement without reference.
- Good: define TypeScript interfaces / Python dataclasses / Rust structs, write tests using those types.

### Over-Specification
- **DO NOT** spec implementation details (_how_), only _what_.
- WRONG: "OrderService uses priority queue internally to sort by date"
- RIGHT: "OrderService.list() returns orders sorted by date descending"

### Horizontal Slices
- **DO NOT** write all tests first, then all code. Produces bad tests.
- RIGHT: Vertical slices via tracer bullets. One test → one implementation → repeat.

## Workflow

### Phase 1: SDD — Define Contracts
- Define data models / schemas / type contracts
- Define public interfaces (inputs, outputs, errors)
- Define validation rules (from SPEC.md business rules)
- Get user approval on contracts

**Output**: Compilable type definitions and interface contracts. Not docs — code.

### Phase 2: TDD — Planning
- Confirm with user what interface changes needed
- Confirm which behaviors to test (prioritize)
- Design interfaces for testability
- List behaviors to test (not implementation steps)
- Get user approval on plan

Ask: "What should public interface look like? Which behaviors matter most?"

### Phase 3: TDD — Tracer Bullet
Write ONE test that confirms ONE thing:
```text
RED:   Write test for first behavior → test fails
GREEN: Write minimal code to pass → test passes
```

### Phase 4: TDD — Incremental Loop
For each remaining behavior:
```text
RED:   Write next test → fails
GREEN: Minimal code to pass → passes
```

Rules: One test at a time. Only enough code to pass. Keep tests focused on observable behavior.

### Phase 5: TDD — Refactor
After all tests pass:
- Extract duplication
- Deepen modules (more behaviour behind smaller interface)
- Apply SOLID where natural
- Run tests after each refactor

**Never refactor while RED.**

### Contract Evolution
- If test reveals contract wrong → **update contract first**, then test, then implementation
- Never silently deviate from contract
- Contract changes documented (commit message or PROGRESS.md)

## Checklist

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
