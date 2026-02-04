mod config;
mod prompt;
mod completer;

use std::process::{Command, Stdio};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::io::Read;
use crate::config::{load_config, get_config_path, RootConfig};
use crate::prompt::format_prompt;
use crate::completer::AeroCompleter;

use reedline::{
    Reedline, Signal, DefaultHinter,
    FileBackedHistory
};

fn hex_to_rgb(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some((r, g, b))
    } else {
        None
    }
}

// Helper to convert config color name to nu-ansi-term Style for Reedline
fn get_style_from_config(color_name: &str, config: &RootConfig) -> nu_ansi_term::Style {
    // 1. Check custom colors (Hex)
    if let Some(hex) = config.colors.get(color_name) {
        if let Some((r, g, b)) = hex_to_rgb(hex) {
            return nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Rgb(r, g, b));
        }
    }

    // 2. Check built-ins
    match color_name {
        "black" => nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Black),
        "red" => nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Red),
        "green" => nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Green),
        "yellow" => nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Yellow),
        "blue" => nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Blue),
        "magenta" => nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Magenta),
        "cyan" => nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Cyan),
        "white" => nu_ansi_term::Style::new().fg(nu_ansi_term::Color::White),
        "grey" | "gray" => nu_ansi_term::Style::new().fg(nu_ansi_term::Color::LightGray), // Or DarkGray depending on preference
        _ => nu_ansi_term::Style::default(), // Default if not found
    }
}

// Convert config color to simple ANSI string for help menu manually
fn get_ansi_from_config(color_name: &str, config: &RootConfig) -> String {
    // 1. Hex
    if let Some(hex) = config.colors.get(color_name) {
        if let Some((r, g, b)) = hex_to_rgb(hex) {
            return format!("\x1B[38;2;{};{};{}m", r, g, b);
        }
    }
    // 2. Built-in
    match color_name {
        "black" => "\x1B[30m".to_string(),
        "red" => "\x1B[31m".to_string(),
        "green" => "\x1B[32m".to_string(),
        "yellow" => "\x1B[33m".to_string(),
        "blue" => "\x1B[34m".to_string(),
        "magenta" => "\x1B[35m".to_string(),
        "cyan" => "\x1B[36m".to_string(),
        "white" => "\x1B[37m".to_string(),
        "grey" | "gray" => "\x1B[90m".to_string(),
        _ => "\x1B[0m".to_string(),
    }
}

fn main() {
    // 0. Setup Global Signal Handler
    ctrlc::set_handler(move || {
        // Do nothing.
    }).expect("Error setting Ctrl-C handler");

    let mut config = load_config();

    // 3. Export Environment Variables (System-wide colors)
    // Export [colors]
    for (name, hex) in &config.colors {
        env::set_var(format!("AERO_COLOR_{}", name.to_uppercase()), hex);
    }
    // Export [theme] parts (names of colors)
    let t = &config.theme;
    env::set_var("AERO_THEME_AUTOCOMPLETE", &t.autocomplete);
    env::set_var("AERO_THEME_TYPING", &t.typing);
    env::set_var("AERO_THEME_TYPINGTEXT", &t.typingtext);
    env::set_var("AERO_THEME_HEADER", &t.header);
    env::set_var("AERO_THEME_SUBHEADER", &t.subheader);
    env::set_var("AERO_THEME_BODY", &t.body);
    env::set_var("AERO_THEME_ACTIVE", &t.active);
    env::set_var("AERO_THEME_DISABLE", &t.disable);

    // Setup Reedline
    let history_path = env::var("HOME")
        .map(|h| format!("{}/.aeroshell_history", h))
        .unwrap_or_else(|_| ".aeroshell_history".to_string());

    let history = Box::new(
        FileBackedHistory::with_file(2000, history_path.into())
            .expect("Error configuring history with file"),
    );

    // Apply styles
    let hint_style = get_style_from_config(&config.theme.autocomplete, &config);
    // Note: Reedline doesn't support changing the "typing text" color dynamically easily
    // without a custom painter, but we can set the hint style easily.
    // We will stick to configuring the hinter for now.

    let mut line_editor = Reedline::create()
        .with_history(history)
        .with_hinter(Box::new(DefaultHinter::default().with_style(hint_style)))
        .with_completer(Box::new(AeroCompleter));

    loop {
        // 1. Generate Prompt
        let prompt_str = format_prompt(&config.theme.prompt_template, &config);

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

                // 3. Parse Input (with quotes support)
                let parts: Vec<String> = match shlex::split(input) {
                    Some(args) => args,
                    None => {
                        eprintln!("Error: Unmatched quote found.");
                        continue;
                    }
                };

                if parts.is_empty() {
                    continue;
                }

                let command = &parts[0];
                let args: Vec<&str> = parts[1..].iter().map(|s| s.as_str()).collect();

                match command.as_str() {
                    "cd" => {
                        let new_dir = if args.is_empty() {
                            env::var("HOME").unwrap_or_else(|_| "/".to_string())
                        } else {
                            if args[0].starts_with("~") {
                                let home = env::var("HOME").unwrap_or_else(|_| "/".to_string());
                                if args[0] == "~" {
                                    home
                                } else {
                                    args[0].replacen("~", &home, 1)
                                }
                            } else {
                                args[0].to_string()
                            }
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
                        open_config(&config);
                        config = load_config();
                    },
                    "aero" => {
                         if args.is_empty() {
                            println!("Usage: aero <command>");
                            println!("Commands:");
                            println!("  config           - Open configuration in editor");
                            println!("  setdefault       - Set AeroShell as default shell");
                            println!("  update <zipfile> - Update AeroShell from a source zip");
                        } else {
                            match args[0] {
                                "config" => {
                                    open_config(&config);
                                    config = load_config();
                                },
                                "setdefault" => {
                                    println!("Setting AeroShell as default shell...");
                                    if let Ok(exe_path) = env::current_exe() {
                                        let path_str = exe_path.to_string_lossy().to_string();

                                        let mut needs_add = true;
                                        if let Ok(mut file) = fs::File::open("/etc/shells") {
                                            let mut contents = String::new();
                                            if file.read_to_string(&mut contents).is_ok() {
                                                if contents.lines().any(|line| line.trim() == path_str) {
                                                    needs_add = false;
                                                }
                                            }
                                        }

                                        if needs_add {
                                            println!("Adding {} to /etc/shells (requires sudo)...", path_str);
                                            let status = Command::new("sudo")
                                                .arg("sh")
                                                .arg("-c")
                                                .arg(format!("echo '{}' >> /etc/shells", path_str))
                                                .status();

                                            if let Ok(s) = status {
                                                if !s.success() {
                                                    eprintln!("Failed to add to /etc/shells. Aborting.");
                                                    continue;
                                                }
                                            } else {
                                                eprintln!("Failed to run sudo. Aborting.");
                                                continue;
                                            }
                                        }

                                        println!("Changing shell (requires password)...");
                                        let status = Command::new("chsh")
                                            .arg("-s")
                                            .arg(&exe_path)
                                            .status();

                                        match status {
                                            Ok(s) if s.success() => println!("Success! Please log out and back in."),
                                            _ => println!("Failed to set default shell."),
                                        }
                                    }
                                },
                                "update" if args.len() > 1 => {
                                    let zip_path = args[1];
                                    if let Err(e) = update_aeroshell(zip_path) {
                                        eprintln!("Update failed: {}", e);
                                    } else {
                                        println!("Update successful! Restart AeroShell to see changes.");
                                    }
                                },
                                _ => println!("Unknown aero command: {}", args[0]),
                            }
                        }
                    },
                    "help" => {
                        // Apply config colors
                        let header_c = get_ansi_from_config(&config.theme.header, &config);
                        let subheader_c = get_ansi_from_config(&config.theme.subheader, &config);
                        let body_c = get_ansi_from_config(&config.theme.body, &config);
                        let active_c = get_ansi_from_config(&config.theme.active, &config);
                        let reset = "\x1B[0m";

                        println!("\n{}AeroShell Built-in Commands:{}", header_c, reset);
                        println!("{}", "=".repeat(30));

                        let commands = [
                            ("cd", "<dir>", "Change directory"),
                            ("exit", "", "Exit shell"),
                            ("clear", "", "Clear screen"),
                            ("config", "", "Open configuration"),
                            ("aero", "<cmd>", "Manage AeroShell (setdefault, update, etc)"),
                            ("help", "", "Show this help"),
                        ];

                        for (cmd, args, desc) in commands {
                            println!("  {}{:<10}{} {}{:<10}{} - {}{}{}",
                                active_c, cmd, reset,
                                subheader_c, args, reset,
                                body_c, desc, reset
                            );
                        }
                        println!("\n{}Usage Tips:{}", header_c, reset);
                        println!("  - Use 'aero update <zip>' to update from source.");
                        println!("  - Edit 'config.toml' to change prompt colors (hex codes supported!)");
                        println!();
                    },
                    cmd => {
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
                println!("Use 'exit' to quit.");
                continue;
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

fn open_config(config: &crate::config::RootConfig) {
    let editor = &config.config.editor;
    let config_path = get_config_path();

    println!("Opening config in {}...", editor);

    let _ = Command::new(editor)
        .arg(config_path)
        .status()
        .map_err(|e| eprintln!("Failed to open editor: {}", e));
}

fn update_aeroshell(zip_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting update process...");

    // 1. Create temp dir
    let temp_dir = env::temp_dir().join("aeroshell_update");
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir)?;
    }
    fs::create_dir(&temp_dir)?;

    println!("Extracting {} to {:?}...", zip_path, temp_dir);

    // 2. Unzip
    let file = fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    archive.extract(&temp_dir)?;

    // Find the source root
    let entries: Vec<PathBuf> = fs::read_dir(&temp_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect();

    let source_dir = if entries.len() == 1 && entries[0].is_dir() {
        entries[0].clone()
    } else {
        temp_dir.clone()
    };

    println!("Building new version in {:?}...", source_dir);

    // 3. Build
    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(&source_dir)
        .status()?;

    if !status.success() {
        return Err("Build failed".into());
    }

    // 4. Install
    let current_exe = env::current_exe()?;
    let new_binary = source_dir.join("target/release/aeroshell");

    if !new_binary.exists() {
        return Err("Built binary not found".into());
    }

    println!("Installing new binary to {:?}...", current_exe);

    let backup_path = current_exe.with_extension("old");
    fs::rename(&current_exe, &backup_path)?;

    if let Err(e) = fs::copy(&new_binary, &current_exe) {
        fs::rename(&backup_path, &current_exe)?;
        return Err(Box::new(e));
    }

    let _ = fs::remove_file(backup_path);
    let _ = fs::remove_dir_all(temp_dir);

    Ok(())
}
