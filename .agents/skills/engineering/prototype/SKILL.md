---
name: prototype
description: Build throwaway logic or UI prototype to validate design decisions. Use when user wants to prototype, sanity-check data models, mock up UIs, explore design options, or says "prototype this" or "try designs".
---

# Prototype

Prototype = **throwaway code that answers a question**. Question decides shape.

## When to Use

Identify which question is being answered — from user's prompt, surrounding code, or by asking if user is around:

- **"Does this logic / state model feel right?"** → [LOGIC.md](./LOGIC.md). Build tiny interactive terminal app that pushes state machine through cases hard to reason about on paper.
- **"What should this look like?"** → [UI.md](./UI.md). Generate several radically different UI variations on single route, switchable via URL search param and floating bottom bar.

Two branches produce very different artifacts — getting this wrong wastes whole prototype. If question genuinely ambiguous and user not reachable, default to whichever branch better matches surrounding code (backend module → logic; page or component → UI) and state assumption at top of prototype.

## Rules (Both Branches)

1. **Throwaway from day one, clearly marked as such.** Locate prototype code close to where it will actually be used (next to module or page it's prototyping for) so context obvious — but name so casual reader can see it's prototype, not production. For throwaway UI routes, obey whatever routing convention project already uses; don't invent new top-level structure.
2. **One command to run.** Whatever project's existing task runner supports — `pnpm <name>`, `python <path>`, `bun <path>`, etc. User must be able to start it without thinking.
3. **No persistence by default.** State lives in memory. Persistence = thing prototype is _checking_, not something it should depend on. If question explicitly involves database, hit scratch DB or local file with clear "PROTOTYPE — wipe me" name.
4. **Skip polish.** No tests, no error handling beyond what makes prototype _runnable_, no abstractions. Point = learn something fast then delete.
5. **Surface state.** After every action (logic) or on every variant switch (UI), print or render full relevant state so user can see what changed.
6. **Delete or absorb when done.** When prototype answered its question, either delete or fold validated decision into real code — don't leave it rotting in repo.

## When Done

Only _answer_ worth keeping from prototype. Capture somewhere durable (commit message, ADR, issue, or `NOTES.md` next to prototype) along with question it was answering. If user around, quick conversation; if not, leave placeholder so they (or you, on next pass) can fill in verdict before deleting prototype.
