---
name: write_a_skill
description: Create new agent skills with proper structure, progressive disclosure, and bundled resources. Use when user wants to create, write, or build new skill.
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
   python .agents/skills/engineering/audit_skills/scripts/audit_skills.py --skill [skill_name]
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
name: skill_name
description: Brief description of capability. Use when [specific triggers].
---

# Skill Name

## When to Use (Optional)

[Optional: Detailed rules about when to use this skill. Only include if too detailed/long for frontmatter YAML.]

## Quick Start (Optional)

[Optional: Minimal working example]

## Workflows

[Step-by-step processes with checklists for complex tasks]

## Advanced Features

[Link to separate files: See [REFERENCE.md](./REFERENCE.md)]
```

## Description Requirements

Description = **only thing agent sees** when deciding which skill to load. Surfaced in system prompt alongside all other installed skills. Agent reads descriptions and picks relevant skill based on user's request.

**Goal**: Give agent just enough info to know:

1. What capability skill provides
2. When/why to trigger it (specific keywords, contexts, file types)

**Format**:

- Max 512 chars
- Write in third person
- First sentence: what it does
- Second sentence: "Use when [specific triggers]"

**YAML vs. Markdown Body Rules**:
- Triggers MUST be defined in frontmatter description.
- Frontmatter description MUST be concise (under 300 characters) to avoid token waste.
- "When to Use" or "Quick Start" section in markdown body is optional. Do not duplicate simple frontmatter triggers in body.
- If "When to Use" conditions are too complex for YAML frontmatter, write complete "When to Use" section in body, and write shorter summary description with "Use when..." phrase in frontmatter (under 300 characters) pointing agent to read full file.

Good example:

```text
Extract text and tables from PDF files, fill forms, merge documents. Use when working with PDF files or when user mentions PDFs, forms, or document extraction.
```

Bad example:

```text
Helps with documents.
```

Bad example gives agent no way to distinguish from other document skills.

## When to Add Scripts

Add utility scripts when:

- Operation deterministic (validation, formatting)
- Same code would be generated repeatedly
- Errors need explicit handling

Scripts save tokens and improve reliability vs generated code.

## When to Split Files

Split into separate files when:

- SKILL.md exceeds 180 lines
- Content has distinct domains (finance vs sales schemas)
- Advanced features rarely needed

## Review Checklist

After drafting, verify:

- [ ] Description includes triggers ("Use when...")
- [ ] Frontmatter description is under 512 characters
- [ ] Triggers only in frontmatter description unless complex (no redundant "When to Use" body section)
- [ ] SKILL.md under 180 lines
- [ ] No time-sensitive info
- [ ] Consistent terminology
- [ ] Concrete examples included
- [ ] References one level deep
