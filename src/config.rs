use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::io::Write;
use std::env;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub prompt_template: String,
    pub username: String,
    pub theme: String,
    pub editor: String,
}

impl Default for Config {
    fn default() -> Self {
        let username = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
        Self {
            prompt_template: "!green!%username%!reset!@!blue!%hostname%!reset! %directory% > ".to_string(),
            username,
            theme: "default".to_string(),
            editor: "nano".to_string(),
        }
    }
}

pub fn get_app_root() -> PathBuf {
    if let Ok(home) = env::var("HOME") {
        return PathBuf::from(home).join("aeroshell");
    }
    PathBuf::from(".")
}

pub fn get_config_path() -> PathBuf {
    get_app_root().join("config/config.toml")
}

pub fn get_themes_dir() -> PathBuf {
    get_app_root().join("themes")
}

pub fn load_config() -> Config {
    let path = get_config_path();

    // If config file doesn't exist, create it with comments
    if !path.exists() {
        let default_config = Config::default();
        if let Err(e) = save_config(&default_config) {
            eprintln!("Warning: Failed to create default config file: {}", e);
        }
        return default_config;
    }

    if let Ok(content) = fs::read_to_string(&path) {
        // Try TOML first
        if let Ok(config) = toml::from_str(&content) {
            return config;
        }
        // Fallback to JSON (migration path)
         if let Ok(config) = serde_json::from_str(&content) {
            return config;
        }
    }

    Config::default()
}

pub fn save_config(config: &Config) -> std::io::Result<()> {
    let path = get_config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    // TOML is more human readable
    let content = toml::to_string_pretty(config).unwrap_or_default();

    // If file doesn't exist, we might want to add comments.
    // Ideally we would use a template string with comments.
    // Logic: always write comments if we are saving (overwriting).
    // Note: this wipes user comments if they added any custom ones manually,
    // unless we parse them, but for a simple "lots of comments explaining things" requirement,
    // rewriting the standard comments block is safer than parsing partial TOML.

    let commented_content = format!(
        "# AeroShell Configuration\n\
         # -----------------------\n\
         # prompt_template: The string used for your prompt.\n\
         #   Variables: %username%, %hostname%, %directory%, %time%\n\
         #   Styles: !red!, !bold!, !green!, !reset!, etc.\n\
         # username: The username displayed in the prompt.\n\
         # theme: The active theme name (from themes/ directory).\n\
         # editor: The editor command used for 'config nano' (e.g. 'nano', 'vim', 'code').\n\n\
         {}",
        content
    );
     let mut file = fs::File::create(path)?;
     file.write_all(commented_content.as_bytes())?;

    Ok(())
}

pub fn apply_theme(config: &mut Config, theme_name: &str) -> std::io::Result<()> {
    let path = get_themes_dir().join(format!("{}.json", theme_name));
    let content = fs::read_to_string(&path)?;
    let theme: serde_json::Value = serde_json::from_str(&content)?;

    if let Some(prompt) = theme.get("prompt_template").and_then(|v| v.as_str()) {
        config.prompt_template = prompt.to_string();
    }
    config.theme = theme_name.to_string();
    save_config(config)?;
    Ok(())
}
