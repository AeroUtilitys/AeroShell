#!/bin/bash
set -e

echo "Uninstalling AeroShell..."

INSTALL_DIR="$HOME/aeroshell"
HISTORY_FILE="$HOME/.aeroshell_history"

# 1. Restore Default Shell if needed
CURRENT_SHELL=$(echo $SHELL)
# Check if current shell is aeroshell (basic check by name)
if [[ "$CURRENT_SHELL" == *"aeroshell"* ]] || [[ "$CURRENT_SHELL" == *"/as"* ]]; then
    echo "AeroShell is currently set as your default shell."

    # Get list of valid shells from /etc/shells, excluding aeroshell
    VALID_SHELLS=$(grep -v "aeroshell" /etc/shells | grep -v "/as$")

    # Count them
    COUNT=$(echo "$VALID_SHELLS" | wc -l)

    TARGET_SHELL=""

    if [ "$COUNT" -eq 0 ]; then
        echo "Warning: No other valid shells found in /etc/shells."
        echo "Please manually run 'chsh -s /bin/bash' (or similar) after uninstall."
    elif [ "$COUNT" -eq 1 ]; then
        TARGET_SHELL=$(echo "$VALID_SHELLS" | tr -d '[:space:]')
        echo "Restoring default shell to $TARGET_SHELL..."
        chsh -s "$TARGET_SHELL"
    else
        echo "Multiple shells found:"
        # Read into array for better handling
        SHELL_ARRAY=()
        while IFS= read -r line; do
            SHELL_ARRAY+=("$line")
        done <<< "$VALID_SHELLS"

        for i in "${!SHELL_ARRAY[@]}"; do
            echo "$((i+1)). ${SHELL_ARRAY[$i]}"
        done

        while true; do
            read -p "Select shell number to revert to: " SELECTION
            # Validate input is a number and within range
            if [[ "$SELECTION" =~ ^[0-9]+$ ]] && [ "$SELECTION" -ge 1 ] && [ "$SELECTION" -le "${#SHELL_ARRAY[@]}" ]; then
                TARGET_SHELL="${SHELL_ARRAY[$((SELECTION-1))]}"
                break
            else
                echo "Invalid selection. Please try again."
            fi
        done

        echo "Restoring default shell to $TARGET_SHELL..."
        chsh -s "$TARGET_SHELL"
    fi
fi

# 2. Remove Aliases
remove_alias() {
    local rc_file="$1"
    if [ -f "$rc_file" ]; then
        # Create backup
        cp "$rc_file" "$rc_file.bak"
        # Remove lines containing "alias as=" and "# AeroShell"
        # Using sed in-place is tricky cross-platform (Mac vs Linux), so we use a temp file approach
        grep -v "alias as='$INSTALL_DIR/as'" "$rc_file" | grep -v "# AeroShell" > "$rc_file.tmp" && mv "$rc_file.tmp" "$rc_file"
        echo "Cleaned up aliases in $rc_file"
    fi
}

remove_alias "$HOME/.zshrc"
remove_alias "$HOME/.bashrc"
remove_alias "$HOME/.bash_profile"

# 3. Remove Files
if [ -d "$INSTALL_DIR" ]; then
    echo "Removing $INSTALL_DIR..."
    rm -rf "$INSTALL_DIR"
fi

if [ -f "$HISTORY_FILE" ]; then
    echo "Removing history file..."
    rm "$HISTORY_FILE"
fi

echo "-----------------------------------------------------"
echo "Uninstallation complete!"
echo "If you switched shells, please log out and back in."
echo "-----------------------------------------------------"
