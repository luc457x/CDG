---
name: caveman-compress
description: Compress natural language memory files into caveman format
when_to_use: user asks to compress file or invoke /caveman-compress.
metadata:
  category: utility
---
# Caveman Compress

Compress natural language files (`.md`, `.txt`) to reduce input tokens.

## When to Use

ACTIVE: `/caveman-compress <filepath>`

## Process

1. **Verify**: Path exists, file <500KB. Skip code/config files (`.py`, `.js`, `.json`, etc.).
2. **Backup**: Write original to `<filepath>.original.md`. Abort if backup exists.
3. **Compress**: Drop articles (`a`, `an`, `the`), filler (`just`, `really`), pleasantries. Keep code blocks, inline code, URLs, paths, commands, technical terms **exactly**.
4. **Validate**: Headings match. Code blocks preserved exactly. Links, paths, commands match.
5. **Write**: Overwrite original. Report size reduction.

## Preserve EXACTLY

- Code blocks (fenced ``` and indented)
- Inline code (`backticks`)
- URLs and links
- File paths and commands
- Technical terms
- Dates, versions, numeric values

## Pattern

```text
Original: You should always make sure to run the test suite before pushing...

Compressed: Run tests before push to main. Catch bugs early, prevent broken prod deploys.
```

## Boundaries

- ONLY compress `.md`, `.txt`, `.typ`, `.typst`, `.tex`, extensionless prose
- NEVER modify `.py`, `.js`, `.ts`, `.json`, `.yaml`, `.toml`, `.env`, `.lock`, etc.
- Mixed content: compress prose only
- Original backed up as `<filename>.original.md`