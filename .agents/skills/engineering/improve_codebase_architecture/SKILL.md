---
name: improve_codebase_architecture
description: Find deepening opportunities in codebase, informed by domain language in SPEC.md and decisions in .agents/adr/. Use when user wants to improve architecture, find refactoring opportunities, consolidate tightly-coupled modules, or make codebase more testable and AI-navigable.
---

# Improve Codebase Architecture

Surface architectural friction and propose **deepening opportunities** — refactors that turn shallow modules into deep ones. Aim: testability and AI-navigability.

## Glossary

Use these terms exactly in every suggestion. Consistent language is point — don't drift into "component," "service," "API," or "boundary." Full definitions in [LANGUAGE.md](./LANGUAGE.md).

- **Module** — anything with interface and implementation (function, class, package, slice).
- **Interface** — everything caller must know to use module: types, invariants, error modes, ordering, config. Not just type signature.
- **Implementation** — code inside.
- **Depth** — leverage at interface: lots of behaviour behind small interface. **Deep** = high leverage. **Shallow** = interface nearly as complex as implementation.
- **Seam** — where interface lives; place behaviour can be altered without editing in place. (Use this, not "boundary.")
- **Adapter** — concrete thing satisfying interface at seam.
- **Leverage** — what callers get from depth.
- **Locality** — what maintainers get from depth: change, bugs, knowledge concentrated in one place.

Key principles (see [LANGUAGE.md](./LANGUAGE.md) for full list):

- **Deletion test**: imagine deleting module. If complexity vanishes, it was pass-through. If complexity reappears across N callers, it was earning its keep.
- **Interface is test surface.**
- **One adapter = hypothetical seam. Two adapters = real seam.**

Skill _informed_ by project's domain model. Domain language gives names to good seams; ADRs record decisions skill should not re-litigate.

## Process

### 1. Explore

Read project's domain glossary/specs in SPEC.md and any ADRs in `.agents/adr/` first.

Then use Agent tool with `subagent_type=Explore` to walk codebase. Don't follow rigid heuristics — explore organically, note where friction experienced:

- Where does understanding one concept require bouncing between many small modules?
- Where are modules **shallow** — interface nearly as complex as implementation?
- Where have pure functions been extracted just for testability, but real bugs hide in how they're called (no **locality**)?
- Where do tightly-coupled modules leak across seams?
- Which parts untested, or hard to test through current interface?

Apply **deletion test** to anything suspected shallow: would deleting it concentrate complexity, or just move it? "Yes, concentrates" = signal wanted.

### 2. Present Candidates as HTML Report

Write self-contained HTML file to OS temp directory — nothing lands in repo. Resolve temp dir from `$TMPDIR`, fallback to `/tmp` (or `%TEMP%` on Windows), write to `<tmpdir>/architecture-review-<timestamp>.html` so each run gets fresh file. Open for user — `xdg-open <path>` on Linux, `open <path>` on macOS, `start <path>` on Windows — tell them absolute path.

Report uses **Tailwind via CDN** for layout/styling, **Mermaid via CDN** for diagrams where graph/flow/sequence reliably communicates structure. Mix Mermaid with hand-crafted CSS/SVG — use Mermaid when relationships graph-shaped (call graphs, dependencies, sequences), hand-built divs/SVG when more editorial (mass diagrams, cross-sections, collapse animations). Each candidate gets **before/after visualisation**. Be visual.

Each candidate as card:

- **Files** — which files/modules involved
- **Problem** — why current architecture causing friction
- **Solution** — plain English description of what would change
- **Benefits** — explained in terms of locality and leverage, how tests would improve
- **Before / After diagram** — side-by-side, custom-drawn, illustrating shallowness and deepening
- **Recommendation strength** — one of `Strong`, `Worth exploring`, `Speculative`, rendered as badge

End report with **Top recommendation** section: which candidate to tackle first and why.

**Use SPEC.md vocabulary for domain, [LANGUAGE.md](./LANGUAGE.md) vocabulary for architecture.** If `SPEC.md` defines "Order," talk about "Order intake module" — not "FooBarHandler," not "Order service."

**ADR conflicts**: if candidate contradicts existing ADR in `.agents/adr/`, only surface when friction real enough to warrant revisiting. Mark clearly in card (e.g. warning callout: _"contradicts ADR-0007 — but worth reopening because…"_). Don't list every theoretical refactor ADR forbids.

See [HTML_REPORT.md](./HTML_REPORT.md) for full HTML scaffold, diagram patterns, and styling guidance.

Do NOT propose interfaces yet. After file written, ask user: "Which of these would you like to explore?"

### 3. Grilling Loop

Once user picks candidate, drop into grilling conversation. Walk design tree — constraints, dependencies, shape of deepened module, what sits behind seam, what tests survive.

Side effects happen inline as decisions crystallize:

- **Naming deepened module after concept not in `SPEC.md`?** Add term/rule to `SPEC.md` — same discipline as `grill_me`.
- **Sharpening fuzzy term during conversation?** Update `SPEC.md` right there.
- **User rejects candidate with load-bearing reason?** Offer ADR, framed as: _"Want me to record this as ADR so future architecture reviews don't re-suggest it?"_ Only offer when reason would be needed by future explorer to avoid re-suggesting same thing — skip ephemeral reasons ("not worth it right now") and self-evident ones. See [0000_TEMPLATE.md](../../../adr/0000_TEMPLATE.md).
- **Want to explore alternative interfaces for deepened module?** See [INTERFACE_DESIGN.md](./INTERFACE_DESIGN.md).
