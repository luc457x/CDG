---
name: design-an-interface
description: Generate multiple radically different interface designs for module using parallel sub-agents
when_to_use: user wants to design API, explore interface options, compare module shapes, or mentions "design it twice".
metadata:
  category: development
---
# Design an Interface

Based on "Design It Twice" from "A Philosophy of Software Design": first idea rarely best. Generate multiple radically different designs, then compare.

## Workflow

### 1. Gather Requirements

Before designing, understand:

- [ ] What problem does this module solve?
- [ ] Who are callers? (other modules, external users, tests)
- [ ] What are key operations?
- [ ] Any constraints? (performance, compatibility, existing patterns)
- [ ] What should be hidden inside vs exposed?

Ask: "What does this module need to do? Who will use it?"

### 2. Generate Designs (Parallel Sub-Agents)

Spawn 3+ sub-agents simultaneously using Task tool. Each must produce **radically different** approach.

```md
Prompt template for each sub-agent:

Design an interface for: [module description]

Requirements: [gathered requirements]

Constraints for this design: [assign a different constraint to each agent]
- Agent 1: "Minimize method count - aim for 1-3 methods max"
- Agent 2: "Maximize flexibility - support many use cases"
- Agent 3: "Optimize for the most common case"
- Agent 4: "Take inspiration from [specific paradigm/library]"

Output format:
1. Interface signature (types/methods)
2. Usage example (how caller uses it)
3. What this design hides internally
4. Trade-offs of this approach
```

### 3. Present Designs

Show each design with:

1. Interface signature — types, methods, params
2. Usage examples — how callers actually use it
3. What it hides — complexity kept internal

Present designs sequentially so user can absorb each before comparison.

### 4. Compare Designs

After showing all, compare on:

- Interface simplicity: fewer methods, simpler params
- General-purpose vs specialized: flexibility vs focus
- Implementation efficiency: does shape allow efficient internals?
- Depth: small interface hiding significant complexity (good) vs large interface with thin implementation (bad)
- Ease of correct use vs ease of misuse

Discuss trade-offs in prose, not tables. Highlight where designs diverge most.

### 5. Synthesize

Best design often combines insights from multiple options. Ask:

- "Which design best fits your primary use case?"
- "Any elements from other designs worth incorporating?"

## Philosophy

From "A Philosophy of Software Design":

- **Interface simplicity**: Fewer methods, simpler params = easier to learn and use correctly.
- **General-purpose**: Handles future use cases without changes. Beware over-generalization.
- **Implementation efficiency**: Interface shape allow efficient implementation? Or force awkward internals?
- **Depth**: Small interface hiding significant complexity = deep module (good). Large interface with thin implementation = shallow module (avoid).

## Anti-Patterns

- Don't let sub-agents produce similar designs — enforce radical difference
- Don't skip comparison — value is in contrast
- Don't implement — purely about interface shape
- Don't evaluate based on implementation effort

## Related

Use **depth vocabulary** when discussing deepening: small interface hiding large implementation (good), large interface with thin implementation (avoid).
