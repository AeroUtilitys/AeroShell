mod config;
mod prompt;
mod completer;

use std::process::{Command, Stdio};
use std::env;
use crate::config::{load_config, save_config, apply_theme, get_config_path};
use crate::prompt::format_prompt;
use crate::completer::AeroCompleter;

use reedline::{
    Reedline, Signal, DefaultHinter,
    FileBackedHistory
};

fn main() {
    let mut config = load_config();

    // Setup Reedline
    let history_path = env::var("HOME")
        .map(|h| format!("{}/.aeroshell_history", h))
        .unwrap_or_else(|_| ".aeroshell_history".to_string());

    let history = Box::new(
        FileBackedHistory::with_file(2000, history_path.into())
            .expect("Error configuring history with file"),
    );

    let mut line_editor = Reedline::create()
        .with_history(history)
        .with_hinter(Box::new(DefaultHinter::default().with_style(nu_ansi_term::Style::new().italic().fg(nu_ansi_term::Color::Cyan))))
        .with_completer(Box::new(AeroCompleter));

    loop {
        // 1. Generate Prompt
        let prompt_str = format_prompt(&config.prompt_template, &config);

        struct AeroPrompt(String);
        impl reedline::Prompt for AeroPrompt {
            fn render_prompt_left(&self) -> std::borrow::Cow<'_, str> {
                std::borrow::Cow::Borrowed(&self.0)
            }
            fn render_prompt_right(&self) -> std::borrow::Cow<'_, str> {
                std::borrow::Cow::Borrowed("")
            }
            fn render_prompt_indicator(&self, _prompt_mode: reedline::PromptEditMode) -> std::borrow::Cow<'_, str> {
                std::borrow::Cow::Borrowed("")
            }
            fn render_prompt_multiline_indicator(&self) -> std::borrow::Cow<'_, str> {
                 std::borrow::Cow::Borrowed(".. ")
            }
            fn render_prompt_history_search_indicator(&self, _history_search: reedline::PromptHistorySearch) -> std::borrow::Cow<'_, str> {
                std::borrow::Cow::Borrowed("(search) ")
            }
        }

        let prompt = AeroPrompt(prompt_str);

        match line_editor.read_line(&prompt) {
            Ok(Signal::Success(buffer)) => {
                let input = buffer.trim();
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
                        let _ = line_editor.clear_screen();
                    },
                    "config" => {
                         if args.is_empty() {
                            println!("Usage: config <command> [args]");
                            println!("Commands:");
                            println!("  nano            - Open config in configured editor");
                            println!("  set-default     - Set AeroShell as your default shell");
                            println!("  prompt <val>    - Set prompt template");
                            println!("  username <val>  - Set username");
                            println!("  editor <val>    - Set default editor");
                        } else {
                            match args[0] {
                                "nano" | "edit" => {
                                    // Open config file in editor
                                    let editor = &config.editor;
                                    let config_path = get_config_path();

                                    println!("Opening config in {}...", editor);

                                    let _ = Command::new(editor)
                                        .arg(config_path)
                                        .status()
                                        .map_err(|e| eprintln!("Failed to open editor: {}", e));

                                    // Reload config after edit
                                    config = load_config();
                                },
                                "set-default" => {
                                    println!("Setting AeroShell as default shell...");
                                    // Get path to current executable
                                    if let Ok(exe_path) = env::current_exe() {
                                        let path_str = exe_path.to_string_lossy().to_string();
                                        println!("Run: chsh -s {}", path_str);
                                        // Try running chsh
                                        let status = Command::new("chsh")
                                            .arg("-s")
                                            .arg(&exe_path)
                                            .status();

                                        match status {
                                            Ok(s) if s.success() => println!("Success! Restart your terminal."),
                                            _ => println!("Failed to set default shell. You may need to add {} to /etc/shells first.", path_str),
                                        }
                                    }
                                },
                                "prompt" if args.len() > 1 => {
                                    let new_prompt = args[1..].join(" ");
                                    config.prompt_template = new_prompt;
                                    save_config(&config).unwrap_or_else(|e| eprintln!("Save error: {}", e));
                                },
                                "username" if args.len() > 1 => {
                                    config.username = args[1].to_string();
                                    save_config(&config).unwrap_or_else(|e| eprintln!("Save error: {}", e));
                                },
                                "editor" if args.len() > 1 => {
                                    config.editor = args[1].to_string();
                                    save_config(&config).unwrap_or_else(|e| eprintln!("Save error: {}", e));
                                },
                                _ => println!("Unknown config command or missing args."),
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
                        // Colored Help
                        let title = "\x1B[1;36m"; // Bold Cyan
                        let cmd_color = "\x1B[32m"; // Green
                        let reset = "\x1B[0m";

                        println!("{}AeroShell Built-in Commands:{}", title, reset);
                        println!("  {}cd <dir>{}         - Change directory", cmd_color, reset);
                        println!("  {}exit{}             - Exit shell", cmd_color, reset);
                        println!("  {}clear{}            - Clear screen", cmd_color, reset);
                        println!("  {}config{}           - Manage configuration", cmd_color, reset);
                        println!("  {}theme <name>{}     - Apply a theme", cmd_color, reset);
                        println!("  {}help{}             - Show this help", cmd_color, reset);
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
            Ok(Signal::CtrlD) => {
                println!("Aborted!");
                break;
            }
            Ok(Signal::CtrlC) => {
                println!("^C");
                continue;
            }
            Err(e) => {
                eprintln!("Error: {:?}", e);
            }
        }
    }
}
