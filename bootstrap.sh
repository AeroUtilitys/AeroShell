#!/bin/bash
set -e

REPO_URL="https://github.com/AeroUtilitys/AeroShell.git"
INSTALL_DIR="$HOME/aeroshell"
HISTORY_FILE="$HOME/.aeroshell_history"

echo "⧣₊˚﹒✦₊ AeroShell All-in-One Manager ₊˚﹒✦₊"

if [ -d "$INSTALL_DIR" ]; then
    echo "[!] AeroShell is already installed at $INSTALL_DIR"
    echo ""
    echo "  1  - Reinstall (Wipe & Install Fresh)"
    echo "  2  - Uninstall (Remove Completely)"
    echo "  3  - Exit"
    echo ""
    
    # ✩₊ Redirecting input from the terminal device!
    read -p "Selection: " ACTION < /dev/tty

    case $ACTION in
        1)
            echo "[*] Wiping old version... ⧣₊˚"
            rm -rf "$INSTALL_DIR"
            ;;
        2)
            echo "[*] Starting Uninstall... ₊˚⊹"
            [ -f "$HOME/.zshrc" ] && sed -i "/# AeroShell/d; /alias as=/d" "$HOME/.zshrc"
            [ -f "$HOME/.bashrc" ] && sed -i "/# AeroShell/d; /alias as=/d" "$HOME/.bashrc"
            rm -rf "$INSTALL_DIR" "$HISTORY_FILE"
            echo "Successfully uninstalled! ᰔ"
            exit 0
            ;;
        *)
            echo "Bye bye! :3"
            exit 0
            ;;
    esac
fi

echo "[*] Checking for Rust & Git..."
if ! command -v git &> /dev/null; then
    sudo dnf install -y git || sudo apt-get install -y git
fi

if ! command -v cargo &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y < /dev/tty
    source "$HOME/.cargo/env"
fi

TEMP_DIR=$(mktemp -d)
git clone "$REPO_URL" "$TEMP_DIR/aeroshell"
cd "$TEMP_DIR/aeroshell"

echo "[*] Building AeroShell... (Rust power! 🦀)"
cargo build --release

mkdir -p "$INSTALL_DIR/config"
cp target/release/aeroshell "$INSTALL_DIR/as"
[ -f "config/config.toml" ] && cp "config/config.toml" "$INSTALL_DIR/config/"

add_alias() {
    if [ -f "$1" ] && ! grep -q "alias as=" "$1"; then
        echo -e "\n# AeroShell\nalias as='$INSTALL_DIR/as'" >> "$1"
    fi
}
add_alias "$HOME/.zshrc"
add_alias "$HOME/.bashrc"
add_alias "$HOME/.profile"

rm -rf "$TEMP_DIR"

echo "-----------------------------------------------------"
echo "Success! Type 'as' to start your new shell! ₊˚⊹ ᰔ"
echo "-----------------------------------------------------"
