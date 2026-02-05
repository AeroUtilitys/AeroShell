# AeroShell

A customizable, Rust-based shell for macOS and Linux.

## Features

-   **Customizable Prompt:** Use `!color!` tags and hex codes.
-   **Autosuggestions:** "Fish-like" ghost text and tab completion.
-   **Configuration:** Simple TOML config with extensive comments.
-   **System-Wide Themes:** Colors defined in AeroShell are exported as environment variables for other programs.
-   **Self-Update:** Update directly from a source zip.

## Installation

**Automatic Install**

*Latest*
`curl -sSf https://raw.githubusercontent.com/AeroUtilitys/AeroShell/main/bootstrap.sh | bash`

*Dev (Nightly)*
`curl -sSf https://raw.githubusercontent.com/AeroUtilitys/AeroShell/dev/bootstrap.sh | bash`

## Updating

To update AeroShell from a source zip:
```bash
aero update path/to/update.zip
```

## Configuration

Run `aero config` to open your configuration file.

### Structure
The config file `~/.aeroshell/config/config.toml` has three main sections:

1.  `[config]`: Basic settings (username, editor).
2.  `[theme]`: UI element colors (prompt, autocomplete, headers).
3.  `[colors]`: Define your own custom colors here!

Example:
```toml
[colors]
mypink = "#FF00FF"

[theme]
prompt_template = "!mypink!%username% > "
```

## Developing for AeroShell

AeroShell exports your theme configuration as environment variables, allowing other CLI tools to adapt to your theme.

**Available Variables:**

*   `AERO_THEME_HEADER`
*   `AERO_THEME_SUBHEADER`
*   `AERO_THEME_BODY`
*   `AERO_THEME_ACTIVE`
*   `AERO_THEME_DISABLE`
*   `AERO_THEME_AUTOCOMPLETE`
*   `AERO_THEME_TYPING`
*   `AERO_COLOR_<NAME>` (e.g., `AERO_COLOR_PINK`, `AERO_COLOR_TEAL`)

**Example (Bash Script):**
```bash
#!/bin/bash
# Use AeroShell colors if available
COLOR_HEADER=${AERO_THEME_HEADER:-green}
echo -e "!$COLOR_HEADER!Welcome to my script"
```
*(Note: You'll need to parse the color names or hex codes depending on your language's capability).*

## Uninstall

Run the provided `uninstall.sh` script to remove AeroShell and restore your previous shell.
