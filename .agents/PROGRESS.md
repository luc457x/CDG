# Progress (PROGRESS.md)

## Status

- State: Documentation synchronized with current codebase.
- Last: Updated doc/*.md and README.md for new backtesting, annualization, orderbook metrics, and CLI flags.

## Log

Old sessions: [PROGRESS_ARCHIVE.md](./PROGRESS_ARCHIVE.md).

### Session 35: Sync Documentation to Codebase

- Date: 2026-07-05
- Agent: Antigravity
- Goal: Update documentation across doc/*.md and README.md to match current codebase state after backtesting and analysis expansion.
- Constraints: None.
- Done:
  - Rebuilt [doc/api_cache.md](file:///c:/Users/lucas/Code/CDG/doc/api_cache.md) with accurate CoinGecko endpoints (tickers, market_chart, market_chart/range, simple/price, global, companies/public_treasury, global/decentralized_finance_defi), fixed stale rate-limit delay claim (no fixed cache-miss delay; retries start at 10s), and added orderbook metrics section.
  - Extended [doc/installation_usage.md](file:///c:/Users/lucas/Code/CDG/doc/installation_usage.md) with `backtest` subcommand, added `run-pipeline` flags (`--concurrency`, `--annualization-factor`, `--backtest`, `--strategy`, `--fee`, `--slippage`, `--rebalance-frequency`), and fixed `check-coin` positional argument note.
  - Updated [doc/analysis_optimization.md](file:///c:/Users/lucas/Code/CDG/doc/analysis_optimization.md) to document annualization factor defaults: `252.0` when `--drop-weekends` is active, `365.0` otherwise.
  - Expanded [doc/custom_strategies.md](file:///c:/Users/lucas/Code/CDG/doc/custom_strategies.md) with built-in strategies table (`rsi`, `macd`, `bollinger`, `all`), backtest execution details (transaction fees, slippage, rebalancing frequencies, US Treasury benchmark, CSV/JSON reports, equity curve plots).
  - Updated [doc/architecture.md](file:///c:/Users/lucas/Code/CDG/doc/architecture.md) Mermaid diagram to include orderbook metrics, ML prep, optimization, backtesting, and export layers. Expanded Core Components with current modules (`pipeline`, `backtest`, `ui`, `export`, `utils`).
  - Fixed typo in [doc/deployment.md](file:///c:/Users/lucas/Code/CDG/doc/deployment.md) ("Data Lakeing" -> "Data Lake") and added `backtests/` directory listing plus standalone `backtest_run_` directory.
  - Updated [README.md](file:///c:/Users/lucas/Code/CDG/README.md) and [doc/README.md](file:///c:/Users/lucas/Code/CDG/doc/README.md) with missing custom-strategies doc links, new features (orderbook metrics, advanced indicators, strategy backtesting), `backtest` subcommand example, expanded CLI flags table, and updated output directory tree with `backtests/`.
- Blocked: None.
- Risk: None.
- Artifact: [doc/api_cache.md](file:///c:/Users/lucas/Code/CDG/doc/api_cache.md), [doc/installation_usage.md](file:///c:/Users/lucas/Code/CDG/doc/installation_usage.md), [doc/analysis_optimization.md](file:///c:/Users/lucas/Code/CDG/doc/analysis_optimization.md), [doc/custom_strategies.md](file:///c:/Users/lucas/Code/CDG/doc/custom_strategies.md), [doc/architecture.md](file:///c:/Users/lucas/Code/CDG/doc/architecture.md), [doc/deployment.md](file:///c:/Users/lucas/Code/CDG/doc/deployment.md), [README.md](file:///c:/Users/lucas/Code/CDG/README.md), [doc/README.md](file:///c:/Users/lucas/Code/CDG/doc/README.md).
- Verification: Manually inspected updated documentation files and verified cross-links and relative paths resolve correctly.
- Pending: None.

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
