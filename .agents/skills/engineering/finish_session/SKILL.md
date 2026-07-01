---
name: finish_session
description: Logs completed work to PROGRESS.md as new session entry, then commits using atomic_commit skill. Triggered at end of task, plan completion, or explicit "finish session" command.
when_to_use: implementation plan finished, task list fully done, or user says "finish session" / "wrap up" / "end session".
metadata:
  category: workflow
---
# Finish Session

## Rules

Run at end of every implementation or explicit user request. Always log before commit.

## Steps

### 1. Determine session number

Read `PROGRESS.md` last session number. Increment by 1.

### 2. Build session entry

Pull facts from current conversation — no invention. Use template below:

```md
### Session N: <Phase Title>

- Date: YYYY-MM-DD
- Agent: Antigravity
- Goal: <one-sentence purpose>
- Constraints: <hard limits or blockers, "None" if clear>
- Done:
  - <atomic change with file refs where relevant>
  - <verification evidence (test counts, commands run)>
- Blocked: <what stopped progress, "None" if clear>
- Risk: <potential regressions or edge cases>
- Artifact: <file paths for key outputs>
- Verification: <command run + result summary>
- Pending: <remaining work or "None">
```

Rules for entry:
- Phase title = short imperative noun phrase (≤5 words)
- `Done` items = atomic facts, file:line refs if changed
- `Verification` = actual command + result (pass/fail, test count)
- No invented data. If info not available, omit field, don't guess.

### 3. Write entry to PROGRESS.md

Append new session block **above** the previous session, below the `## Log` header line.

Example PROGRESS.md insert position:
```
## Log

Old sessions: [PROGRESS_ARCHIVE.md](./PROGRESS_ARCHIVE.md).

### Session N: <new>    ← insert here

### Session N-1: <prev>
```

Also update `## Status` section at top with:
- `State:` — one-line current state after this session
- `Last:` — one-line what was just done

### 4. Archive via archive_progress skill

Read and follow [archive_progress skill](../archive_progress/SKILL.md).

Run after writing new entry. Skill handles threshold check and move automatically.

### 5. Commit via atomic_commit skill

Read and follow [atomic_commit skill](../atomic_commit/SKILL.md).

Include PROGRESS.md update in `docs` commit or with most relevant code commit if only one commit.

## Example

Session wrapping up work on a new pipeline stage:

```md
### Session 12: Add Parquet Export Stage

- Date: 2026-06-30
- Agent: Antigravity
- Goal: Export processed OHLCV data to Parquet for GCS upload.
- Constraints: Must use existing polars pipeline, no new deps.
- Done:
  - Added `export_parquet()` in `src/pipeline/export.rs`.
  - Wired stage into `src/pipeline/mod.rs`.
- Blocked: None.
- Risk: Large file sizes may spike memory on low-RAM targets.
- Artifact: `src/pipeline/export.rs`.
- Verification: `cargo test` — 34 tests passed.
- Pending: None.
```
