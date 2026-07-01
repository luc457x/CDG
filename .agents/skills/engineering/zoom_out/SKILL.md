---
name: zoom_out
description: Disciplined protocol to map high-level codebase architecture, trace component boundaries, and establish mental models of unfamiliar modules. Use when getting lost in low-level details, onboarding to new subsystem, or when explicitly asked to "zoom out".
---

# Zoom Out

Protocol to escape line-by-line detail and construct high-level architectural map of codebase or subsystem.

## When to Use

- **Onboarding:** First time looking at new repository or major subsystem.
- **Lost in Details:** When debugging/coding feels like "whack-a-mole" — don't understand side effects or callers.
- **Before Refactoring:** Understand dependencies and boundaries before moving code.
- **Explicit Request:** User says "zoom out" or "give high-level view".

## Step-by-Step Mapping Protocol

Follow these phases to build structured, high-level mental map:

### Phase 1 — Identify Entry Points

Locate where execution enters module or system:

1. **Public APIs / Exporters:** Identify exported functions, interfaces, HTTP endpoints, or CLI commands.
2. **Configuration / Wiring:** Find where dependencies injected, routes registered, or services initialized.
3. **Data Ingestion:** Find where external data enters (event queues, database listeners, webhook handlers).

### Phase 2 — Trace Dependencies & Data Flow

Determine what module talks to and how data moves:

- **Upward (Callers):** What orchestrates or calls this module?
- **Downward (Callees):** What helper libraries, databases, or external APIs does module call?
- **Sideways (Seams):** Where are mockable boundaries or abstract interfaces?

### Phase 3 — Document the Landscape

Construct visual and conceptual map of area:

1. **Visual Diagram:** Draw clean ASCII flow/dependency diagram or Mermaid chart showing relationship between core components.
2. **Responsibilities:** For each major file or component, write single-sentence definition of core responsibility (matching **Ubiquitous Language** glossary).
3. **Architectural Pattern:** Identify primary pattern in play (e.g., Clean Architecture, Hexagonal, Layered, Event-Driven, Pipeline).

## Anti-Patterns

- **No line-by-line explanations:** Never paste large blocks of code. Describe _shapes_ and _flows_, not implementation.
- **No deep dives without context:** Do not focus on utility functions or edge-case handling logic until core path mapped.
- **Ignoring boundaries:** Do not treat external systems (DB, third-party APIs) as black boxes if understanding their schema necessary for layout.
