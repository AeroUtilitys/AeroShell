#!/bin/bash
set -e

echo "Installing AeroShell..."

# 1. Check Cargo
if ! command -v cargo &> /dev/null; then
    echo "Rust (cargo) is not installed. Please install Rust first: https://rustup.rs/"
    exit 1
fi

# 2. Build
echo "Building release binary..."
# Ensure we are in the project root
cd "$(dirname "$0")"
cargo build --release

# 3. Setup Install Dir
INSTALL_DIR="$HOME/aeroshell"
echo "Installing to $INSTALL_DIR..."
mkdir -p "$INSTALL_DIR"
mkdir -p "$INSTALL_DIR/themes"
mkdir -p "$INSTALL_DIR/config"

# 4. Copy Files
cp target/release/aeroshell "$INSTALL_DIR/as"
cp themes/*.json "$INSTALL_DIR/themes/" 2>/dev/null || true
# Copy default config if it exists, but don't overwrite user config
if [ -f "config/config.toml" ] && [ ! -f "$INSTALL_DIR/config/config.toml" ]; then
    cp config/config.toml "$INSTALL_DIR/config/"
fi

# 5. Add to PATH/Alias
# We try to add alias to both zshrc and bashrc if they exist
ADDED_ALIAS=false

add_alias() {
    local rc_file="$1"
    if [ -f "$rc_file" ]; then
        if ! grep -q "alias as=" "$rc_file"; then
            echo "" >> "$rc_file"
            echo "# AeroShell" >> "$rc_file"
            echo "alias as='$INSTALL_DIR/as'" >> "$rc_file"
            echo "Added alias 'as' to $rc_file"
            ADDED_ALIAS=true
        else
            echo "Alias 'as' already exists in $rc_file"
            ADDED_ALIAS=true
        fi
    fi
}

add_alias "$HOME/.zshrc"
add_alias "$HOME/.bashrc"
add_alias "$HOME/.bash_profile"

if [ "$ADDED_ALIAS" = false ]; then
    echo "Could not find standard shell config files. Please manually alias:"
    echo "alias as='$INSTALL_DIR/as'"
fi

echo "-----------------------------------------------------"
echo "Installation complete!"
echo "Type 'as' to launch AeroShell."
echo "(You may need to restart your terminal or source your rc file)"
echo "-----------------------------------------------------"
