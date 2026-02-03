mod config;
mod prompt;
mod completer;

use std::process::{Command, Stdio};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::io::Read;
use crate::config::{load_config, get_config_path};
use crate::prompt::format_prompt;
use crate::completer::AeroCompleter;

use reedline::{
    Reedline, Signal, DefaultHinter,
    FileBackedHistory
};

fn main() {
    // 0. Setup Global Signal Handler
    ctrlc::set_handler(move || {
        // Do nothing.
    }).expect("Error setting Ctrl-C handler");

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
                // Convert to Vec<&str> for Command args
                let args: Vec<&str> = parts[1..].iter().map(|s| s.as_str()).collect();

                // 4. Execute Command
                match command.as_str() {
                    "cd" => {
                        let new_dir = if args.is_empty() {
                            env::var("HOME").unwrap_or_else(|_| "/".to_string())
                        } else {
                            // Tilde Expansion
                            if args[0].starts_with("~") {
                                let home = env::var("HOME").unwrap_or_else(|_| "/".to_string());
                                if args[0] == "~" {
                                    home
                                } else {
                                    // replace leading ~ with home
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
                        // Just open the config file
                        open_config(&config);
                        config = load_config(); // Reload after edit
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

                                        // 1. Check /etc/shells
                                        let mut needs_add = true;
                                        if let Ok(mut file) = fs::File::open("/etc/shells") {
                                            let mut contents = String::new();
                                            if file.read_to_string(&mut contents).is_ok() {
                                                if contents.lines().any(|line| line.trim() == path_str) {
                                                    needs_add = false;
                                                }
                                            }
                                        }

                                        // 2. Add to /etc/shells if needed
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

                                        // 3. Run chsh
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
                        // More colorful help menu
                        let title_style = "\x1B[1;36m"; // Bold Cyan
                        let header_style = "\x1B[1;33m"; // Bold Yellow
                        let cmd_style = "\x1B[32m";     // Green
                        let arg_style = "\x1B[35m";     // Magenta
                        let desc_style = "\x1B[0m";     // Reset
                        let reset = "\x1B[0m";

                        println!("\n{}AeroShell Built-in Commands:{}", title_style, reset);
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
                                cmd_style, cmd, reset,
                                arg_style, args, reset,
                                desc_style, desc, reset
                            );
                        }
                        println!("\n{}Usage Tips:{}", header_style, reset);
                        println!("  - Use 'aero update <zip>' to update from source.");
                        println!("  - Edit 'config.toml' to change prompt colors (hex codes supported!)");
                        println!();
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

fn open_config(config: &crate::config::Config) {
    let editor = &config.editor;
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

    // Find the source root (might be nested in a folder like aeroshell-main)
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

    // 4. Install (Overwrite current binary)
    // We need to know where we are installed.
    // Assuming ~/.aeroshell/as or we can use env::current_exe
    let current_exe = env::current_exe()?;
    let new_binary = source_dir.join("target/release/aeroshell");

    if !new_binary.exists() {
        return Err("Built binary not found".into());
    }

    println!("Installing new binary to {:?}...", current_exe);

    // Replacing a running binary can be tricky on some OSs (Windows), but on Linux/Mac usually fine
    // or requires a mv trick.
    // Try copy first.
    // Rename current to .old just in case
    let backup_path = current_exe.with_extension("old");
    fs::rename(&current_exe, &backup_path)?;

    if let Err(e) = fs::copy(&new_binary, &current_exe) {
        // Rollback
        fs::rename(&backup_path, &current_exe)?;
        return Err(Box::new(e));
    }

    // Cleanup
    let _ = fs::remove_file(backup_path); // Delete backup if successful
    let _ = fs::remove_dir_all(temp_dir); // Cleanup temp

    Ok(())
}
