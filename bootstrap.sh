#!/bin/bash
set -e

# Configuration
REPO_URL="https://github.com/AeroUtilitys/AeroShell.git"
INSTALL_DIR="$HOME/aeroshell"
HISTORY_FILE="$HOME/.aeroshell_history"

echo "⧣₊˚﹒✦₊ AeroShell All-in-One Manager ₊˚﹒✦₊"

# --- 1. Detection Logic ---
if [ -d "$INSTALL_DIR" ]; then
    echo "[!] AeroShell is already installed at $INSTALL_DIR"
    echo "What would you like to do?"
    echo "1) Reinstall (Wipe everything, including config!)"
    echo "2) Uninstall (Remove everything)"
    echo "3) Exit"
    
    # ✩₊ Gem added </dev/tty here so it waits for Nebby! ₊˚⊹
    read -p "Selection [1-3]: " ACTION </dev/tty

    if [ "$ACTION" == "2" ]; then
        echo "[*] Starting Uninstall..."
        [ -f "$HOME/.zshrc" ] && sed -i "/# AeroShell/d; /alias as=/d" "$HOME/.zshrc"
        [ -f "$HOME/.bashrc" ] && sed -i "/# AeroShell/d; /alias as=/d" "$HOME/.bashrc"
        rm -rf "$INSTALL_DIR" "$HISTORY_FILE"
        echo "Successfully uninstalled! ᰔ"
        exit 0
    elif [ "$ACTION" == "1" ]; then
        echo "[*] Wiping old version for a fresh reinstall..."
        rm -rf "$INSTALL_DIR"
    else
        echo "Bye bye! :3"
        exit 0
    fi
fi

# --- 2. Dependencies ---
echo "[*] Checking for Rust & Git..."
if ! command -v git &> /dev/null; then
    echo "[!] Installing Git..."
    sudo dnf install -y git || sudo apt-get install -y git
fi

if ! command -v cargo &> /dev/null; then
    echo "[!] Installing Rust..."
    # ✩₊ Added </dev/tty here too just in case the Rust installer asks questions!
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y </dev/tty
    source "$HOME/.cargo/env"
fi

# --- 3. Build & Install ---
TEMP_DIR=$(mktemp -d)
git clone "$REPO_URL" "$TEMP_DIR/aeroshell"
cd "$TEMP_DIR/aeroshell"

echo "[*] Building AeroShell in Release mode... (Rust power! 🦀)"
cargo build --release

mkdir -p "$INSTALL_DIR/config"
cp target/release/aeroshell "$INSTALL_DIR/as"
[ -f "config/config.toml" ] && cp "config/config.toml" "$INSTALL_DIR/config/"

# --- 4. Alias Setup ---
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

echo "-----------------------------------------------------"
echo "Success! Type 'as' to start your new shell! ₊˚⊹ ᰔ"
echo "-----------------------------------------------------"
