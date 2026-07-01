# Local CLI — Commit Examples

Use when `github-mcp-server` **not** available.

## 1. Atomic Commit (Single Block)

Stage and commit one atomic block:

```bash
git add src/auth.js src/auth.test.js
git commit -m "feat: add user authentication handler"
```

## 2. Feature Branch Setup

Create and push isolation branch before starting new task:

```bash
git checkout -b feat/oauth-support
git push -u origin feat/oauth-support
```

## 3. Push Changes

```bash
git push
```

## 4. Session Tagging

On final commit of session, tag it and push:

```bash
git tag session-03
git push --tags
```

## 5. Verify Last Commits

```bash
git log --oneline -3
```
