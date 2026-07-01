---
name: diagnose
description: Disciplined diagnosis loop for hard bugs and performance regressions. Reproduce → minimise → hypothesise → instrument → fix → regression-test
when_to_use: user says "diagnose this" / "debug this", reports bug, says something broken/throwing/failing, or describes performance regression.
metadata:
  category: development
---
# Diagnose

## When to Use

Discipline for hard bugs. Use when user reports a bug, says something is broken/throwing/failing, or describes a performance regression.

When exploring codebase, use project's domain glossary for clear mental model of relevant modules, check ADRs in area being touched.

## Process

### Phase 1 — Build a Feedback Loop

**This is the skill.** Everything else mechanical. Fast, deterministic, agent-runnable pass/fail signal = will find cause. No loop = no amount of staring at code will save you.

Spend disproportionate effort here. **Be aggressive. Be creative. Refuse to give up.**

### Ways to Construct — try in order

1. **Failing test** at whatever seam reaches bug — unit, integration, e2e.
2. **Curl / HTTP script** against running dev server.
3. **CLI invocation** with fixture input, diff stdout against known-good snapshot.
4. **Headless browser script** (Playwright / Puppeteer) — drives UI, asserts on DOM/console/network.
5. **Replay captured trace.** Save real network request / payload / event log to disk; replay through code path in isolation.
6. **Throwaway harness.** Spin up minimal subset of system (one service, mocked deps) exercising bug code path with single function call.
7. **Property / fuzz loop.** If bug = "sometimes wrong output", run 1000 random inputs and look for failure mode.
8. **Bisection harness.** If bug appeared between two known states (commit, dataset, version), automate "boot at state X, check, repeat" so can `git bisect run` it.
9. **Differential loop.** Run same input through old-version vs new-version (or two configs) and diff outputs.
10. **HITL bash script.** Last resort. If human must click, drive _them_ with `scripts/hitl_loop.template.sh` so loop still structured. Captured output feeds back to you.

Build right feedback loop → bug 90% fixed.

### Iterate on Loop Itself

Treat loop as product. Once loop exists, ask:

- Can I make it faster? (Cache setup, skip unrelated init, narrow test scope.)
- Can I sharpen signal? (Assert on specific symptom, not "didn't crash".)
- Can I make it deterministic? (Pin time, seed RNG, isolate filesystem, freeze network.)

30-second flaky loop barely better than no loop. 2-second deterministic loop = debugging superpower.

### Non-Deterministic Bugs

Goal not clean repro but **higher reproduction rate**. Loop trigger 100×, parallelise, add stress, narrow timing windows, inject sleeps. 50%-flake = debuggable. 1% = not — keep raising rate until debuggable.

### When Cannot Build Loop

Stop and say so explicitly. List what tried. Ask user for: (a) access to env that reproduces it, (b) captured artifact (HAR file, log dump, core dump, screen recording with timestamps), or (c) permission to add temporary production instrumentation. Do **not** proceed to hypothesise without loop.

Do not proceed to Phase 2 without loop.

### Phase 2 — Reproduce

Run loop. Watch bug appear.

Confirm:

- [ ] Loop produces failure mode **user** described — not different failure nearby. Wrong bug = wrong fix.
- [ ] Failure reproducible across multiple runs (or, for non-deterministic bugs, at high enough rate to debug against).
- [ ] Captured exact symptom (error message, wrong output, slow timing) so later phases can verify fix addresses it.

Do not proceed until bug reproduced.

### Phase 3 — Hypothesise

Generate **3–5 ranked hypotheses** before testing any. Single-hypothesis generation anchors on first plausible idea.

Each hypothesis must be **falsifiable**: state prediction it makes.

> Format: "If \<X\> is cause, then \<changing Y\> will make bug disappear / \<changing Z\> will make it worse."

Cannot state prediction → hypothesis is vibe — discard or sharpen.

**Show ranked list to user before testing.** Often have domain knowledge that re-ranks instantly ("we just deployed change to #3"), or know hypotheses already ruled out. Cheap checkpoint, big time saver. Don't block on it — proceed with ranking if user is AFK.

### Phase 4 — Instrument

Each probe must map to specific prediction from Phase 3. **Change one variable at a time.**

Tool preference:

1. **Debugger / REPL inspection** if env supports it. One breakpoint beats ten logs.
2. **Targeted logs** at boundaries that distinguish hypotheses.
3. Never "log everything and grep".

**Tag every debug log** with unique prefix, e.g. `[DEBUG-a4f2]`. Cleanup = single grep. Untagged logs survive; tagged logs die.

**Perf branch.** For performance regressions, logs usually wrong. Instead: establish baseline measurement (timing harness, `performance.now()`, profiler, query plan), then bisect. Measure first, fix second.

### Phase 5 — Fix + Regression Test

Write regression test **before fix** — but only if **correct seam** exists.

Correct seam = test exercises **real bug pattern** as it occurs at call site. If only available seam too shallow (single-caller test when bug needs multiple callers, unit test that can't replicate chain that triggered bug), regression test there gives false confidence.

**No correct seam = that itself is finding.** Note it. Codebase architecture preventing bug from being locked down. Flag for next phase.

If correct seam exists:

1. Turn minimised repro into failing test at that seam.
2. Watch it fail.
3. Apply fix.
4. Watch it pass.
5. Re-run Phase 1 feedback loop against original (un-minimised) scenario.

### Phase 6 — Cleanup + Post-Mortem

Required before declaring done:

- [ ] Original repro no longer reproduces (re-run Phase 1 loop)
- [ ] Regression test passes (or absence of seam documented)
- [ ] All `{DEBUG-...}` instrumentation removed (`grep` prefix)
- [ ] Throwaway prototypes deleted (or moved to clearly-marked debug location)
- [ ] Hypothesis that turned out correct stated in commit / PR message — next debugger learns

**Then ask: what would have prevented this bug?** If answer involves architectural change (no good test seam, tangled callers, hidden coupling), flag for architectural improvement. Make recommendation **after** fix is in, not before.
