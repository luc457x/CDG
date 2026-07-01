# RTK Troubleshooting Guide

## 1. Git Bash Utilities Path Resolution

If Git Bash inside IDE terminal fails to run wrapped commands like `rtk pwd` or `rtk ls`, internal path resolver cannot find Git Bash's utility folder.

### Solution

Agent can programmatically append Git Bash utility folder (`C:\Program Files\Git\usr\bin`) to persistent User environment PATH, or user can update manually.

1. Ask user: *"Would you like me to automatically update your User environment PATH with Git Bash utilities folder, or perform steps manually?"*

2. **If automatic**:
   - Run script [add_git_bash_to_path.ps1](../scripts/add_git_bash_to_path.ps1) via PowerShell to update registry path and active session PATH.
   - If running in Git Bash: `export PATH="/c/Program Files/Git\usr\bin:$PATH"` to update active session immediately.
   - Inform user active session PATH updated. Restarting IDE recommended later to persist across all new sessions.

3. **If manual**:
   1. Press **Windows Key**, type `Environment Variables`, press **Enter**.
   2. Click **Environment Variables...** (bottom right).
   3. Under **User variables**, select **Path**, click **Edit**.
   4. Click **New** and paste: `C:\Program Files\Git\usr\bin`
   5. Click **OK** on all windows, close and reopen IDE.

## 2. Command Not Found / Unrecognized Command

If `rtk` not recognized by shell:

1. **Check Local Path**: Check if compiled at `%USERPROFILE%\.cargo\bin\rtk.exe` (Windows) or `~/.cargo/bin/rtk` (Linux).
2. **Add to Path**: If present, update active session PATH and retry:
   - PowerShell: `$env:Path = "$env:USERPROFILE\.cargo\bin;" + $env:Path`
   - Linux: `export PATH="$HOME/.cargo/bin:$PATH"`
3. **Install RTK**: If missing entirely, read [Installation Guide](./INSTALLATION.md).