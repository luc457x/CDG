# Progress (PROGRESS.md)

## Status

- State: Modular project documentation system created under doc/ directory. Root README linked.
- Last: Generated doc/ README, architecture, installation, api/caching, indicators, and deployment guides.

## Log

Old sessions: [PROGRESS_ARCHIVE.md](./PROGRESS_ARCHIVE.md).

### Session 14: Generate Project Documentation

- Date: 2026-07-01
- Agent: Antigravity
- Goal: Create modular, comprehensive documentation system in doc/ directory.
- Constraints: None.
- Done:
  - Created [README.md](file:///c:/Users/lucas/Code/CDG/doc/README.md) hub to organize documentation system.
  - Created [architecture.md](file:///c:/Users/lucas/Code/CDG/doc/architecture.md) detailing ingestion/processing flows and Mermaid diagrams.
  - Created [installation_usage.md](file:///c:/Users/lucas/Code/CDG/doc/installation_usage.md) detailing CLI commands and interactive pager.
  - Created [api_cache.md](file:///c:/Users/lucas/Code/CDG/doc/api_cache.md) detailing CoinGecko retry mechanisms and SQLite caching logic.
  - Created [analysis_optimization.md](file:///c:/Users/lucas/Code/CDG/doc/analysis_optimization.md) documenting technical indicators formulas and Monte Carlo simulation.
  - Created [deployment.md](file:///c:/Users/lucas/Code/CDG/doc/deployment.md) covering output folder structure and containerization / GCP Cloud Run integration.
  - Modified root [README.md](file:///c:/Users/lucas/Code/CDG/README.md) to link to the new documentation files.
- Blocked: None.
- Risk: None.
- Artifact: `doc/` directory files: `doc/README.md`, `doc/architecture.md`, `doc/installation_usage.md`, `doc/api_cache.md`, `doc/analysis_optimization.md`, `doc/deployment.md`.
- Verification: Passed `cargo test` successfully (all 27 tests passed).
- Pending: None.

### Session 13: AI Engineering Infrastructure Sync

- Date: 2026-07-01
- Agent: Antigravity
- Goal: Mirror CDGonGCP's AI engineering setup (skills, rules, docs) into CDG workspace.
- Constraints: Preserve CDG project-specific content (SPEC.md, PROGRESS.md, BACKLOG.md, ADRs).
- Done:
  - Added 3 new rules to `.agents/rules/`: `harness.md`, `structure.md`, `workflow.md`.
  - Updated `rules/engineering.md`: added Rule 7 (Relative Paths), fixed blocker link to `rules/workflow.md`.
  - Updated `rules/safety.md`: removed stale TASKS.md reference.
  - Updated `rules/load.md`: replaced root file links with rules/ equivalents; removed TASKS refs.
  - Updated `rules/structure.md`: reflects actual `.agents/` root; added `etc/` dir; removed stale entries.
  - Added 4 new engineering skills: `archive_progress`, `clean_architecture`, `finish_session`, `spec_triage`.
  - Added 1 new productivity skill: `project_documentation`.
  - Deleted obsolete `.agents/` root files: `HARNESS.md`, `STRUCTURE.md`, `WORKFLOW.md`, `TASKS.md`, `TASKS_ARCHIVE.md`.
  - Created `.agents/etc/` dir; copied `cdg-lib-migration-candidates.md` from CDGonGCP.
  - Aligned `.agentignore` with CDGonGCP format (normalized db glob patterns, fixed DS_Store casing).
  - Aligned `.gitignore` with CDGonGCP (added `.kilo/.gemini/.claude`; normalized db patterns; restored Python runtime artifacts for skill scripts; removed packaging bloat).
- Blocked: None.
- Risk: None — infrastructure only, no source code touched.
- Artifact: `.agents/rules/`, `.agents/skills/engineering/`, `.agents/skills/productivity/`, `.agents/etc/cdg-lib-migration-candidates.md`.
- Verification: Manual — all rule files reference valid paths; all new skill SKILL.md files present.
- Pending: CDG lib migration work (see `.agents/etc/cdg-lib-migration-candidates.md`).

### Session 12: Interactive CLI and Raw OHLCV Exporter Enhancements

- Date: June 13, 2026
- Agent: Antigravity
- Done: Added raw OHLCV folder output, improved CLI pager UX with terminal clearing and a [Back] button, and updated coin listing to fetch/display top 50 coins by market cap.
- Actions:
  - main.rs:
    - Implemented clear_terminal and wait_for_back helpers.
    - Cleared terminal before displaying the interactive menu and executing actions.
    - Appended [Back] button logic at the end of non-Exit options.
    - Saved fetched raw OHLCV data to `cdg_files/can_YYYYMMDD_HHMMSS` as JSON and CSV in run_pipeline_flow and run_ohlcv_flow.
    - Modified list-coins subcommand and interactive "List Supported Coins" menu action to query the `/coins/markets` endpoint (sorted by market cap desc) instead of `/coins/list`.
  - Spec: Updated [SPEC.md](file:///c:/Users/lucas/Code/CDG/.agents/SPEC.md) with functional requirements FR13, FR16 and FR17.
- Verification:
  - Run `cargo fmt -- --check`: 100% clean.
  - Run `cargo clippy -- -D warnings`: 100% clean.
  - Run `cargo test`: Passed all 27 tests.
  - Manual verification: Verified list-coins prints top 50 by market cap with price and market cap columns.
- Pending: None.
