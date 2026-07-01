# Language

Shared vocabulary for every suggestion this skill makes. Use these terms exactly — don't substitute "component," "service," "API," or "boundary." Consistent language is point.

## Terms

**Module**
Anything with interface and implementation. Deliberately scale-agnostic — applies equally to function, class, package, or tier-spanning slice.
_Avoid_: unit, component, service.

**Interface**
Everything caller must know to use module correctly. Includes type signature, but also invariants, ordering constraints, error modes, required configuration, and performance characteristics.
_Avoid_: API, signature (too narrow — refer only to type-level surface).

**Implementation**
What's inside module — its body of code. Distinct from **Adapter**: thing can be small adapter with large implementation (Postgres repo) or large adapter with small implementation (in-memory fake). Reach for "adapter" when seam is topic; "implementation" otherwise.

**Depth**
Leverage at interface — amount of behaviour caller (or test) can exercise per unit of interface they have to learn. Module is **deep** when large amount of behaviour sits behind small interface. Module is **shallow** when interface nearly as complex as implementation.

**Seam** _(from Michael Feathers)_
Place where behaviour can be altered without editing in that place. _Location_ at which module's interface lives. Choosing where to put seam is own design decision, distinct from what goes behind it.
_Avoid_: boundary (overloaded with DDD's bounded context).

**Adapter**
Concrete thing that satisfies interface at seam. Describes _role_ (what slot it fills), not substance (what's inside).

**Leverage**
What callers get from depth. More capability per unit of interface they have to learn. One implementation pays back across N call sites and M tests.

**Locality**
What maintainers get from depth. Change, bugs, knowledge, and verification concentrate at one place rather than spreading across callers. Fix once, fixed everywhere.

## Principles

- **Depth is property of interface, not implementation.** Deep module can be internally composed of small, mockable, swappable parts — they just aren't part of interface. Module can have **internal seams** (private to implementation, used by own tests) as well as **external seam** at interface.
- **Deletion test.** Imagine deleting module. If complexity vanishes, module wasn't hiding anything (pass-through). If complexity reappears across N callers, module was earning its keep.
- **Interface is test surface.** Callers and tests cross same seam. If want to test _past_ interface, module is probably wrong shape.
- **One adapter = hypothetical seam. Two adapters = real one.** Don't introduce seam unless something actually varies across it.

## Relationships

- **Module** has exactly one **Interface** (surface it presents to callers and tests).
- **Depth** is property of **Module**, measured against its **Interface**.
- **Seam** is where **Module**'s **Interface** lives.
- **Adapter** sits at **Seam** and satisfies **Interface**.
- **Depth** produces **Leverage** for callers and **Locality** for maintainers.

## Rejected Framings

- **Depth as ratio of implementation-lines to interface-lines** (Ousterhout): rewards padding implementation. We use depth-as-leverage instead.
- **"Interface" as TypeScript `interface` keyword or class's public methods**: too narrow — interface here includes every fact caller must know.
- **"Boundary"**: overloaded with DDD's bounded context. Say **seam** or **interface**.
