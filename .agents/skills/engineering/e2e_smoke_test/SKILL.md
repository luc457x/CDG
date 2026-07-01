---
name: e2e_smoke_test
description: End-to-end verification and happy path validation protocol. Use when running e2e tests, verifying happy paths, or executing smoke tests.
---

# E2E Smoke Test Skill

## When to Use

After implementing feature, logic, or interface affecting application's critical user flow.

## Goal

Verify "Happy Path" (default/standard execution flow with no error conditions, as defined in `SPEC.md` or task description) is functional using most appropriate tool for current tech stack. Do not guess happy path — if not explicitly documented, check `SPEC.md` first or clarify with user.

## Steps

1. **Select mode** based on platform/language:
   - **Web/Hybrid:** Browser subagent → "Navigate to URL. Perform [Action]. Verify [Result]."
   - **API / Backend:**
     - **JS/TS:** `npx jest` or `curl -X GET http://localhost:PORT/api/health`
     - **Python:** `pytest` or `python -m unittest`
     - **Rust:** `cargo test`
   - **Native:** `npx detox test` or equivalent test runner.

2. **Ensure** app/service running and testable.
3. **Execute** verification, check for errors, timeouts, or logic violations.
4. **Compare** output against expected happy path results in `SPEC.md` or task description.
5. **Document** result: "Smoke Test Passed ([Mode])" in task validation.
