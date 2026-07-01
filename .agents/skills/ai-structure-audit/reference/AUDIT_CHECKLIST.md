# AI Structure Audit - Detailed Checklist

Reference guide containing all detailed verification sub-steps and final report formatting guidelines for the AI structure audit.

## Steps Checklist

### 1. Read Entry Point & Ignore

Read root entry point and ignore rules first to gather clean view of project and ensure files/folders specified in `.agentignore` are excluded in all subsequent steps.

#### 1a. Read Entry Point
- Read root entry point (`AGENTS.md` or equivalent Constitution) to gather project "philosophy".

#### 1b. Read Ignore List
- Read `.agentignore` to retrieve list of folders and files to exclude from mapping, scanning, and auditing.

---

### 2. Run Automated Scan

Run automated helper script first to bootstrap audit report and gather initial context, preventing redundant manual verification.

#### 2a. Run Script
- Execute script: [./scripts/audit_helper.py](../scripts/audit_helper.py). Ensure script excludes any dirs and files listed in `.agentignore`.
- Generates baseline `audit_report.md` with:
  - File inventory table (paths, sizes, purpose, status)
  - Style consistency violations summary (headers, link formats, frontmatter, emphasis overuse)
  - Coupling & cohesion details (fan-out, fan-in, circular reference pairs)
  - Overengineering metrics (file counts, task ratios, dead files, hop depth)
  - Critical, medium, and low gaps for placeholders and invalid links.

#### 2b. Inspect Report
- Open and read generated `audit_report.md` artifact.
- **CRITICAL**: Use script's output as pre-computed baseline for subsequent steps. Do not manually scan files for placeholders, relative link formats, file sizes, or circular couplings.

---

### 3. Inventory — Map Full Structure

Verify and refine mapped structure of repo and `.agents/` directory:

#### 3a. List Directory
- Verify list of directories recursively. (Use Step 2's inventory table as base context. Do not skip any directory, except those in `.agentignore`.)

#### 3b. Catalog Files
- Confirm all files categorized with path, size, and stated purpose. (Refer to Step 2's inventory table.)

#### 3c. Flag Anomalies
- Flag unexpected files, empty directories, or missing expected files (e.g. skill directory lacking `SKILL.md`).

---

### 4. Read — Full Sequential Scan

Perform full sequential scan of project structure and context files to understand semantics, guidelines, and quality:

#### 4a. Read All Files
- Read **every** file in `.agents/`, including skills, rules, examples, and design docs (script only does structural checks — manual read required to evaluate prompt quality, architecture fit, and logic consistency).

#### 4b. Complete Context Scan
- Do **not** skip files based on loading rules (except those in `.agentignore`). Full semantic context required.

---

### 5. Cross-Reference Integrity

Verify all inter-file references valid, bidirectional, and properly formatted:

#### 5a. Index Verification
- Verify rule files and skill dependencies point to real content.

#### 5b. Reference Validity
- Verify every file referencing another (e.g. `rules/workflow.md` referencing `SPEC.md`) points to real, populated content.

#### 5c. Skill Dependencies
- Verify skills referencing other files have those dependencies satisfied.

#### 5d. Artifact Reference
- Verify UML/design artifacts referenced from `SPEC.md` or documented in index.

#### 5e. Link Format Consistency
- Review link formatting gaps identified in Step 2's Gaps table. Ensure all cross-file references use relative markdown links (`[label](./relative/path)`).

---

### 6. Placeholder Density Analysis

Analyze placeholders and empty sections using gaps pre-computed in Step 2:

#### 6a. Bracket Placeholders
- Refer to Step 2's Gaps table to locate bracket placeholders: `{e.g., ...}`, `{Define ...}`, `{Item 1}`.

#### 6b. Empty Sections
- Locate empty sections (headers with no content beneath) identified in Step 2.

#### 6c. File Classification
- Review classifications: **Instantiated** (0 placeholders), **Partial** (some filled), or **Hollow** (mostly/all placeholders).

#### 6d. Dependency Impact
- Flag if critical dependencies (e.g. `SPEC.md`) are Hollow or Critical.

---

### 7. Rule Consistency Check

Verify structure does not violate own declared rules:

#### 7a. Language Rule
- Verify all content matches declared language policy (check comments, UML, examples).

#### 7b. Loading Rules
- Verify loading rules logically coherent (no circular dependencies or ambiguous conditions).

#### 7c. Methodology Alignment
- Verify loading and rule patterns align with methodology declared in `AGENTS.md`.

#### 7d. Dependency Governance
- Verify approved dependencies in `rules/engineering.md` are concrete decisions.

---

### 8. Style Consistency Check

Verify uniform surface style using Step 2's Style Consistency Summary:

#### 8a. Header Naming Convention
- Check majority convention for top-level headers (e.g., `# Title (FILENAME.md)`) vs. bare titles and verify violations using Step 2's style summary.

#### 8b. Section Numbering
- Check section numbering consistency across peer-level files.

#### 8c. Frontmatter Presence
- Check all skills have YAML frontmatter, and non-skill files do not.

#### 8d. Link Syntax Uniformity
- Check link syntax uniformity using Step 2's pre-computed report.

#### 8e. Bullet Style
- Check mixed bullet characters (`-` and `*`) not present in any file.

#### 8f. Emphasis Conventions
- Confirm files with emphasis overuse (`**` used in >10% of lines) from Step 2.

---

### 9. Prompt Engineering Quality

Evaluate each directive for LLM executability and prompt engineering standards:

#### 9a. Determinism
- Flag vague language ("~3 sessions", "when needed", "if appropriate").

#### 9b. Completeness
- Check for missing protocols (error recovery, escalation, rollback, max retries).

#### 9c. Specificity
- Check if skills define exact file-path rules, regex patterns, or format specs.

---

### 10. Coupling & Cohesion Audit

Evaluate documentation coupling and single responsibility:

#### 10a. Single Responsibility
- Flag skills with descriptions listing multiple distinct actions, or step-by-steps mixing unrelated domains.

#### 10b. Dependency Fan-out & Fan-in
- Review fan-out (links out) and fan-in (links in) values pre-computed in Step 2's table.

#### 10c. Bidirectional Coupling
- Review circular reference pairs (A ↔ B) identified in Step 2.

#### 10d. Cohesion of Core Docs
- Check for domain leakage between `rules/engineering.md`, `rules/workflow.md`, and `SPEC.md`.

#### 10e. Skill Overlap
- Compare skill descriptions for potential duplication (shared goals >50%).

---

### 11. Overengineering Check

Assess whether structure matches project's complexity:

#### 11a. File-to-Scope Ratio
- Compare total files to project scope.

#### 11b. Skill-to-Scope Ratio
- Check pre-computed ratio of skills to project scope. Projects without `TASKS.md` should be evaluated on dead-file count and hop depth instead.

#### 11c. Dead Structure
- Inspect dead files list (zero references) from Step 2.

#### 11d. Abstraction Depth
- Review max hop depth from entry point (`AGENTS.md`) pre-computed in Step 2.

#### 11e. Verdict Label
- Classify as: **Proportionate**, **Premature**, or **Bloated**.

---

### 12. Token Efficiency & Cost Analysis

Evaluate token optimization:

#### 12a. Language Cost
- Verify all content written in lowest-cost language (English).

#### 12b. Conciseness Enforcement
- Check if concise output rules exist and instructions themselves are concise.

#### 12c. Format-Fit Analysis
- Check if YAML headers, prose vs lists, and markdown tables used optimally.

#### 12d. Structural Density
- Verify content-to-overhead ratios and check for emoji usage instead of verbose text markers.

#### 12e. Redundancy Detection
- Scan for repeated information (e.g. build commands duplicated between files).

#### 12f. Vagueness as Token Waste
- Flag vague terms ("use appropriate tooling") that increase response lengths.

---

### 13. Scope & Architecture Fit

Verify alignment with physical project details:

#### 13a. Template vs. Instance
- Identify if structure is project-specific or generic template.

#### 13b. Agent Model
- Check if agent roles actionable or decorative.

#### 13c. Context Budget
- Check for mechanisms limiting `.agents/` directory context growth.

#### 13d. Ignore Coverage
- Verify `.agentignore` correctly excludes build and package folders.

---

### 14. Git Exposure Check

Verify repository exposure risk:

#### 14a. Check Exclusions
- Check if `.agents/` in `.gitignore`.

#### 14b. Check Entry Point
- Check if `AGENTS.md` in `.gitignore`.

#### 14c. Classify Finding
- Classify as: **🟢 Excluded**, **🟡 Tracked (Intentional)**, or **🟠 Tracked (Review Recommended)**.

---

### 15. Enrich and Finalize Report

Generate structured report and save as artifact (`audit_report.md`) in artifact directory. Follow format spec below exactly. Do NOT write full report directly in chat. Write concise summary in chat and link to generated artifact.

**Sequential Generation Process**:
1. **Automated Base**: Step 2 script creates baseline structure, file inventory table, style summary, coupling data, overengineering metrics, and lists gaps.
2. **Manual Enrichment**: Steps 3–14 gather manual details (pros/cons, scorecard notes, scores, blockers), then edit/update generated artifact.
3. **Save**: Do NOT run helper script after manual updates — will overwrite and wipe manual findings.

#### 15a. File Inventory Table
Columns: `File | Size | Purpose | Status`
- **Status** uses emoji: ✅ Instantiated · ⚠️ Partial · 🔴 Hollow · 🔴 **Critical** (hollow + depended upon)
- Append summary line: `X Instantiated · Y Partial · Z Hollow/Critical`

#### 15b. Pros & Cons Table
Columns: `Category | Strengths | Weaknesses`
- Categories match audit steps: Architecture, Skills, Prompt quality, Token efficiency, Integrity, Style consistency, Coupling & cohesion, Overengineering, Git exposure.
- Keep each cell to 1–2 sentences max.

#### 15c. Critical Gaps
- Number each gap as **GAP-N**.
- Prefix heading with severity emoji: 🔴 Critical · ⚠️ Medium · ℹ️ Low.
- Each gap must contain exactly three fields: **Severity**, **Problem**, **Fix**.
- Order: Critical first, then Medium, then Low.

#### 15d. Token Efficiency Scorecard
Columns: `File | Lang Cost | Format Fit | Redundancy | Score`
- **Lang Cost**: ✅ English · 🔴 Non-English
- **Format Fit**: ✅ Justified · ⚠️ Improvable
- **Redundancy**: ✅ None · ⚠️ Minor overlap
- **Score**: letter grade — **A** (no issues), **B** (minor), **C** (moderate), **D** (blocking)
- Add notes row for any non-obvious score rationale.

#### 15e. Style Consistency Summary
Columns: `Pattern | Majority convention | # Violations | Example violator`
- Patterns: Header naming, Section numbering, Frontmatter presence, Link syntax, Bullet style, Emphasis usage.
- Keep violations count as integer; list at most one example violator per row.

#### 15f. Coupling & Cohesion Summary
Columns: `File/Skill | Fan-out (refs out) | Fan-in (refs in) | Verdict`
- **Fan-out**: number of `.agents/` files this file links to.
- **Fan-in**: number of `.agents/` files that link to this file.
- **Verdict**: ✅ Healthy · ⚠️ High fan-out · 🔴 Circular
- Append cohesion note row for any file with domain leakage.

#### 15g. Overengineering Scorecard
Columns: `Metric | Value | Threshold | Status`
- Metrics: Total files, Skills, Dead files (zero refs), Max hop depth.
- **Status**: ✅ Proportionate · ⚠️ Premature · 🔴 Bloated
- Final row: overall overengineering verdict label.

#### 15h. Summary Verdict Table
Columns: `Dimension | Score`
- Rate each dimension out of 10: Cross-reference integrity, Placeholder density, Rule consistency, Prompt engineering quality, Token efficiency, Style consistency, Coupling & cohesion, Overengineering, Scope & architecture fit.
- Final row: `**Overall** | **X.X / 10**` (arithmetic mean, one decimal).
- Follow with plain-text verdict: top 3 blockers before first use, and recommended next milestone.
