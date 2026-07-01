---
trigger: always_on
---

# Harness Rules

## Dependencies

- **Rust**: Serialization: `serde`, Async: `tokio`, HTTP: `reqwest`, DB: `sqlx`, Dataframes: `polars`, Plotting: `plotters`, CLI: `clap`, Env: `dotenvy`

## Commands

### Rust

- Build/Dev: `cargo build` | `cargo run`
- Test: `cargo test`
- Lint: `cargo clippy`

### Token Optimization (rtk)

- Wrapper: Prefix terminal commands (`git status/diff/log`, `cargo test`, `cargo build`, etc.) with `rtk`.

## QA Protocol

1. **Build**: Project compiles without error (`cargo build`)
2. **Lint/Format**: Styles match rustfmt and clippy runs green (`cargo clippy`, `cargo fmt`)
3. **Clean**: No debug logs (`dbg!`, `println!`), draft comments, temp files
4. **Tests**: Run suite. 0 failures (`cargo test`)
5. **Smoke**: Run manual verification on generated outputs
