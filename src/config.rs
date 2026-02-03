use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::io::Write;
use std::env;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub prompt_template: String,
    pub username: String,
    pub editor: String,
    #[serde(default)]
    pub colors: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        let username = std::env::var("USER").unwrap_or_else(|_| "user".to_string());

        // Define default example colors
        let mut colors = HashMap::new();
        colors.insert("lightpink".to_string(), "#FFB6C1".to_string());
        colors.insert("purple".to_string(), "#800080".to_string());

        Self {
            prompt_template: "!lightpink!%username%!grey!<>!purple!%directory%!green!:!reset! ".to_string(),
            username,
            editor: "nano".to_string(),
            colors,
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

pub fn load_config() -> Config {
    let path = get_config_path();

    if !path.exists() {
        let default_config = Config::default();
        if let Err(e) = save_config(&default_config) {
            eprintln!("Warning: Failed to create default config file: {}", e);
        }
        return default_config;
    }

    if let Ok(content) = fs::read_to_string(&path) {
        match toml::from_str(&content) {
            Ok(config) => return config,
            Err(e) => {
                // Print specific error details
                eprintln!("\x1B[31mError parsing config file ({:?}):\x1B[0m", path);
                eprintln!("{}", e);
                eprintln!("Using default configuration.");
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
    let content = toml::to_string_pretty(config).unwrap_or_default();

    // Add extensive comments as requested
    let commented_content = format!(
        "# AeroShell Configuration\n\
         # -----------------------\n\
         # Color Selector\n\
         # define custom colors here using hex codes (e.g. #RRGGBB)\n\
         # usage in prompt: !colorname!\n\
         # Available built-in colors: \n\
         #   black, red, green, yellow, blue, magenta, cyan, white, grey\n\
         # Available styles:\n\
         #   bold, italic, underline, reset\n\
         #\n\
         {}\n",
        content
    );
     let mut file = fs::File::create(path)?;
     file.write_all(commented_content.as_bytes())?;

    Ok(())
}
