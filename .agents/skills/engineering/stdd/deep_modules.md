# Deep Modules

From "A Philosophy of Software Design":

Formal definitions of depth, module, interface, and related terms: see [LANGUAGE.md](../improve_codebase_architecture/LANGUAGE.md).

**Deep module** = small interface + lots of implementation

```text
┌─────────────────────┐
│   Small Interface   │  ← Few methods, simple params
├─────────────────────┤
│                     │
│                     │
│  Deep Implementation│  ← Complex logic hidden
│                     │
│                     │
└─────────────────────┘
```

**Shallow module** = large interface + little implementation (avoid)

```text
┌─────────────────────────────────┐
│       Large Interface           │  ← Many methods, complex params
├─────────────────────────────────┤
│  Thin Implementation            │  ← Just passes through
└─────────────────────────────────┘
```

When designing interfaces, ask:

- Can I reduce method count?
- Can I simplify parameters?
- Can I hide more complexity inside?
