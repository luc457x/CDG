---
name: archive_progress
description: Cleans PROGRESS.md by archiving oldest sessions to PROGRESS_ARCHIVE.md when session count exceeds 3. Keeps newest 2 sessions in PROGRESS.md after archive threshold triggers.
when_to_use: called by finish-session after writing new session entry, or explicitly when user asks to archive or clean progress log.
metadata:
  category: workflow
---
# Archive Progress

## Rules

Run after every new session entry written to PROGRESS.md. Never delete sessions — only move.

## Steps

### 1. Count sessions

Count `### Session` header blocks under `## Log` in `PROGRESS.md`.

### 2. Check threshold

- Count ≤ 3 → **stop, nothing to archive.**
- Count > 3 → **archive oldest sessions until 2 remain.**

Sessions to archive = all except 2 with highest session numbers.

### 3. Extract sessions to archive

Sessions ordered newest-first under `## Log`. Oldest sessions are at bottom.

Extract all sessions except top 2. Preserve exact content verbatim.

### 4. Prepend to PROGRESS_ARCHIVE.md

Open `PROGRESS_ARCHIVE.md`. Find `## Archived Sessions` header.

Prepend extracted sessions **immediately below** `## Archived Sessions` line, ordered newest-first (highest number first).

If `## Archived Sessions` section missing, add after file title.

### 5. Remove from PROGRESS.md

Delete extracted session blocks from `PROGRESS.md`.

`## Log` section after cleanup must contain exactly 2 session blocks.

### 6. Verify

Confirm:
- `PROGRESS.md` has exactly 2 `### Session` blocks under `## Log`.
- `PROGRESS_ARCHIVE.md` contains moved sessions in `## Archived Sessions` section.
- No session content lost (entry count = before - 2 remaining, all moved to archive).

## Example

4 sessions in PROGRESS.md:
```
### Session 27: ...   ← newest, keep
### Session 26: ...   ← keep
### Session 25: ...   ← archive
### Session 24: ...   ← archive
```
