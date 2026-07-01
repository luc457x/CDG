---
name: atomic_commit
description: Atomic git commit protocol using conventional, caveman-terse messages. Use when committing code, saving progress, ending session, or running atomic commits.
---

# Atomic Commit Skill

## When to Use

When saving workspace changes, closing tasks, or ending session.

## Goal

Commit changes in atomic, prioritized blocks (Core > Infra > Docs) using conventional, terse, fluff-free messages. New features on isolation branches — merge only when fully functional.

## Steps

1. **Prioritize & Group**: Split changes into atomic blocks (Core logic > Infrastructure > Docs).
2. **Verify Branch**: New features on separate feature branch.
3. **Detect Mode**: Use `github-mcp-server` (GitHub MCP mode) if available; otherwise local Git CLI.
4. **Commit**: Apply Conventional Commit rules below for each block.
5. **Tag Session**: On final commit of session, tag it `session-XX` (via GitHub MCP release/push or local tags).
6. **Verify**: Confirm commit landed (`git log --oneline -3` or MCP response) before next block.

## Commit Message Rules

- Subject Line: `<type>(<scope>): <imperative summary>` (e.g. `feat(auth): add OAuth flow`)
  - Types: `feat`, `fix`, `refactor`, `perf`, `docs`, `test`, `chore`, `build`, `ci`, `style`, `revert`
  - Imperative mood: "add", "fix" (not "added", "fixing")
  - ≤50 chars, no trailing period.
- Body (Optional): Only add for non-obvious *why*, breaking changes, or issue links (wrap at 72 chars).
- No Fluff: No "I", "we", "This commit", or AI attribution comments.

## References

- Local CLI examples: See [CLI_EXAMPLES.md](./examples/CLI_EXAMPLES.md).
- GitHub MCP examples: See [MCP_EXAMPLES.md](./examples/MCP_EXAMPLES.md).
