# Interface Design

When user wants to explore alternative interfaces for chosen deepening candidate, use parallel sub-agent pattern. Based on "Design It Twice" (Ousterhout) — first idea rarely best.

Specialization of [design_an_interface](../design_an_interface/SKILL.md) for architecture deepening. Same base pattern (parallel sub-agents, radical difference, compare and synthesize), extended with seam/adapter/leverage vocabulary.

Uses vocabulary in [LANGUAGE.md](./LANGUAGE.md) — **module**, **interface**, **seam**, **adapter**, **leverage**.

## Process

### 1. Frame the Problem Space

Before spawning sub-agents, write user-facing explanation of problem space for chosen candidate:

- Constraints any new interface would need to satisfy
- Dependencies it would rely on, and which category they fall into (see [DEEPENING.md](./DEEPENING.md))
- Rough illustrative code sketch to ground constraints — not proposal, just make constraints concrete

Show to user, then immediately proceed to Step 2. User reads and thinks while sub-agents work in parallel.

### 2. Spawn Sub-Agents

Spawn 3+ sub-agents in parallel using Agent tool. Each must produce **radically different** interface for deepened module.

Prompt each sub-agent with separate technical brief (file paths, coupling details, dependency category from [DEEPENING.md](./DEEPENING.md), what sits behind seam). Brief independent of user-facing problem-space explanation in Step 1. Give each agent different design constraint:

- Agent 1: "Minimize interface — aim for 1–3 entry points max. Maximise leverage per entry point."
- Agent 2: "Maximise flexibility — support many use cases and extension."
- Agent 3: "Optimise for most common caller — make default case trivial."
- Agent 4 (if applicable): "Design around ports & adapters for cross-seam dependencies."

Include both [LANGUAGE.md](./LANGUAGE.md) vocabulary and SPEC.md vocabulary in brief so each sub-agent names things consistently with architecture language and project's domain language.

Each sub-agent outputs:

1. Interface (types, methods, params — plus invariants, ordering, error modes)
2. Usage example showing how callers use it
3. What implementation hides behind seam
4. Dependency strategy and adapters (see [DEEPENING.md](./DEEPENING.md))
5. Trade-offs — where leverage high, where thin

### 3. Present and Compare

Present designs sequentially so user can absorb each one, then compare in prose. Contrast by **depth** (leverage at interface), **locality** (where change concentrates), and **seam placement**.

After comparing, give recommendation: which design strongest and why. If elements from different designs would combine well, propose hybrid. Be opinionated — user wants strong read, not menu.
