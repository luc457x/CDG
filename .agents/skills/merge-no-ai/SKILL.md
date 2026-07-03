---
name: merge-no-ai
description: Merges dev branch to main. Keeps AI docs (.agents/, AGENTS.md, etc.) tracked only on dev. Ignored/untracked on main.
when_to_use: user requests to merge dev branch to main, push to main branch, or sync dev to main excluding AI documentation.
metadata:
  category: development
---

# Merge to Main Excluding AI Documentation

Merges development changes into `main` without tracking AI documentation (`.agents/`, `AGENTS.md`, `.agentignore`). Ensures `.gitignore` on `main` ignores these files. Files remain tracked on `dev`.

## Execution

Run Python helper script from workspace root:

```bash
python .agents/skills/merge-no-ai/scripts/merge_to_main.py
```

### Manual Steps Performed by Script

If running manually:

1. **Verify Clean Workspace**: Ensure no uncommitted changes on active branch.
2. **Switch to Main**: `git checkout main`
3. **Verify and Update `.gitignore`**:
   - Ensure `AGENTS.md`, `.agentignore`, `.agents/` uncommented in `.gitignore`.
   - If changed, commit update on `main`.
4. **Untrack AI Files**:
   - If AI docs tracked on `main`, run `git rm -r --cached .agents AGENTS.md .agentignore --ignore-unmatch`.
   - Commit: `git commit -m "chore: untrack AI documentation on main"`
5. **Merge Branch**:
   - Run `git merge <dev_branch> --no-commit --no-ff`
6. **Exclude AI Files from Merge Commit**:
   - Run `git rm -r --cached .agents AGENTS.md .agentignore --ignore-unmatch`
7. **Commit Merge**: `git commit -m "Merge branch '<dev_branch>' into main (excluding AI documentation)"`
8. **Restore Dev Branch**: `git checkout -f <dev_branch>`
