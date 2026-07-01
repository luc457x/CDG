# UI Prototype

Generate **several radically different UI variations** on single route, switchable from floating bottom bar. User flips between variants in browser, picks one (or steals bits from each), then throws rest away.

If question is about logic/state rather than what something looks like — wrong branch. Use Logic prototype guidelines instead.

## When This Is Right Shape

- "What should this page look like?"
- "I want to see a few options for this dashboard before committing."
- "Try different layout for settings screen."
- Any time user would otherwise spend day picking between three vague mockups in their head.

## Two Sub-Shapes — Strongly Prefer Sub-Shape A

UI prototype much easier to judge when **butting up against rest of app** — real header, real sidebar, real data, real density. Throwaway route on own is vacuum: every variant looks fine in isolation. Default to sub-shape A whenever plausible existing page to host variants. Only reach for sub-shape B if prototype genuinely has no nearby home.

### Sub-Shape A — Adjustment to Existing Page (Preferred)

Route already exists. Variants rendered **on same route**, gated by `?variant=` URL search param. Existing data fetching, params, and auth all stay — only rendering swaps. Default; pick unless specific reason not to.

If prototype is for something without page yet but *would naturally live inside one* (new section of dashboard, new card on settings screen, new step in existing flow) — still sub-shape A. Mount variants inside host page.

### Sub-Shape B — New Page (Last Resort)

Only when thing being prototyped genuinely has no existing page to live inside — e.g. entirely new top-level surface, or flow that can't be embedded anywhere.

Create **throwaway route** following whatever routing convention project already uses — don't invent new top-level structure. Name it so it's obviously prototype (e.g. include word `prototype` in path or filename). Same `?variant=` pattern.

Before committing to sub-shape B, sanity-check: is there really no existing page this could be embedded in? Empty route hides design problems populated one would expose.

In both sub-shapes, floating bottom bar identical.

## Process

### 1. State the Question and Pick N

Default to **3 variants**. More than 5 stops being radically different and starts being noise — cap there.

Write down plan in one line, in prototype's location or top-of-file comment:

> "Three variants of settings page, switchable via `?variant=`, on existing `/settings` route."

Works whether user is here to push back or not.

### 2. Generate Radically Different Variants

Draft each variant. Hold each one to:

- Page's purpose and data it has access to.
- Project's component library / styling system (TailwindCSS, shadcn, MUI, plain CSS, whatever).
- Clear exported component name, e.g. `VariantA`, `VariantB`, `VariantC`.

Variants must be **structurally different** — different layout, different information hierarchy, different primary affordance, not just different colours. Three slightly-tweaked card grids isn't UI prototype, it's wallpaper. If two drafts come out too similar, redo one with explicit "do not use card grid" guidance.

### 3. Wire Them Together

Create single switcher component on route:

```tsx
// pseudo-code — adapt to project's framework
const variant = searchParams.get('variant') ?? 'A';
return (
  <>
    {variant === 'A' && <VariantA {...data} />}
    {variant === 'B' && <VariantB {...data} />}
    {variant === 'C' && <VariantC {...data} />}
    <PrototypeSwitcher variants={['A','B','C']} current={variant} />
  </>
);
```

Sub-shape A (existing page): keep all existing data fetching above switcher; only rendered subtree changes per variant.

Sub-shape B (new page): throwaway route under `/prototype/<name>` mounts same switcher.

### 4. Build the Floating Switcher

Small fixed-position bar at bottom-centre of screen with three pieces:

- **Left arrow** — cycles to previous variant (wraps around).
- **Variant label** — shows current variant key and, if variant exports name, that name too. e.g. `B — Sidebar layout`.
- **Right arrow** — cycles forward (wraps around).

Behaviour:

- Clicking arrow updates URL search param (use framework's router — `router.replace` on Next, `navigate` on React Router, etc) so variant shareable and reload-stable.
- Keyboard: `←` and `→` arrow keys also cycle. Don't intercept arrow keys when `<input>`, `<textarea>`, or `[contenteditable]` focused.
- Visually distinct from page (e.g. high-contrast pill, subtle shadow) so obviously not part of design being evaluated.
- Hidden in production builds — gate on `process.env.NODE_ENV !== 'production'` or equivalent, so stray prototype merge can't ship bar to users.

Put switcher in single shared component so both sub-shapes can reuse. Locate wherever shared UI lives in project.

### 5. Hand It Over

Surface URL (and `?variant=` keys). User will flip through whenever they get to it. Interesting feedback usually **"I want header from B with sidebar from C"** — that's actual design they want.

### 6. Capture the Answer and Clean Up

Once variant won, write down which one and why (commit message, ADR, issue, or `NOTES.md` next to prototype if running AFK and user hasn't responded yet). Then:

- **Sub-shape A** — delete losing variants and switcher; fold winner into existing page.
- **Sub-shape B** — promote winning variant to real route, delete throwaway route and switcher.

Don't leave variant components or switcher lying around. They rot fast and confuse next reader.

## Anti-Patterns

- **Variants differing only in colour or copy.** That's tweak, not prototype. Real variants disagree about structure.
- **Sharing too much code between variants.** Shared `<Header>` fine; shared `<Layout>` defeats point. Each variant should be free to throw out layout.
- **Wiring variants to real mutations.** Read-only prototypes fine. If variant needs to mutate, point at stub — question is "what should this look like", not "does backend work".
- **Promoting prototype directly to production.** Variant code written under prototype constraints (no tests, minimal error handling). Rewrite properly when folding in.
