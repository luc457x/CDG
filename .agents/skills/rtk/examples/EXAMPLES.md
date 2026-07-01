# RTK Usage Examples

## Optimization Wrapped Commands

Prefix commands with `rtk` to filter and minimize token usage.

### 1. Git Status

```bash
rtk git status
```

### 2. Running Tests

```bash
rtk cargo test
```

### 3. Building Project

```bash
rtk npm run build
```

### 4. Wrapping PowerShell Cmdlets

PowerShell cmdlets (`Get-Location`, `Get-ChildItem`) cannot be wrapped directly. Use proxy wrapper:

```powershell
rtk proxy powershell -Command "Get-Location"
```

### 5. Tracking Savings

```bash
rtk gain
```
