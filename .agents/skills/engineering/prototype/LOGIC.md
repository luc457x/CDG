# Logic Prototype

Tiny interactive terminal app that lets user drive state model by hand. Use when question about **business logic, state transitions, or data shape** — kind of thing that looks reasonable on paper but only feels wrong once pushed through real cases.

## When This Is Right Shape

- "I'm not sure if this state machine handles edge case where X then Y."
- "Does this data model actually let me represent case where..."
- "I want to feel out what API should look like before writing it."
- Anything where user wants to **press buttons and watch state change**.

If question is "what should this look like" — wrong branch. Use UI prototype guidelines instead.

## Process

### 1. State the Question

Before writing code, write down what state model and what question being prototyped. One paragraph, in prototype's README or top-of-file comment. Logic prototype answering wrong question = pure waste — make question explicit so it can be checked later.

### 2. Pick the Language

Use whatever host project uses. If project has no obvious runtime (e.g. docs repo), ask.

Match project's existing conventions for tooling — don't add new package manager or runtime just for prototype.

### 3. Isolate Logic in Portable Module

Put actual logic — bit answering the question — behind small, pure interface that could be lifted out and dropped into real codebase later. TUI around it is throwaway; logic module shouldn't be.

Right shape depends on question:

- **Pure reducer** — `(state, action) => state`. Good when actions are discrete events and state is single value.
- **State machine** — explicit states and transitions. Good when "which actions are even legal right now" is part of question.
- **Small set of pure functions** over plain data type. Good when no implicit current state — just transformations.
- **Class or module with clear method surface** when logic genuinely owns ongoing internal state.

Pick shape that best fits question, *not* whichever easiest to wire to TUI. Keep pure: no I/O, no terminal code, no `console.log` for control flow. TUI imports it and calls into it; nothing flows other direction.

This makes prototype useful past own lifetime. When question answered, validated reducer / machine / function set can be lifted into real module — TUI shell gets deleted.

### 4. Build Smallest TUI That Exposes State

Build as **lightweight TUI** — on every tick, clear screen (`console.clear()` / `print("\033[2J\033[H")` / equivalent) and re-render whole frame. User always sees one stable view, not ever-growing scrollback.

Each frame has two parts, in order:

1. **Current state**, pretty-printed and diff-friendly (one field per line, or formatted JSON). Use **bold** for field names or section headers and **dim** for less important context (timestamps, IDs, derived values). Native ANSI escape codes fine — `\x1b[1m` bold, `\x1b[2m` dim, `\x1b[0m` reset. No need to pull in styling library unless already in project.
2. **Keyboard shortcuts**, listed at bottom: `[a] add user  [d] delete user  [t] tick clock  [q] quit`. Bold key, dim description, or vice-versa — whatever reads cleanly.

Behaviour:

1. **Initialise state** — single in-memory object/struct. Render first frame on start.
2. **Read one keystroke (or one line)** at a time, dispatch to handler that mutates state.
3. **Re-render** full frame after every action — don't append, replace.
4. **Loop until quit.**

Whole frame should fit on one screen.

### 5. Make Runnable in One Command

Add script to project's existing task runner (`package.json` scripts, `Makefile`, `justfile`, `pyproject.toml`). User should run `pnpm run <prototype-name>` or equivalent — never need to remember path.

If host project has no task runner, put command at top of prototype's README.

### 6. Hand It Over

Give user run command. They'll drive it themselves; interesting moments are when they say "wait, that shouldn't be possible" or "huh, I assumed X would be different" — those are bugs in *idea*, which is whole point. If they want new actions added, add them. Prototypes evolve.

### 7. Capture the Answer

When prototype done its job, answer to question = only thing worth keeping. If user is around, ask what it taught them. If not, leave `NOTES.md` next to prototype so answer can be filled in before prototype gets deleted.

## Anti-Patterns

- **Don't add tests.** Prototype that needs tests no longer prototype.
- **Don't wire to real database.** Use in-memory store unless question specifically about persistence.
- **Don't generalise.** No "what if we wanted to support X later." Prototype answers one question.
- **Don't blur logic and TUI together.** If reducer / state machine references `console.log`, prompts, or terminal escape codes, it's no longer portable. Keep TUI as thin shell over pure module.
- **Don't ship TUI shell into production.** Shell optimised for being driven by hand from terminal. Logic module behind it = bit worth keeping.
