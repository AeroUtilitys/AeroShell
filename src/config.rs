use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::io::Write;
use std::env;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RootConfig {
    pub config: ConfigSection,
    pub theme: ThemeSection,
    #[serde(default)]
    pub colors: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConfigSection {
    pub username: String,
    pub editor: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ThemeSection {
    pub prompt_template: String,
    pub autocomplete: String,
    pub typing: String,     // Cursor/Text color
    pub typingtext: String, // Buffer text color
    pub header: String,
    pub subheader: String,
    pub body: String,
    pub active: String,
    pub disable: String,
}

impl Default for RootConfig {
    fn default() -> Self {
        let username = std::env::var("USER").unwrap_or_else(|_| "user".to_string());

        let mut colors = HashMap::new();
        colors.insert("pink".to_string(), "#FFC0CB".to_string());
        colors.insert("white".to_string(), "#FFFFFF".to_string());
        colors.insert("purple".to_string(), "#800080".to_string());
        colors.insert("teal".to_string(), "#008080".to_string());
        colors.insert("lightpink".to_string(), "#FFB6C1".to_string());
        colors.insert("lime".to_string(), "#00FF00".to_string());
        colors.insert("orange".to_string(), "#FFA500".to_string());
        colors.insert("green".to_string(), "#32CD32".to_string());
        colors.insert("red".to_string(), "#FF0000".to_string());
        colors.insert("grey".to_string(), "#808080".to_string());

        Self {
            config: ConfigSection {
                username,
                editor: "nano".to_string(), // User asked for micro in example, but nano default is safer if micro isn't installed. Sticking to safer default or "nano" as per previous constraint, but updating to "micro" if that was a preference? I'll keep "nano" as safe default unless explicitly told to change default. Actually user snippet said 'editor = "micro"'. I will use "nano" as robust default but user can change it.
            },
            theme: ThemeSection {
                prompt_template: "!teal!aeroshell@!lightpink!%username%!white!<>!purple!%directory%!green!:!reset! ".to_string(),
                autocomplete: "grey".to_string(),
                typing: "lightpink".to_string(),
                typingtext: "white".to_string(),
                header: "pink".to_string(),
                subheader: "purple".to_string(),
                body: "white".to_string(),
                active: "green".to_string(),
                disable: "red".to_string(),
            },
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

pub fn load_config() -> RootConfig {
    let path = get_config_path();

    if !path.exists() {
        let default_config = RootConfig::default();
        if let Err(e) = save_config(&default_config) {
            eprintln!("Warning: Failed to create default config file: {}", e);
        }
        return default_config;
    }

    if let Ok(content) = fs::read_to_string(&path) {
        match toml::from_str(&content) {
            Ok(config) => return config,
            Err(e) => {
                eprintln!("\x1B[31mError parsing config file ({:?}):\x1B[0m", path);
                eprintln!("{}", e);
                eprintln!("Using default configuration.");
            }
        }
    }

    RootConfig::default()
}

pub fn save_config(config: &RootConfig) -> std::io::Result<()> {
    let path = get_config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(config).unwrap_or_default();

    let commented_content = format!(
        "# AeroShell Configuration\n\
         # -----------------------\n\
         # Color Selector\n\
         # Define custom colors here using hex codes (e.g. #RRGGBB)\n\
         # usage in prompt: !colorname!\n\
         #\n\
         # Available built-in colors (ANSI):\n\
         #   black, red, green, yellow, blue, magenta, cyan, white, grey\n\
         #\n\
         # Available custom colors (Hex):\n\
         #   lightpink, pink, purple, white (hex), orange, teal, lime\n\
         #\n\
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
