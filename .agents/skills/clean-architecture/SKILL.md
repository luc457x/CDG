---
name: clean-architecture
description: Enforces Clean Code Architecture rules when writing or reviewing code. Applies layer separation, dependency inversion, SOLID, naming, and structure conventions.
when_to_use: writing new modules, reviewing code, user mentions clean architecture, SOLID, layers, use cases, entities, or asks for well-structured code.
metadata:
  category: development
---
# Clean Architecture

## Rules

Apply when code written or reviewed.

## Layers (inner → outer, dependencies point inward only)

```
Entities → Use Cases → Interface Adapters → Frameworks/Drivers
```

| Layer | Lives here | Must NOT depend on |
|---|---|---|
| **Entities** | Business objects, domain rules | Anything outer |
| **Use Cases** | App logic, orchestration | Adapters, frameworks |
| **Adapters** | Controllers, presenters, gateways | Frameworks (DB, HTTP lib) |
| **Frameworks** | DB, HTTP, UI, CLI | Nothing inner |

## Dependency Rule

> Source code dependency MUST point inward. Outer knows inner. Inner knows nothing outer.

- Inner layers define interfaces. Outer layers implement.
- Never import framework type into entity or use case.
- Use dependency injection at composition root.

## SOLID — One rule per decision

| | Rule | Apply when |
|---|---|---|
| **S** | Single Responsibility | Class/fn does >1 thing → split |
| **O** | Open/Closed | Adding behavior → extend, not modify |
| **L** | Liskov Substitution | Subtype breaks parent contract → redesign |
| **I** | Interface Segregation | Client forced to depend unused method → split interface |
| **D** | Dependency Inversion | High-level depends concrete → inject abstraction |

## Naming

- **Entities**: noun, domain language (`Order`, `Invoice`, `User`)
- **Use Cases**: verb + noun (`CreateOrder`, `CancelInvoice`, `GetUser`)
- **Interfaces/Ports**: role-based (`OrderRepository`, `EmailSender`)
- **Implementations**: suffix source (`PostgresOrderRepository`, `SendgridEmailSender`)
- **DTOs**: suffix direction (`OrderRequest`, `OrderResponse`)
- No `Manager`, `Helper`, `Utils`, `Handler` — name what it does

## Structure (adapt to stack)

```
src/
├── domain/           # Entities, value objects, domain errors
├── usecases/         # Use case classes/fns, port interfaces
├── adapters/
│   ├── controllers/  # HTTP/CLI input → use case call
│   ├── presenters/   # Use case output → response format
│   └── gateways/     # DB/external service implementations
└── infra/            # Frameworks, DB clients, config, DI wiring
```

## Functions & Classes

- Fn: ≤20 lines. One level of abstraction.
- Class: ≤200 lines. One reason to change.
- Args: ≤3. >3 → extract param object.
- No side effects in pure fns. Mark impure fns clearly.
- Return early. Avoid deep nesting (max 2 levels).

## Error Handling

- Domain errors = typed, domain-language (`OrderNotFoundError`, not `Error("not found")`)
- Use cases return result type or throw domain error — never HTTP status codes
- Adapters translate domain errors → framework errors (HTTP 404, gRPC NOT_FOUND)

## Testing per Layer

| Layer | Test type | Mock? |
|---|---|---|
| Entities | Unit | No |
| Use Cases | Unit | Mock ports (interfaces) |
| Adapters | Integration | Real infra or test double |
| Infra | Integration/E2E | Real system |

## Checklist (run before every PR)

- [ ] No inner layer imports outer layer module
- [ ] Use case has no framework import (no `express`, `pg`, `axios` etc.)
- [ ] Entity has no DB/HTTP logic
- [ ] All external deps injected, not instantiated inside
- [ ] Names follow domain language (check `UBIQUITOUS_LANGUAGE.md` if exists)
- [ ] Fn ≤20 lines, class ≤200 lines, args ≤3
- [ ] Errors typed and translated at adapter boundary
- [ ] Tests cover use case logic via mocked ports

## Common Violations → Fixes

| Violation | Fix |
|---|---|
| Use case imports `express.Request` | Extract DTO, controller maps req → DTO |
| Entity calls DB directly | Move DB call to gateway, inject repository port |
| Controller has business logic | Move logic to use case, controller only maps |
| Concrete class injected in use case | Create interface, inject interface |
| God class >200 lines multiple concerns | Split by responsibility |
| Error string comparison | Typed error classes |

## References

- [Clean Architecture — Robert C. Martin](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [SOLID](https://en.wikipedia.org/wiki/SOLID)
