#!/bin/bash
set -e

# Default Repo URL (Can be overridden)
REPO_URL="${REPO_URL:-https://github.com/nebuff/aeroshell.git}"

echo "=========================================="
echo "   AeroShell Auto-Installer"
echo "=========================================="

# 1. Detect OS & Install Dependencies
install_deps() {
    echo "[*] Checking dependencies..."

    local missing_git=false
    if ! command -v git &> /dev/null; then
        missing_git=true
    fi

    if [ "$missing_git" = true ]; then
        echo "[!] Git is missing. Attempting to install..."
        if [ -x "$(command -v apt-get)" ]; then
            sudo apt-get update && sudo apt-get install -y git build-essential
        elif [ -x "$(command -v dnf)" ]; then
            sudo dnf install -y git
        elif [ -x "$(command -v pacman)" ]; then
            sudo pacman -S --noconfirm git base-devel
        elif [ -x "$(command -v brew)" ]; then
            brew install git
        else
            echo "[Error] Could not install git automatically. Please install git and run again."
            exit 1
        fi
    fi

    if ! command -v cargo &> /dev/null; then
        echo "[!] Rust (cargo) is missing. Installing via rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        # Source env to make cargo available immediately
        source "$HOME/.cargo/env"
    fi
}

install_deps

# 2. Clone Repository
TEMP_DIR=$(mktemp -d)
echo "[*] Cloning AeroShell from $REPO_URL to $TEMP_DIR..."

if git clone "$REPO_URL" "$TEMP_DIR/aeroshell"; then
    cd "$TEMP_DIR/aeroshell"
else
    echo "[Error] Failed to clone repository."
    rm -rf "$TEMP_DIR"
    exit 1
fi

# 3. Run Install Script
echo "[*] Running local installer..."
chmod +x install.sh
./install.sh

# 4. Cleanup
echo "[*] Cleaning up temporary files..."
cd "$HOME"
rm -rf "$TEMP_DIR"

echo "=========================================="
echo "   Success! AeroShell installed."
echo "=========================================="
