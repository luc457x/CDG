# Progress (PROGRESS.md)

## Status

- State: Configurable base output directory and configurable raw output format (JSON/CSV) fully implemented.
- Last: Added output_dir and raw_format options to Cli, PipelineConfig, flows, tests, and documentation.

## Log

Old sessions: [PROGRESS_ARCHIVE.md](./PROGRESS_ARCHIVE.md).

### Session 20: Output Savings Refinement

- Date: 2026-07-03
- Agent: Antigravity
- Goal: Implement configurable base output directory and configurable raw format via CLI/env, with nested raw OHLCV folder.
- Constraints: None.
- Done:
  - Added `--output-dir` parameter (default `"cdg_files"`, env `CDG_OUTPUT_DIR`) to `Cli` struct in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L9-L16).
  - Added `--raw-format` parameter (default `"json"`, env `CDG_RAW_FORMAT`) to `Cli` struct and validated it in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L114-L122).
  - Made `db_path` and `output_prefix` fields optional and dynamically resolved in `main` of [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L123-L130).
  - Passed `output_dir` and `raw_format` in `PipelineConfig` and formatted run/candlestick output directories dynamically in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L301-L335).
  - Nested raw OHLCV folder under `raw_ohlcv` inside the pipeline run directory and saved only in the configured format in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L329-L335).
  - Changed `run_ohlcv_flow` and `run_interactive_menu` signature and calls to pass `output_dir` and `raw_format` in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L759-L1139).
  - Added unit tests `test_dynamic_path_resolution` and `test_raw_format_validation` in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L1208-L1320).
  - Updated specifications in [SPEC.md](file:///c:/Users/lucas/Code/CDG/.agents/SPEC.md#L59-L64).
  - Updated user-facing documentation in [README.md](file:///c:/Users/lucas/Code/CDG/README.md#L107-L198), [installation_usage.md](file:///c:/Users/lucas/Code/CDG/doc/installation_usage.md#L114-L119), and [deployment.md](file:///c:/Users/lucas/Code/CDG/doc/deployment.md#L20-L36).
- Blocked: None.
- Risk: None.
- Artifact: [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs), [SPEC.md](file:///c:/Users/lucas/Code/CDG/.agents/SPEC.md), [README.md](file:///c:/Users/lucas/Code/CDG/README.md), [installation_usage.md](file:///c:/Users/lucas/Code/CDG/doc/installation_usage.md), [deployment.md](file:///c:/Users/lucas/Code/CDG/doc/deployment.md).
- Verification: `cargo test` (41 tests passed), and smoke test run verifying JSON and CSV exports.
- Pending: None.

### Session 19: Dynamic Concurrency Limit

- Date: 2026-07-03
- Agent: Antigravity
- Goal: Implement key-aware concurrency limit defaulting (1 for demo/free keys, 3 for pro keys), overridable by CLI flag or env var.
- Constraints: None.
- Done:
  - Modified `concurrency` in clap CLI parser and `PipelineConfig` to be `Option<usize>` in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L58-L60).
  - Implemented automatic CoinGecko API key support (Demo/Pro base URL and headers) in [coingecko.rs](file:///c:/Users/lucas/Code/CDG/src/api/coingecko.rs#L22-L41).
  - Implemented default concurrency resolution logic in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L290-L302) based on key presence.
  - Updated interactive CLI menu default concurrency to adapt to key presence in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L915-L930).
  - Added unit test `test_default_concurrency_resolution` in [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs#L1182-L1232).
- Blocked: None.
- Risk: None.
- Artifact: [main.rs](file:///c:/Users/lucas/Code/CDG/src/main.rs), [coingecko.rs](file:///c:/Users/lucas/Code/CDG/src/api/coingecko.rs).
- Verification: `cargo test` passed 39/39 tests. `cargo clippy` passed with zero errors/warnings.
- Pending: None.
