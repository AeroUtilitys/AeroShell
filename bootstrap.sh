#!/bin/bash
set -e

# Configuration
REPO_URL="https://github.com/AeroUtilitys/AeroShell.git"
INSTALL_DIR="$HOME/aeroshell"
HISTORY_FILE="$HOME/.aeroshell_history"

# --- Visual Helper ---
print_header() {
    echo ""
    echo "AeroShell Installer"
    echo "=============================="
}

print_option() {
    local key="$1"
    local desc="$2"
    printf "  %-10s - %s\n" "$key" "$desc"
}

# --- Main Menu ---
clear
print_header

if [ -d "$INSTALL_DIR" ]; then
    echo "[!] AeroShell is already installed at $INSTALL_DIR"
    echo ""
    print_option "1" "Reinstall (Wipe & Install Fresh)"
    print_option "2" "Uninstall (Remove Completely)"
    print_option "3" "Exit"
    echo ""
    read -p "Selection [1-3]: " ACTION

    if [ "$ACTION" == "2" ]; then
        echo ""
        echo "Starting Uninstall..."
        [ -f "$HOME/.zshrc" ] && sed -i "/# AeroShell/d; /alias as=/d" "$HOME/.zshrc"
        [ -f "$HOME/.bashrc" ] && sed -i "/# AeroShell/d; /alias as=/d" "$HOME/.bashrc"
        [ -f "$HOME/.profile" ] && sed -i "/# AeroShell/d; /alias as=/d" "$HOME/.profile"
        rm -rf "$INSTALL_DIR" "$HISTORY_FILE"
        echo "Successfully uninstalled!"
        exit 0
    elif [ "$ACTION" == "1" ]; then
        echo "Wiping old version..."
        rm -rf "$INSTALL_DIR"
    else
        exit 0
    fi
    # If reinstalling, fall through to install menu
fi

clear
print_header
print_option "1" "Install Stable (Recommended)"
print_option "2" "Install Dev (Buggy/Experimental)"
print_option "3" "Exit"
echo ""
read -p "Selection [1-3]: " INST_TYPE

BRANCH="main"
BUILD_FLAG="--release"
TARGET_DIR="release"

if [ "$INST_TYPE" == "2" ]; then
    echo ""
    echo "WARNING: Dev builds may be unstable and contain bugs."
    echo "Features may change or break without notice."
    read -p "Proceed with caution? (y/N): " CONFIRM
    if [[ "$CONFIRM" != "y" && "$CONFIRM" != "Y" ]]; then
        echo "Aborted."
        exit 0
    fi
    BRANCH="dev"
    BUILD_FLAG=""
    TARGET_DIR="debug"
elif [ "$INST_TYPE" == "3" ]; then
    exit 0
fi

# --- Dependencies ---
echo ""
echo "Checking dependencies..."

if ! command -v git &> /dev/null; then
    echo "Installing Git..."
    sudo dnf install -y git || sudo apt-get install -y git
fi

if ! command -v cargo &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# --- Build & Install ---
TEMP_DIR=$(mktemp -d)
echo "Cloning $BRANCH branch..."
git clone -b "$BRANCH" "$REPO_URL" "$TEMP_DIR/aeroshell"
cd "$TEMP_DIR/aeroshell"

echo "Building AeroShell ($BRANCH)..."
cargo build $BUILD_FLAG

# Verify build
BINARY_PATH="target/$TARGET_DIR/aeroshell"
if [ ! -f "$BINARY_PATH" ]; then
    echo "Error: Build failed. Binary not found at $BINARY_PATH"
    exit 1
fi

echo "Installing to $INSTALL_DIR..."
mkdir -p "$INSTALL_DIR/config"
cp "$BINARY_PATH" "$INSTALL_DIR/as"

# Default config copy
if [ ! -f "$INSTALL_DIR/config/config.toml" ]; then
    [ -f "config.example.toml" ] && cp "config.example.toml" "$INSTALL_DIR/config/config.toml"
    # Fallback if config.example.toml is not there but maybe config/config.toml exists in repo
    [ -f "config/config.toml" ] && cp "config/config.toml" "$INSTALL_DIR/config/"
fi

# --- Alias Setup ---
add_alias() {
    if [ -f "$1" ] && ! grep -q "alias as=" "$1"; then
        echo -e "\n# AeroShell\nalias as='$INSTALL_DIR/as'" >> "$1"
    fi
}
add_alias "$HOME/.zshrc"
add_alias "$HOME/.bashrc"
add_alias "$HOME/.profile"

# Cleanup
rm -rf "$TEMP_DIR"

echo ""
echo "Success! Type 'as' to start your new shell!"
