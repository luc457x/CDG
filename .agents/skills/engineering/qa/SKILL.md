---
name: qa
description: Interactive QA session where user reports bugs or issues conversationally, and agent files GitHub issues. Explores codebase in background for context and domain language. Use when user wants to report bugs, do QA, file issues conversationally, or mentions "QA session".
---

# QA Session

## When to Use

Run interactive QA session. User describes problems. Clarify, explore codebase for context, file GitHub issues — durable, user-focused, using project's domain language.

## For Each Issue the User Raises

### 1. Listen and Lightly Clarify

Let user describe problem in own words. Ask **at most 2-3 short clarifying questions** focused on:

- What they expected vs what actually happened
- Steps to reproduce (if not obvious)
- Whether consistent or intermittent

Do NOT over-interview. If description clear enough to file, move on.

### 2. Explore Codebase in Background

While talking to user, kick off Agent (`subagent_type=Explore`) in background to understand relevant area. Goal NOT to find fix — it's to:

- Learn domain language used in that area (check `.agents/UBIQUITOUS_LANGUAGE.md` if exists)
- Understand what feature supposed to do
- Identify user-facing behavior boundary

Context helps write better issue — but issue itself should NOT reference specific files, line numbers, or internal implementation details.

### 3. Assess Scope: Single Issue or Breakdown?

Before filing, decide: **single issue** or **broken down** into multiple.

Break down when:

- Fix spans multiple independent areas (e.g. "form validation wrong AND success message missing AND redirect broken")
- Clearly separable concerns different people could work on in parallel
- User describes something with multiple distinct failure modes or symptoms

Keep as single when:

- One behavior wrong in one place
- Symptoms all caused by same root behavior

### 4. File GitHub Issue(s)

Create issues with `gh issue create`. Do NOT ask user to review first — just file and share URLs.

Issues must be **durable** — still make sense after major refactors. Write from user's perspective.

#### For a Single Issue

```md
## What happened

[Describe actual behavior user experienced, in plain language]

## What I expected

[Describe expected behavior]

## Steps to reproduce

1. [Concrete, numbered steps developer can follow]
2. [Use domain terms from codebase, not internal module names]
3. [Include relevant inputs, flags, or configuration]

## Additional context

[Extra observations from user or codebase exploration that help frame issue — use domain language, don't cite files]
```

#### For a Breakdown (Multiple Issues)

Create issues in dependency order (blockers first) so can reference real issue numbers.

```md
## Parent issue

#<parent-issue-number> (if tracking issue created) or "Reported during QA session"

## What's wrong

[Describe this specific behavior problem — just this slice, not whole report]

## What I expected

[Expected behavior for this specific slice]

## Steps to reproduce

1. [Steps specific to THIS issue]

## Blocked by

- #<issue-number> (if can't be fixed until another resolved)

Or "None — can start immediately" if no blockers.

## Additional context

[Extra observations relevant to this slice]
```

When creating breakdown:

- **Many thin issues over few thick ones** — each independently fixable and verifiable
- **Mark blocking relationships honestly** — if issue B genuinely can't be tested until issue A fixed, say so. If independent, mark both "None — can start immediately"
- **Create in dependency order** so can reference real issue numbers in "Blocked by"
- **Maximize parallelism** — goal: multiple people (or agents) can grab different issues simultaneously

#### Rules for All Issue Bodies

- **No file paths or line numbers** — these go stale
- **Use project's domain language** (check `.agents/UBIQUITOUS_LANGUAGE.md` if exists)
- **Describe behaviors, not code** — "sync service fails to apply patch" not "applyPatch() throws on line 42"
- **Reproduction steps mandatory** — if can't determine, ask user
- **Keep concise** — developer should read issue in 30 seconds

After filing, print all issue URLs (with blocking relationships summarized) and ask: "Next issue, or done?"

### 5. Continue Session

Keep going until user says done. Each issue independent — don't batch.
