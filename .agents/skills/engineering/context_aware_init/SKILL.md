---
name: context_aware_init
description: Stack-aware repository initialization and repair protocol. Use when initializing repository, setting up folder structure, or repairing stack configuration.
---

# Context-Aware Initialization Skill

## When to Use

- Start of new project.
- Mid-project when repo missing crucial configs (e.g., `.gitignore`, `.agentignore`, env templates, basic directory patterns).

## Goal

Establish or repair technical foundation pre-optimized for tech stack and AI-friendly (`.agentignore` hides unnecessary dirs like `node_modules`, builds, etc.).

## Steps

1. **Stack & State Detection**:
   - Identify existing languages, frameworks, and tools.
   - Check if Git initialized.
   - Audit directory for missing configs and ignores (`.gitignore`, `.agentignore`, `.venv`).

2. **Structure Audit**:
   - Recommend standard folders (`/src`, `/tests`) if absent and appropriate for detected stack.

3. **Incremental Configuration**:
   - **Git**: If no repo found, init with `git init`.
   - **.gitignore**: Create if missing. If existing, append missing ignores (IDE, build artifacts, `.venv`) without overwriting. Audit: if rules for AI agent files (`AGENTS.md`, `.agentignore`, `.agents/`) or Python venvs (`.venv/`, `*/.venv/`) are commented out (`#`), uncomment to activate.
   - **.agentignore**: Create if missing. If existing, verify hides large/binary dirs (`node_modules`, `.venv`) to keep AI context clean.
   - **Python Env**: If Python scripts used in workflow, ensure local `.venv` configured and ignored.

4. **Verification & Commit**:
   - Verify all config files correctly formatted.
   - Commit changes following `atomic_commit` guidelines.
