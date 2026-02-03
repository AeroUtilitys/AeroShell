use crate::config::Config;
use chrono::Local;
use std::env;

fn hex_to_ansi(hex: &str) -> Option<String> {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        // ANSI 24-bit color: ESC[38;2;R;G;Bm
        Some(format!("\x1B[38;2;{};{};{}m", r, g, b))
    } else {
        None
    }
}

pub fn format_prompt(template: &str, config: &Config) -> String {
    let mut result = template.to_string();

    // 1. Replace Variables
    // %username%
    result = result.replace("%username%", &config.username);

    // %hostname%
    let hostname = env::var("HOSTNAME").unwrap_or_else(|_| "localhost".to_string());
    result = result.replace("%hostname%", &hostname);

    // %directory%
    let cwd = env::current_dir().unwrap_or_default();
    let cwd_str = cwd.to_string_lossy();
    // Optional: replace home with ~
    let home = env::var("HOME").unwrap_or_default();
    let display_cwd = if !home.is_empty() && cwd_str.starts_with(&home) {
        cwd_str.replacen(&home, "~", 1)
    } else {
        cwd_str.to_string()
    };
    result = result.replace("%directory%", &display_cwd);

    // %time%
    let time = Local::now().format("%H:%M:%S").to_string();
    result = result.replace("%time%", &time);

    // 2. Parse Colors/Styles (!tag!)
    let mut final_output = String::new();
    let mut chars = result.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '!' {
            // Potential start of tag
            let mut tag_content = String::new();
            let mut valid_tag = false;

            // Look ahead
            while let Some(&next_c) = chars.peek() {
                chars.next(); // consume
                if next_c == '!' {
                    valid_tag = true;
                    break;
                }
                tag_content.push(next_c);
            }

            if valid_tag {
                // Parse tag_content (e.g. "bold,yellow", "lightpink")
                let parts: Vec<&str> = tag_content.split(',').map(|s| s.trim()).collect();
                for part in parts {
                    // Check if it's a custom color first
                    if let Some(hex) = config.colors.get(part) {
                         if let Some(ansi) = hex_to_ansi(hex) {
                             final_output.push_str(&ansi);
                             continue;
                         }
                    }

                    match part {
                        "reset" => final_output.push_str("\x1B[0m"),
                        "bold" => final_output.push_str("\x1B[1m"),
                        "italic" => final_output.push_str("\x1B[3m"),
                        "underline" => final_output.push_str("\x1B[4m"),
                        "black" => final_output.push_str("\x1B[30m"),
                        "red" => final_output.push_str("\x1B[31m"),
                        "green" => final_output.push_str("\x1B[32m"),
                        "yellow" => final_output.push_str("\x1B[33m"),
                        "blue" => final_output.push_str("\x1B[34m"),
                        "magenta" => final_output.push_str("\x1B[35m"),
                        "cyan" => final_output.push_str("\x1B[36m"),
                        "white" => final_output.push_str("\x1B[37m"),
                        "grey" | "gray" => final_output.push_str("\x1B[90m"),
                         // For now, let's also support explicit hex in tag !#RRGGBB!
                        s if s.starts_with('#') => {
                             if let Some(ansi) = hex_to_ansi(s) {
                                 final_output.push_str(&ansi);
                             }
                        },
                        _ => {
                            // Unknown tag
                        }
                    }
                }
            } else {
                // No closing '!', treat as literal
                final_output.push('!');
                final_output.push_str(&tag_content);
            }
        } else {
            final_output.push(c);
        }
    }

    final_output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::collections::HashMap;

    #[test]
    fn test_variable_replacement() {
        let config = Config {
            prompt_template: "".to_string(),
            username: "testuser".to_string(),
            editor: "nano".to_string(),
            colors: HashMap::new(),
        };

        let res = format_prompt("Hello %username%", &config);
        assert_eq!(res, "Hello testuser");
    }

    #[test]
    fn test_custom_color() {
        let mut colors = HashMap::new();
        colors.insert("mypink".to_string(), "#FF00FF".to_string()); // Magenta
        let config = Config {
            prompt_template: "".to_string(),
            username: "test".to_string(),
            editor: "nano".to_string(),
            colors,
        };

        let res = format_prompt("!mypink!Hi", &config);
        // Expect ANSI 24-bit for FF00FF: 255;0;255
        // hex_to_ansi returns ESC[38;2;R;G;Bm
        assert_eq!(res, "\x1B[38;2;255;0;255mHi");
    }
}
