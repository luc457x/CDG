#!/usr/bin/env bash
# install.sh - Automated RTK installation script for Linux hosts
# Part of the rtk workspace skill

set -euo pipefail

echo "=== Starting RTK Installation for Linux ==="

# 1. DEPENDENCY CHECK (RUST & CARGO)
echo "[1/3] Checking dependencies (Rust & Cargo)..."

if ! command -v cargo &> /dev/null || ! command -v rustc &> /dev/null; then
    # Try importing cargo env just in case it is installed but not active
    if [ -f "$HOME/.cargo/env" ]; then
        echo "Found existing Cargo env file. Sourcing it..."
        source "$HOME/.cargo/env"
    fi
fi

if ! command -v cargo &> /dev/null || ! command -v rustc &> /dev/null; then
    echo "Rust/Cargo not detected. Sourcing standard rustup installer..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    
    # Sourcing profile
    source "$HOME/.cargo/env"
else
    echo "Rust & Cargo are already available on the system PATH."
fi

# Print cargo version
cargo --version

# 2. RTK INSTALLATION
echo "[2/3] Installing RTK (Rust Token Killer)..."
INSTALL_SUCCESS=false

# Try compiling and installing from github
if command -v cargo &> /dev/null; then
    echo "Compiling and installing RTK from source repository..."
    if cargo install --git https://github.com/rtk-ai/rtk; then
        INSTALL_SUCCESS=true
    else
        echo "Warning: Cargo install failed. Trying fallback fast script..."
    fi
fi

if [ "$INSTALL_SUCCESS" = false ]; then
    echo "Using fast script fallback installer..."
    curl -fsSL https://raw.githubusercontent.com/rtk-ai/rtk/refs/heads/master/install.sh | sh
fi

# 3. PATH VALIDATION & VERIFICATION
echo "[3/3] Validating RTK command and path..."

if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

if ! command -v rtk &> /dev/null; then
    # Check if rtk binary is present in ~/.cargo/bin
    if [ -f "$HOME/.cargo/bin/rtk" ]; then
        echo "rtk binary is located at $HOME/.cargo/bin/rtk but it is not in your current PATH."
        echo "Please add the following line to your profile (~/.bashrc, ~/.zshrc, or ~/.profile):"
        echo 'export PATH="$HOME/.cargo/bin:$PATH"'
        
        # Try appending to current PATH
        export PATH="$HOME/.cargo/bin:$PATH"
    else
        echo "Error: rtk binary not found after installation steps."
        exit 1
    fi
fi

# Run validation
rtk_ver=$(rtk --version)
echo "Verification Success: $rtk_ver"
echo "=== RTK Installation Completed Successfully ==="
