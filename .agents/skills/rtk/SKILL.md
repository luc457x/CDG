---
name: rtk
description: Utilizes 'rtk' tool for terminal output and token optimization
when_to_use: wrapping terminal commands, reducing token volume, or running commands via rtk.
metadata:
  category: utility
---
# RTK (Rust Token Killer) Skill

## When to Use

Wrap any high-volume terminal commands (git, build, test, file-listing) with `rtk` to save tokens.

## Steps

1. **Execute wrapped command**: Run target command prefixed with `rtk` (e.g. `rtk git status`).
2. **Handle failures**: If execution fails or command unrecognized, read [Troubleshooting Guide](./troubleshooting/git_bash_missing.md).

## Rules & Reference

- Wrap git (`status`, `diff`, `log`), build, test, and file listing commands.
- Usage examples: See [EXAMPLES.md](./examples/EXAMPLES.md).
- Troubleshooting: See [Troubleshooting Guide](./troubleshooting/git_bash_missing.md).
