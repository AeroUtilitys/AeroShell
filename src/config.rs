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
}

impl Default for Config {
    fn default() -> Self {
        let username = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
        Self {
            prompt_template: "!green!%username%!reset!@!blue!%hostname%!reset! %directory% > ".to_string(),
            username,
            theme: "default".to_string(),
        }
    }
}

pub fn get_app_root() -> PathBuf {
    // Try to find the home directory
    if let Ok(home) = env::var("HOME") {
        return PathBuf::from(home).join("aeroshell");
    }
    // Fallback to current directory
    PathBuf::from(".")
}

pub fn get_config_path() -> PathBuf {
    get_app_root().join("config/config.json")
}

pub fn get_themes_dir() -> PathBuf {
    get_app_root().join("themes")
}

pub fn load_config() -> Config {
    let path = get_config_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(config) = serde_json::from_str(&content) {
                return config;
            }
        }
    }
    Config::default()
}

pub fn save_config(config: &Config) -> std::io::Result<()> {
    let path = get_config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(config)?;
    let mut file = fs::File::create(path)?;
    file.write_all(content.as_bytes())?;
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
