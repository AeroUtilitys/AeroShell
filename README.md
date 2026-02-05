# AeroShell

A customizable, Rust-based shell for macOS and Linux.

## Features

-   **Customizable Prompt:** Use `!color!` tags and hex codes.
-   **Autosuggestions:** "Fish-like" ghost text and tab completion.
-   **Configuration:** Simple TOML config with extensive comments.
-   **System-Wide Themes:** Colors defined in AeroShell are exported as environment variables for other programs.
-   **Self-Update:** Update directly from a source zip.

## Installation

*(Note: Make sure you have Curl installed!).*

**Automatic Install**

*Latest*
`curl -sSf https://raw.githubusercontent.com/AeroUtilitys/AeroShell/main/bootstrap.sh | bash`

### If you want the latest Dev or Nightly version
Download the Latest Stable Version then get the lastest Dev **update.zip** the use the Aero Update command on the Stable Version

## Updating

> [!CAUTION]
> If you update aero to a older version than your base, **It will break things**

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

Run `curl -sSf https://raw.githubusercontent.com/AeroUtilitys/AeroShell/main/bootstrap.sh | bash` Then select Uninstall, If your having problems, Open an Issue and try reinstalling using the bootstrap script provided to reinstall

# Contact
*Email:* recloudnoreply@gmail.com

## Important Information

> [!IMPORTANT]
> This project was made partly by AI

This does not mean the entire project was created by AI, I created (nebuff) the scripts but Jules (Googles code Bot) Helped me in writing rust since Im not familier with Rust, I know some people dont like using or being involved with AI so thats why I decided to put this in to the Readme, Most of my Projects are to show the powers of Ai while making Useful stuff out of the AI Boom, I Hope you like AeroShell and the Other utilitys my Project Aero has
