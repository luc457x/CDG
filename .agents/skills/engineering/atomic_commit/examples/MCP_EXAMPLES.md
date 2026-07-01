# GitHub MCP Server — Commit Examples

Use when `github-mcp-server` available. No local Git CLI needed.
All tools called via `call_mcp_tool` interface with `ServerName: github-mcp-server`.

## 1. Atomic Commit (Multi-File Upload)

Use `push_files` to submit multiple file edits in single atomic commit:

- **ServerName**: `github-mcp-server`
- **ToolName**: `push_files`
- **Arguments**:

  ```json
  {
    "owner": "username",
    "repo": "repository-name",
    "branch": "main",
    "message": "feat: add user authentication handler",
    "files": [
      {
        "path": "src/auth.js",
        "content": "const login = () => { ... }"
      },
      {
        "path": "src/auth.test.js",
        "content": "describe('auth', () => { ... })"
      }
    ]
  }
  ```

## 2. Feature Branch Setup

Use `create_branch` to create isolation branches before starting new task:

- **ServerName**: `github-mcp-server`
- **ToolName**: `create_branch`
- **Arguments**:

  ```json
  {
    "owner": "username",
    "repo": "repository-name",
    "branch": "feat/oauth-support",
    "source_branch": "main"
  }
  ```

## 3. Open Pull Request

Submit completed work for review after all atomic commits on feature branch done:

- **ServerName**: `github-mcp-server`
- **ToolName**: `create_pull_request`
- **Arguments**:

  ```json
  {
    "owner": "username",
    "repo": "repository-name",
    "title": "feat: OAuth support",
    "body": "Implements OAuth 2.0 login flow. Closes #12.",
    "head": "feat/oauth-support",
    "base": "main"
  }
  ```

## 4. Session Tagging

Tag end of session by encoding `session-XX` in final `push_files` commit message and documenting in [PROGRESS.md](../../../../PROGRESS.md):

- **ServerName**: `github-mcp-server`
- **ToolName**: `push_files`
- **Arguments**:

  ```json
  {
    "owner": "username",
    "repo": "repository-name",
    "branch": "main",
    "message": "chore: session-03 — finalize auth module",
    "files": []
  }
  ```

## 5. Create Issue for Blocked Task

When task marked `[BLOCKED]`, open tracking issue to preserve context:

- **ServerName**: `github-mcp-server`
- **ToolName**: `create_issue`
- **Arguments**:

  ```json
  {
    "owner": "username",
    "repo": "repository-name",
    "title": "[BLOCKED] T-5: WebSocket reconnection fails under load",
    "body": "Blocked due to missing upstream fix in dependency X.\nSee PROGRESS.md Session 3 for details.",
    "labels": ["blocked", "bug"]
  }
  ```

## 6. Verify Last Push

Confirm last commit landed correctly:

- **ServerName**: `github-mcp-server`
- **ToolName**: `list_commits`
- **Arguments**:

  ```json
  {
    "owner": "username",
    "repo": "repository-name",
    "sha": "main",
    "per_page": 5
  }
  ```
