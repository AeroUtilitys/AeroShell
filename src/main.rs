mod config;
mod prompt;

use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::env;
use crate::config::{load_config, save_config, apply_theme};
use crate::prompt::format_prompt;

fn main() {
    let mut config = load_config();

    loop {
        // 1. Print Prompt
        let prompt_str = format_prompt(&config.prompt_template, &config);
        print!("{}", prompt_str);
        if let Err(e) = io::stdout().flush() {
            eprintln!("Error flushing stdout: {}", e);
            continue;
        }

        // 2. Read Input
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(n) => {
                if n == 0 {
                    // EOF (Ctrl+D)
                    println!();
                    break;
                }
            },
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                continue;
            }
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        // 3. Parse Input
        let parts: Vec<&str> = input.split_whitespace().collect();
        let command = parts[0];
        let args = &parts[1..];

        // 4. Execute Command
        match command {
            "cd" => {
                let new_dir = if args.is_empty() {
                    env::var("HOME").unwrap_or_else(|_| "/".to_string())
                } else {
                    args[0].to_string()
                };

                if let Err(e) = env::set_current_dir(&new_dir) {
                    eprintln!("cd: {}", e);
                }
            },
            "exit" => break,
            "clear" => {
                print!("\x1B[2J\x1B[1;1H");
            },
            "config" => {
                if args.is_empty() {
                    println!("Usage: config <key> <value>");
                    println!("Keys: prompt, username");
                    println!("Current Config:");
                    println!("  prompt: {}", config.prompt_template);
                    println!("  username: {}", config.username);
                } else if args.len() >= 2 {
                    match args[0] {
                        "prompt" => {
                            // Join all remaining args to allow spaces in prompt
                            let new_prompt = args[1..].join(" ");
                            config.prompt_template = new_prompt;
                            if let Err(e) = save_config(&config) {
                                eprintln!("Error saving config: {}", e);
                            } else {
                                println!("Prompt updated.");
                            }
                        },
                        "username" => {
                            config.username = args[1].to_string();
                            if let Err(e) = save_config(&config) {
                                eprintln!("Error saving config: {}", e);
                            } else {
                                println!("Username updated.");
                            }
                        },
                        _ => println!("Unknown config key: {}", args[0]),
                    }
                }
            },
            "theme" => {
                if args.is_empty() {
                    println!("Usage: theme <name>");
                } else {
                    let theme_name = args[0];
                    if let Err(e) = apply_theme(&mut config, theme_name) {
                        eprintln!("Error applying theme: {}", e);
                    } else {
                        println!("Theme '{}' applied.", theme_name);
                    }
                }
            },
            "help" => {
                println!("AeroShell Built-in Commands:");
                println!("  cd <dir>         - Change directory");
                println!("  exit             - Exit shell");
                println!("  clear            - Clear screen");
                println!("  config <k> <v>   - Change configuration");
                println!("  theme <name>     - Apply a theme from themes/ dir");
                println!("  help             - Show this help");
            },
            cmd => {
                // External command
                let child_result = Command::new(cmd)
                    .args(args)
                    .stdin(Stdio::inherit())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .spawn();

                match child_result {
                    Ok(mut child) => {
                        if let Err(e) = child.wait() {
                            eprintln!("Error waiting for command: {}", e);
                        }
                    },
                    Err(_) => {
                        eprintln!("{}: command not found", cmd);
                    }
                }
            }
        }
    }
}
