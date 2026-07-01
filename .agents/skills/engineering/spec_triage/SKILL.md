---
name: spec_triage
description: Routes unspecced features to SPEC.md (small additions) or BACKLOG.md (big/risky/hard-to-revert changes) before any implementation starts.
when_to_use: planning feature not yet in SPEC.md, user proposes new behavior, agent detects missing spec for task, or user says "add to spec" / "add to backlog".
metadata:
  category: workflow
---
# Spec Triage

## Rules

Run before building any feature not found in `SPEC.md`. Never implement from backlog directly.

## Decision: SPEC or BACKLOG?

Ask these questions in order. First `YES` determines destination.

| Question | YES → |
|---|---|
| Already exists in `SPEC.md`? | Already specced. No action. |
| Hard to revert? (schema change, data migration, auth change, API break) | `BACKLOG.md` |
| Touches >2 subsystems or >3 files estimated? | `BACKLOG.md` |
| Risky: security, data loss, performance regression possible? | `BACKLOG.md` |
| Small additive change to existing behaviour? (new flag, new field, new log line) | `SPEC.md` |
| Clarification or constraint on existing FR/BR/NFR? | `SPEC.md` |
| Default → uncertain or ambiguous scope | `BACKLOG.md` |

**When in doubt → BACKLOG.** Promote to SPEC only after explicit user approval.

## Process

### 1. Read SPEC.md

Scan `SPEC.md` for existing FR/BR/NFR that covers the feature. If found: stop, note the relevant ID.

### 2. Classify

Apply decision table above. Produce:
- **Destination**: `SPEC.md` or `BACKLOG.md`
- **Rationale**: one line why

### 3. Confirm with user (if ambiguous)

If classification unclear, present:
```
Feature: <name>
Destination: BACKLOG (or SPEC)
Reason: <one line>
Proceed? (yes / change destination)
```

Skip confirmation if trigger is unambiguous (e.g., user explicitly said "add to backlog").

### 4. Write entry

**BACKLOG.md** — append under `## Items`:
```md
<!-- Format: Title — 1-line description -->
**<FeatureName>** — <one-line description of what and why>
```

**SPEC.md** — append new item to appropriate section:
- New functional requirement → `## Functional Requirements`, next `FRN+1`
- New business rule → `## Business Rules`, next `BRN+1`
- New non-functional requirement → `## Non-Functional Requirements`, next `NFRN+1`

SPEC.md entry format:
```md
- **FRN (<Short Name>)**: <Precise requirement statement. Testable. No ambiguity.>
```

### 5. Confirm done

Report:
```
✓ Added to <SPEC.md|BACKLOG.md>: <entry ID or title>
```

Do NOT implement. Stop here unless user says to proceed.

## Examples

**Small → SPEC:**
> "Add `--dry-run` flag that skips GCS uploads"

Additive CLI flag, no schema change, touches 1–2 files → `SPEC.md` as next FR.

**Big → BACKLOG:**
> "Migrate SQLite cache to PostgreSQL"

Schema migration, new infra, hard to revert, touches many files → `BACKLOG.md`.

**Ambiguous → Ask:**
> "Support multi-user auth"

Auth change = security + hard to revert → classify `BACKLOG`, confirm with user before writing.
