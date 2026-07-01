# HTML Report Format

Architectural review = single self-contained HTML file in OS temp directory. Tailwind and Mermaid from CDNs. Mermaid handles graph-shaped diagrams; hand-built divs and inline SVG handle editorial visuals (mass diagrams, cross-sections). Mix both — don't lean on Mermaid for everything.

## Scaffold

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <title>Architecture review — {{repo name}}</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <script type="module">
      import mermaid from "https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.esm.min.mjs";
      mermaid.initialize({ startOnLoad: true, theme: "neutral", securityLevel: "loose" });
    </script>
    <style>
      /* small custom layer for things Tailwind doesn't cover cleanly:
         dashed seam lines, hand-drawn-feeling arrow heads, etc. */
      .seam { stroke-dasharray: 4 4; }
      .leak { stroke: #dc2626; }
      .deep { background: linear-gradient(135deg, #0f172a, #1e293b); }
    </style>
  </head>
  <body class="bg-stone-50 text-slate-900 font-sans">
    <main class="max-w-5xl mx-auto px-6 py-12 space-y-12">
      <header>...</header>
      <section id="candidates" class="space-y-10">...</section>
      <section id="top-recommendation">...</section>
    </main>
  </body>
</html>
```

## Header

Repo name, date, compact legend: solid box = module, dashed line = seam, red arrow = leakage, thick dark box = deep module. No introduction paragraph — straight into candidates.

## Candidate Card

Diagrams carry weight. Prose sparse, plain, using glossary terms ([LANGUAGE.md](./LANGUAGE.md)).

Each candidate is one `<article>`:

- **Title** — short, names deepening (e.g. "Collapse Order intake pipeline").
- **Badge row** — recommendation strength (`Strong` = emerald, `Worth exploring` = amber, `Speculative` = slate), plus tag for dependency category (`in-process`, `local-substitutable`, `ports & adapters`, `mock`).
- **Files** — monospaced list, `font-mono text-sm`.
- **Before / After diagram** — centrepiece. Two columns, side by side. See patterns below.
- **Problem** — one sentence. What hurts.
- **Solution** — one sentence. What changes.
- **Wins** — bullets, ≤6 words each. e.g. "Tests hit one interface", "Pricing logic stops leaking", "Delete 4 shallow wrappers".
- **ADR callout** (if applicable) — one line in amber-tinted box.

No paragraphs of explanation. If diagram needs paragraph to be understood, redraw diagram.

## Diagram Patterns

Pick pattern that fits candidate. Mix them. Don't make every diagram look same.

### Mermaid Graph (Workhorse for Dependencies / Call Flow)

Use Mermaid `flowchart` or `graph` when point is "X calls Y calls Z, look at mess." Wrap in Tailwind-styled card. Style with classDef to colour leakage edges red and deep module dark. Sequence diagrams work for "before: 6 round-trips; after: 1."

```html
<div class="rounded-lg border border-slate-200 bg-white p-4">
  <pre class="mermaid">
    flowchart LR
      A[OrderHandler] --> B[OrderValidator]
      B --> C[OrderRepo]
      C -.leak.-> D[PricingClient]
      classDef leak stroke:#dc2626,stroke-width:2px;
      class C,D leak
  </pre>
</div>
```

### Hand-Built Boxes-and-Arrows (When Mermaid Layout Fights You)

Modules as `<div>`s with borders and labels. Arrows as inline SVG `<line>` or `<path>` positioned absolutely over relative container. Reach for this when "after" diagram should feel like one thick-bordered deep module with greyed-out internals — Mermaid won't render that with right weight.

### Cross-Section (Layered Shallowness)

Stack horizontal bands (`h-12 border-l-4`) to show layers call passes through. Before: 6 thin layers each doing nothing. After: 1 thick band labelled with consolidated responsibility.

### Mass Diagram (Interface as Wide as Implementation)

Two rectangles per module — one for interface surface area, one for implementation. Before: interface rectangle nearly as tall as implementation (shallow). After: interface rectangle short, implementation rectangle tall (deep).

### Call-Graph Collapse

Before: tree of function calls as nested boxes. After: same tree collapsed into one box, now-internal calls shown faded inside.

## Style Guidance

- Editorial, not corporate-dashboard. Generous whitespace. Serif optional for headings.
- Colour sparingly: one accent (emerald or indigo) plus red for leakage and amber for warnings.
- Keep diagrams ~320px tall so before/after sits comfortably side by side.
- Use `text-xs uppercase tracking-wider` for module labels inside diagrams — should read as schematic, not UI.
- Only scripts = Tailwind CDN and Mermaid ESM import. Report otherwise static.

## Top Recommendation Section

One larger card. Candidate name, one sentence on why, anchor link to its card.

## Tone

Plain English, concise — architectural nouns and verbs come straight from [LANGUAGE.md](./LANGUAGE.md).

**Use exactly:** module, interface, implementation, depth, deep, shallow, seam, adapter, leverage, locality.

**Never substitute:** component, service, unit (for module) · API, signature (for interface) · boundary (for seam) · layer, wrapper (for module, when you mean module).

**Phrasings that fit:**

- "Order intake module is shallow — interface nearly matches implementation."
- "Pricing leaks across seam."
- "Deepen: one interface, one place to test."
- "Two adapters justify seam: HTTP in prod, in-memory in tests."

**Wins bullets** name gain in glossary terms: *"locality: bugs concentrate in one module"*, *"leverage: one interface, N call sites"*, *"interface shrinks; implementation absorbs wrappers"*. Don't write *"easier to maintain"* or *"cleaner code"* — not in glossary, don't earn their place.

No hedging, no throat-clearing. If sentence could be bullet, make it bullet. If bullet could be cut, cut it. If term isn't in [LANGUAGE.md](./LANGUAGE.md), reach for one that is.
