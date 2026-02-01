use crate::config::Config;
use chrono::Local;
use std::env;

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
                // Parse tag_content (e.g. "bold,yellow")
                let parts: Vec<&str> = tag_content.split(',').map(|s| s.trim()).collect();
                for part in parts {
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
                        _ => {
                            // Unknown tag, maybe print literally?
                            // For safety/simplicity, ignoring unknown tags.
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

    #[test]
    fn test_variable_replacement() {
        let config = Config {
            prompt_template: "".to_string(),
            username: "testuser".to_string(),
            theme: "default".to_string(),
        };

        // Mock env vars would be ideal, but for now we test username from config
        let res = format_prompt("Hello %username%", &config);
        assert_eq!(res, "Hello testuser");
    }

    #[test]
    fn test_color_parsing() {
        let config = Config::default();
        let res = format_prompt("!red!Hello!reset!", &config);
        assert_eq!(res, "\x1B[31mHello\x1B[0m");
    }

    #[test]
    fn test_combined_styles() {
        let config = Config::default();
        let res = format_prompt("!bold,blue!Text", &config);
        // order depends on implementation, we push bold then blue
        assert_eq!(res, "\x1B[1m\x1B[34mText");
    }

    #[test]
    fn test_invalid_tags() {
        let config = Config::default();
        let res = format_prompt("!invalid!", &config);
        // Invalid tags are ignored (consumed but output nothing based on current logic)
        // logic: matches _ => {}
        assert_eq!(res, "");

        let res2 = format_prompt("!notatag", &config);
        // Missing closing !, treated as literal
        assert_eq!(res2, "!notatag");
    }
}
