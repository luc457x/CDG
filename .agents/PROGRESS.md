# Progress (PROGRESS.md)

## Status

- State: Settings sub-menu added to interactive CLI mode.
- Last: Replaced Configure Cache TTL with Settings menu and moved warning print inside it.

## Log

Old sessions: [PROGRESS_ARCHIVE.md](./PROGRESS_ARCHIVE.md).

### Session 34: Add Settings Submenu to Interactive UI

- Date: 2026-07-04
- Agent: Antigravity
- Goal: Replace the "Configure Cache TTL" main menu option with a "Settings" sub-menu in interactive mode, displaying a warning message when opened.
- Constraints: None.
- Done:
  - Replaced the "Configure Cache TTL" option with a "Settings" option in the main interactive menu of [ui.rs](file:///c:/Users/lucas/Code/CDG/src/ui.rs#L46-L55).
  - Implemented the "Settings" sub-menu in [ui.rs](file:///c:/Users/lucas/Code/CDG/src/ui.rs#L364-L403) that displays a warning print and prompts with "Configure Cache TTL" and "Back" settings choices.
  - Removed the env file warning print from the "Run Portfolio Pipeline" menu selection in [ui.rs](file:///c:/Users/lucas/Code/CDG/src/ui.rs#L77) and moved it to the "Settings" menu option.
  - Adjusted the outer menu loop post-match check in [ui.rs](file:///c:/Users/lucas/Code/CDG/src/ui.rs#L405-L407) to skip `wait_for_back()` if the choice was `"Settings"`.
  - Added functional requirement `FR23` to [SPEC.md](file:///c:/Users/lucas/Code/CDG/.agents/SPEC.md#L66-L67).
  - Updated documentation files [installation_usage.md](file:///c:/Users/lucas/Code/CDG/doc/installation_usage.md#L55) and [api_cache.md](file:///c:/Users/lucas/Code/CDG/doc/api_cache.md#L67) to reference the new settings submenu path.
- Blocked: None.
- Risk: None.
- Artifact: [ui.rs](file:///c:/Users/lucas/Code/CDG/src/ui.rs), [SPEC.md](file:///c:/Users/lucas/Code/CDG/.agents/SPEC.md), [installation_usage.md](file:///c:/Users/lucas/Code/CDG/doc/installation_usage.md), [api_cache.md](file:///c:/Users/lucas/Code/CDG/doc/api_cache.md).
- Verification: Compiled successfully using `cargo check`.
- Pending: None.

### Session 33: Modularization and Polars Optimization

- Date: 2026-07-04
- Agent: Antigravity
- Goal: Clean up CLI monolith, optimize DataFrame column insertions, and simulate portfolio rebalancing transaction costs.
- Constraints: None.
- Done:
  - Created [pipeline.rs](file:///c:/Users/lucas/Code/CDG/src/pipeline.rs) and [ui.rs](file:///c:/Users/lucas/Code/CDG/src/ui.rs) to modularize CLI logic from [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs).
  - Registered new modules in [lib.rs](file:///c:/Users/lucas/Code/CDG/src/lib.rs) and cleaned up [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs).
  - Optimized [analysis.rs](file:///c:/Users/lucas/Code/CDG/src/analysis.rs) by batch-inserting calculated indicators using `hstack` instead of iterating `.insert_column` calls.
  - Added transaction fee and slippage math to portfolio daily rebalancing in [backtest.rs](file:///c:/Users/lucas/Code/CDG/src/backtest.rs) and [pipeline.rs](file:///c:/Users/lucas/Code/CDG/src/pipeline.rs).
  - Added calendar-based rebalancing frequency options (daily, weekly, monthly) configurable via `--rebalance-frequency` CLI flag and `CDG_REBALANCE_FREQUENCY` env var.
  - Modeled weight drift on non-rebalancing days in the portfolio simulation.
  - Added interactive prompt selections for rebalancing frequency.
  - Created ADR 001 to document choices on native Polars expressions for indicators.
  - Created [env.example](file:///c:/Users/lucas/Code/CDG/.env.example) template file.
  - Added warning in [ui.rs](file:///c:/Users/lucas/Code/CDG/src/ui.rs) notifying user to edit `.env` for permanent defaults.
- Blocked: None.
- Risk: None.
- Artifacts: [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs), [lib.rs](file:///c:/Users/lucas/Code/CDG/src/lib.rs), [pipeline.rs](file:///c:/Users/lucas/Code/CDG/src/pipeline.rs), [ui.rs](file:///c:/Users/lucas/Code/CDG/src/ui.rs), [analysis.rs](file:///c:/Users/lucas/Code/CDG/src/analysis.rs), [backtest.rs](file:///c:/Users/lucas/Code/CDG/src/backtest.rs), [Cargo.toml](file:///c:/Users/lucas/Code/CDG/Cargo.toml), [env.example](file:///c:/Users/lucas/Code/CDG/.env.example).
- Verification: `cargo test` - 50 tests passed.
- Pending: None.
