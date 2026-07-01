---
name: atomic-commit
description: Commit atomic, prioritized blocks (Core > Infra > Docs)
when_to_use: saving workspace changes, closing tasks, ending session
metadata:
  category: workflow
---

## Steps

1. **Group**: Split into atomic blocks (Core logic > Infrastructure > Docs)
2. **Verify**: New features on separate feature branch
3. **Commit**: Use conventional format `type(scope): imperative-summary`
4. **Tag**: Only substantial edits, 2 words max hyphenated

## Format

- Types: `feat`, `fix`, `refactor`, `perf`, `docs`, `test`, `chore`, `build`, `ci`, `style`, `revert`
- Subject: ≤50 chars, imperative mood, no trailing period
- Body: Only for non-obvious *why* or issue links