---
name: caveman_compress
description: Compress natural language memory files into caveman format, preserving technical content and saving input tokens. Use when compressing memory files, converting natural language to caveman, or running /caveman_compress.
---

# Caveman Compress

## When to Use

Compress natural language files (AGENTS.md, todos, preferences) into caveman-speak to reduce input tokens. Compressed version overwrites original. Human-readable backup saved as `<filename>.original.md`.

## Trigger

`/caveman_compress <filepath>` or when user asks to compress memory file.

## Process

When triggered with `/caveman_compress <filepath>` or when asked to compress memory file, agent must perform compression directly using workspace file tools and inline LLM reasoning.

1. **Verify & Validate Path**:
   - Ensure target path exists and is file.
   - Reject files larger than 500KB to avoid context limit issues.
   - Refuse files looking like they contain secrets or PII (e.g. matching `.env`, `.netrc`, `credentials`, `secrets`, `passwords`, `id_rsa`, certificates `.pem`/`.key`, or containing sensitive directory names like `.ssh`, `.aws`, `.gnupg`, `.kube`, `.docker`).
   - Detect file type: only compress natural language files (`.md`, `.txt`, `.markdown`, `.rst`, `.typ`, `.typst`, `.tex`, or extensionless prose files). Skip code/config files (`.py`, `.js`, `.ts`, `.json`, `.yaml`, `.yml`, `.toml`, `.lock`, `.css`, `.html`, `.xml`, `.sql`, `.sh`, etc.).

2. **Backup**:
   - Read original text of file.
   - Compute backup path as `filepath.with_name(filepath.stem + ".original.md")`.
   - If backup file already exists, abort to prevent data loss.
   - Write original text to backup path, then read back to verify write succeeded.

3. **Compress Inline**:
   - Perform compression in agent's context using [Compression Rules](#compression-rules).
   - Do NOT wrap entire compressed content in markdown code fence (keep internal code blocks as-is).

4. **Validate & Fix**:
   - Verify headings match exactly in count and content.
   - Verify code blocks (fenced ``` and indented) match exactly (no changes to spacing, comments, or lines inside).
   - Verify inline code backticks content matches exactly.
   - Verify URLs and links match exactly.
   - Verify file paths and commands match exactly.
   - If any errors found, fix in context before writing.

5. **Write & Report**:
   - Write validated compressed content back to original file path.
   - Present success summary to user, listing validation results, original size, compressed size, and file paths.

## Compression Rules

### Remove

- Articles: `a`, `an`, `the`
- Filler: `just`, `really`, `basically`, `actually`, `simply`, `essentially`, `generally`
- Pleasantries: `"sure"`, `"certainly"`, `"of course"`, `"happy to"`, `"I'd recommend"`
- Hedging: `"it might be worth"`, `"you could consider"`, `"it would be good to"`
- Redundant phrasing: `"in order to"` → `"to"`, `"make sure to"` → `"ensure"`, `"the reason is because"` → `"because"`
- Connective fluff: `"however"`, `"furthermore"`, `"additionally"`, `"in addition"`

### Preserve EXACTLY (never modify)

- Code blocks (fenced ``` and indented)
- Inline code (`backtick content`)
- URLs and links (full URLs, markdown links)
- File paths (`/src/components/...`, `./config.yaml`)
- Commands (`npm install`, `git commit`, `docker build`)
- Technical terms (library names, API names, protocols, algorithms)
- Proper nouns (project names, people, companies)
- Dates, version numbers, numeric values
- Environment variables (`$HOME`, `NODE_ENV`)

### Preserve Structure

- All markdown headings (keep exact heading text, compress body below)
- Bullet point hierarchy (keep nesting level)
- Numbered lists (keep numbering)
- Tables (compress cell text, keep structure)
- Frontmatter/YAML headers in markdown files

### Compress

- Use short synonyms: "big" not "extensive", "fix" not "implement a solution for", "use" not "utilize"
- Fragments OK: "Run tests before commit" not "You should always run tests before committing"
- Drop "you should", "make sure to", "remember to" — just state the action
- Merge redundant bullets that say the same thing differently
- Keep one example where multiple examples show the same pattern

CRITICAL RULE:
Anything inside ``` ... ``` must be copied EXACTLY.
Do not:

- remove comments
- remove spacing
- reorder lines
- shorten commands
- simplify anything

Inline code (`...`) must be preserved EXACTLY.
Do not modify anything inside backticks.

If file contains code blocks:

- Treat code blocks as read-only regions
- Only compress text outside them
- Do not merge sections around code

## Pattern

```text
Original:
You should always make sure to run the test suite before pushing any changes to the main branch. This is important because it helps catch bugs early and prevents broken builds from being deployed to production.

Compressed:
Run tests before push to main. Catch bugs early, prevent broken prod deploys.
```

```text
Original:
The application uses a microservices architecture with the following components. The API gateway handles all incoming requests and routes them to the appropriate service. The authentication service is responsible for managing user sessions and JWT tokens.

Compressed:
Microservices architecture. API gateway route all requests to services. Auth service manage user sessions + JWT tokens.
```

## Boundaries

- ONLY compress natural language files (.md, .txt, .typ, .typst, .tex, extensionless)
- NEVER modify: .py, .js, .ts, .json, .yaml, .yml, .toml, .env, .lock, .css, .html, .xml, .sql, .sh
- If file has mixed content (prose + code), compress ONLY the prose sections
- If unsure whether something is code or prose, leave it unchanged
- Original file is backed up as FILE.original.md before overwriting
- Never compress FILE.original.md (skip it)
