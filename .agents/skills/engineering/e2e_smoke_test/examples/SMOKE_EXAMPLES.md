# Smoke Test Examples (Multi-Platform)

## Web / Hybrid Mode (Browser Subagent)

Use when application accessible via URL.

- **Task Template**: "Navigate to `http://localhost:PORT`. Perform [Action]. Verify that [Result] is visible."
- **Verification**: Check for DOM elements, console errors, and responsiveness.

## API Mode (Terminal)

Use for backends or headless services.

- **JavaScript / TypeScript**:
  - Command: `npx jest src/__tests__/integration` or `npm test`.
  - Manual: `curl -X GET http://localhost:PORT/api/health`.
- **Python**:
  - Command: `pytest` or `python -m unittest discover`.
  - Verification: `pytest --tb=short` or `python -m pytest`.
- **Rust**:
  - Command: `cargo test`
  - Verification: `cargo test -- --nocapture` to inspect output.

## CLI Mode (Terminal)

Use for command-line interfaces.

- Command: `./your-tool --help` or `node index.js --version`.
- Verification: Check stdout contains specific keywords or exit code is 0.

## Native Mode (Terminal)

Use for native mobile/desktop apps.

- Command: `npx detox test -c ios.sim.debug` or `xcodebuild test ...`.
- Verification: Analyze test runner output for successful assertions.
