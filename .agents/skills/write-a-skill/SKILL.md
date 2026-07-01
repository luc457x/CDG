---
name: write-a-skill
description: Create new agent skills with proper structure, progressive disclosure, and bundled resources
when_to_use: user wants to create, write, or build new skill.
metadata:
  category: maintenance
---
# Writing Skills

## Process

1. **Gather requirements** — ask user about:
   - What task/domain does skill cover?
   - What specific use cases to handle?
   - Does it need executable scripts or just instructions?
   - Any reference materials to include?

2. **Draft skill** — create:
   - SKILL.md with concise instructions
   - Additional reference files if content exceeds 500 lines
   - Utility scripts if deterministic operations needed

3. **Review with user** — present draft and ask:
   - Does this cover your use cases?
   - Anything missing or unclear?
   - Should any section be more/less detailed?

4. **Validate skill** — run skill quality auditor to check structural correctness and links for new skill:
    ```bash
    python .agents/skills/audit-skills/scripts/audit_skills.py --skill [skill_name]
    ```
    Fix reported issues (broken links, missing description triggers, line limit breaches, name mismatches) before commit.

## Skill Structure

```text
skill-name/
├── SKILL.md           # Main instructions (required)
├── reference/         # Detailed docs (if needed)
├── examples/          # Usage examples (if needed)
└── scripts/           # Utility scripts (if needed)
```

## SKILL.md Template

```md
---
name: skill-name
description: Brief description of capability.
when_to_use: specific triggers (e.g., when doing X, when user asks to Y)
metadata:
  category: development
---
```

## Description and Trigger Requirements

Frontmatter contains description and when_to_use triggers, which are the main fields the agent sees when deciding which skill to load.

**Goal**: Give agent just enough info to know what capability the skill provides and exactly when to use it.

**Format**:
- `description`: Max 300 chars, third person, what skill does.
- `when_to_use`: Max 300 chars, when to trigger (e.g., "working with PDF files or user mentions PDFs").

**YAML vs. Markdown Body Rules**:
- Triggers MUST be defined in frontmatter `when_to_use`.
- Frontmatter fields MUST be concise to avoid token waste.
- "When to Use" or "Quick Start" section in markdown body is optional. Do not duplicate simple frontmatter triggers in body.
- If "When to Use" conditions are too complex for YAML frontmatter, write complete "When to Use" section in body, and write shorter summary trigger in frontmatter pointing agent to read full file.

Good example:
```yaml
---
name: pdf_utility
description: Extract text and tables from PDF files, fill forms, merge documents.
when_to_use: working with PDF files, form filling, or document extraction.
---
```

Bad example:
```yaml
---
name: pdf_utility
description: Helps with documents.
when_to_use: anytime
---
```

Bad example gives agent no way to distinguish from other document skills.

## When to Add Scripts

Add utility scripts when:

- Operation deterministic (validation, formatting)
- Same code would be generated repeatedly
- Errors need explicit handling

**Scripts save tokens and improve reliability vs generated code.**

## When to Split Files

Split into separate files when:

- SKILL.md exceeds 180 lines
- Content has distinct domains (finance vs sales schemas)
- Advanced features rarely needed

## Review Checklist

After drafting, verify:

- [ ] Has when_to_use trigger field in frontmatter
- [ ] Frontmatter description and when_to_use are under 300 characters each
- [ ] Triggers only in frontmatter unless complex (no redundant "When to Use" body section)
- [ ] SKILL.md under 180 lines
- [ ] No time-sensitive info
- [ ] Consistent terminology
- [ ] Concrete examples included
- [ ] References one level deep

