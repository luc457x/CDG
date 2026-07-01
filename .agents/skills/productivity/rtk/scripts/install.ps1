# install.ps1 - Automated RTK installation script for Windows 11
# Part of the rtk workspace skill

$ErrorActionPreference = "Stop"

Write-Host "=== Starting RTK Installation for Windows 11 ===" -ForegroundColor Cyan

# 1. DEPENDENCY CHECK (RUST & CARGO)
Write-Host "[1/3] Checking dependencies (Rust & Cargo)..." -ForegroundColor Yellow

$cargoPath = Get-Command cargo -ErrorAction SilentlyContinue
$rustcPath = Get-Command rustc -ErrorAction SilentlyContinue

$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"

if (-not $cargoPath -or -not $rustcPath) {
    # Check if they exist in the default user profile path but just aren't in this session PATH
    if (Test-Path (Join-Path $cargoBin "cargo.exe")) {
        Write-Host "Found Cargo in $cargoBin. Appending to current session path..." -ForegroundColor Green
        $env:Path = "$cargoBin;$env:Path"
    } else {
        Write-Host "Rust and Cargo are not detected on the system." -ForegroundColor Yellow
        Write-Host "NOTE: Please ensure you have Visual Studio Build Tools with C++ workload installed." -ForegroundColor DarkYellow
        Write-Host "Downloading the official rustup-init.exe..." -ForegroundColor Yellow
        
        $url = "https://win.rustup.rs/x86_64"
        $dest = Join-Path $env:TEMP "rustup-init.exe"
        
        # Download silently
        [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.SecurityProtocolType]::Tls12
        Invoke-WebRequest -Uri $url -OutFile $dest -UseBasicParsing
        
        Write-Host "Executing rustup-init.exe silently..." -ForegroundColor Yellow
        $process = Start-Process -FilePath $dest -ArgumentList "-y" -Wait -NoNewWindow -PassThru
        if ($process.ExitCode -ne 0) {
            throw "rustup-init failed with exit code $($process.ExitCode)"
        }
        
        Write-Host "Rust installed successfully. Appending .cargo\bin to current session path..." -ForegroundColor Green
        $env:Path = "$cargoBin;$env:Path"
    }
} else {
    Write-Host "Rust & Cargo are already available on the system PATH." -ForegroundColor Green
}

# Double check cargo version
cargo --version

# 2. RTK INSTALLATION
Write-Host "[2/3] Installing RTK (Rust Token Killer)..." -ForegroundColor Yellow
Write-Host "Running: cargo install --git https://github.com/rtk-ai/rtk" -ForegroundColor Gray

# Compile and install
cargo install --git https://github.com/rtk-ai/rtk

# 3. PATH VALIDATION & VERIFICATION
Write-Host "[3/3] Validating RTK command and path..." -ForegroundColor Yellow

$rtkPath = Get-Command rtk -ErrorAction SilentlyContinue
if (-not $rtkPath) {
    # It might be in .cargo\bin but path isn't permanently set or hasn't refreshed
    if (Test-Path (Join-Path $cargoBin "rtk.exe")) {
        Write-Host "rtk.exe is present in $cargoBin but not in your environment PATH." -ForegroundColor Yellow
        $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
        if (($userPath -split ';') -notcontains $cargoBin) {
            Write-Host "Appending $cargoBin to persistent User Path..." -ForegroundColor Yellow
            [Environment]::SetEnvironmentVariable("Path", "$userPath;$cargoBin", "User")
            $env:Path = "$cargoBin;$env:Path"
            Write-Host "Successfully added to persistent User Path." -ForegroundColor Green
        } else {
            Write-Host "$cargoBin is already in the persistent User PATH. Please restart your terminal/IDE to refresh environment variables." -ForegroundColor Yellow
        }
    } else {
        throw "RTK executable was not found after installation."
    }
}

# Run version verification
$rtkVersion = & rtk --version
Write-Host "Verification Success: $rtkVersion" -ForegroundColor Green
Write-Host "=== RTK Installation Completed Successfully ===" -ForegroundColor Green
