---
name: ai_structure_audit
description: Systematic audit of .agents/ AI engineering structure for consistency, completeness, and prompt quality. Use when checking agent engineering structure, validating directory format, or running structure audit.
---

# Structure Audit Skill

## When to Use

When validating `.agents/` engineering structure after initial setup, after significant template changes, or before bootstrapping new project from template. Also useful as periodic health check on living project.

## Goal

Detect internal contradictions, hollow placeholders, missing protocols, and prompt engineering weaknesses across all `.agents/` documentation before they cause non-deterministic agent behavior. **Strictly read-only diagnostic process. Do NOT edit, modify, or create any files in repo during audit (no typo fixes, no gitignore files, etc.). Report findings by writing audit report as artifact (`audit_report.md`) in artifact directory, and providing brief summary in chat.**

Automated scan tool available at [audit_helper.py](./scripts/audit_helper.py) to automate placeholder, formatting, and link checking.

## Process

Perform the audit following the steps below. For the comprehensive, sub-item checklist of each step, refer to [AUDIT_CHECKLIST.md](./reference/AUDIT_CHECKLIST.md).

### Steps

1. **[Read Entry Point & Ignore](./reference/AUDIT_CHECKLIST.md#1-read-entry-point--ignore)**: Read `AGENTS.md` and `.agentignore`. Define scope.
2. **[Run Automated Scan](./reference/AUDIT_CHECKLIST.md#2-run-automated-scan)**: Run [audit_helper.py](./scripts/audit_helper.py). Read the pre-computed baseline report.
3. **[Inventory — Map Full Structure](./reference/AUDIT_CHECKLIST.md#3-inventory--map-full-structure)**: Verify directories and files. Flag anomalies.
4. **[Read — Full Sequential Scan](./reference/AUDIT_CHECKLIST.md#4-read--full-sequential-scan)**: Sequential manual scan of all project and context files.
5. **[Cross-Reference Integrity](./reference/AUDIT_CHECKLIST.md#5-cross-reference-integrity)**: Verify all file links and index coverage are valid and formatted correctly.
6. **[Placeholder Density Analysis](./reference/AUDIT_CHECKLIST.md#6-placeholder-density-analysis)**: Locate bracket placeholders and empty sections. Classify file completion.
7. **[Rule Consistency Check](./reference/AUDIT_CHECKLIST.md#7-rule-consistency-check)**: Verify structure matches declared language, loading, and methodology rules.
8. **[Style Consistency Check](./reference/AUDIT_CHECKLIST.md#8-style-consistency-check)**: Verify headers, numbering, frontmatter, and emphasis conventions.
9. **[Prompt Engineering Quality](./reference/AUDIT_CHECKLIST.md#9-prompt-engineering-quality)**: Assess determinism, completeness, and specificity of directives.
10. **[Coupling & Cohesion Audit](./reference/AUDIT_CHECKLIST.md#10-coupling--cohesion-audit)**: Evaluate single responsibility, dependency fan-out/fan-in, and circular coupling.
11. **[Overengineering Check](./reference/AUDIT_CHECKLIST.md#11-overengineering-check)**: Check file-to-scope and skill-to-task ratios. Classify overengineering state.
12. **[Token Efficiency & Cost Analysis](./reference/AUDIT_CHECKLIST.md#12-token-efficiency--cost-analysis)**: Check language cost, format-fit, and redundancy.
13. **[Scope & Architecture Fit](./reference/AUDIT_CHECKLIST.md#13-scope--architecture-fit)**: Check instance vs. template mapping, agent role actionability, and context budgets.
14. **[Git Exposure Check](./reference/AUDIT_CHECKLIST.md#14-git-exposure-check)**: Verify repository exposure and check exclusions of the `.agents/` folder.
15. **[Enrich and Finalize Report](./reference/AUDIT_CHECKLIST.md#15-enrich-and-finalize-report)**: Write the final manual enriched report to the artifact directory.
