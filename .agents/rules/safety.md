---
trigger: always_on
---

# Safety Rules

1. **Checkpoints**: Checkpoint after big steps. Log done, verified, left. If state unclear, trigger Rollback & Restart (Rule 4).
2. **Fail Loud**: Never skip requirements/tests. Surface failures/uncertainty now. Skipping = incomplete task.
3. **Code Convention**: Match codebase style. Flag harmful/insecure patterns before run.
4. **Rollback & Restart**: If confused or state unclear, stop. Do the following:
   - **Document Failure**: Log attempted actions, reason for confusion, and critical traps to avoid in [PROGRESS.md](../PROGRESS.md).
   - **Rollback**: Discard confusing/unverified changes.
   - **Reset & Restart**: Re-read [SPEC.md](../SPEC.md) and failure notes. Start again from last safe verified milestone.