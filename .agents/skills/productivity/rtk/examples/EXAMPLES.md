# RTK Usage Examples

## Optimization Wrapped Commands

Prefix commands with `rtk` to filter and minimize token usage.

### 1. Git Status

```bash
rtk git status
```

### 2. Git Diff

```bash
rtk git diff
```

### 3. Git Log

```bash
rtk git log -n 50
```

### 4. Running Tests

```bash
rtk cargo test
```

### 5. Building Project

```bash
rtk npm run build
```

### 6. Wrapping PowerShell Cmdlets

PowerShell cmdlets (`Get-Location`, `Get-ChildItem`) cannot be wrapped directly. Use proxy wrapper:

```powershell
rtk proxy powershell -Command "Get-Location"
```

## Tracking Savings

```bash
rtk gain
```
