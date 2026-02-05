mod config;
mod prompt;
mod completer;

use std::process::{Command, Stdio};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::io::Read;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use sysinfo::{System, ProcessesToUpdate};

use crate::config::{load_config, get_config_path, RootConfig};
use crate::prompt::format_prompt;
use crate::completer::AeroCompleter;

use reedline::{
    Reedline, Signal, DefaultHinter,
    FileBackedHistory, Color
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
        "grey" | "gray" => nu_ansi_term::Style::new().fg(nu_ansi_term::Color::LightGray),
        _ => nu_ansi_term::Style::default(),
    }
}

// Convert config color to simple ANSI string
fn get_ansi_from_config(color_name: &str, config: &RootConfig) -> String {
    if let Some(hex) = config.colors.get(color_name) {
        if let Some((r, g, b)) = hex_to_rgb(hex) {
            return format!("\x1B[38;2;{};{};{}m", r, g, b);
        }
    }
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

fn cmd_ls(args: &[&str], config: &RootConfig) {
    let target = if args.is_empty() { "." } else { args[0] };

    // Read dir
    match fs::read_dir(target) {
        Ok(entries) => {
            let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
            // Sort by name
            entries.sort_by_key(|e| e.file_name());

            for entry in entries {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                let metadata = entry.metadata().ok();

                let mut color_key = String::from("default");

                if path.is_dir() {
                    color_key = String::from("directory");
                } else if let Some(m) = metadata {
                    let mut is_exe = false;
                    #[cfg(unix)]
                    {
                        if m.permissions().mode() & 0o111 != 0 {
                            is_exe = true;
                        }
                    }

                    if is_exe {
                        color_key = String::from("executable");
                    }

                    // Check extension dynamically against config
                    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                        let file_key = format!("file.{}", ext);
                        if config.theme.files.contains_key(&file_key) {
                            color_key = file_key;
                        } else if config.theme.files.contains_key(ext) {
                            color_key = ext.to_string();
                        } else {
                            // Legacy/Helper mappings
                            match ext {
                                "py" => if config.theme.files.contains_key("python") { color_key = String::from("python"); },
                                "sh" => if config.theme.files.contains_key("shellscript") { color_key = String::from("shellscript"); },
                                "rs" => if config.theme.files.contains_key("rust") { color_key = String::from("rust"); },
                                "js" => if config.theme.files.contains_key("javascript") { color_key = String::from("javascript"); },
                                _ => {}
                            }
                        }
                    }
                }

                // Lookup color name
                let file_color_name = config.theme.files.get(&color_key)
                    .or_else(|| config.theme.files.get("default"))
                    .map(|s| s.as_str())
                    .unwrap_or("white");

                let color_ansi = get_ansi_from_config(file_color_name, config);
                let reset = "\x1B[0m";

                print!("{}{}{}  ", color_ansi, name, reset);
            }
            println!();
        },
        Err(e) => eprintln!("ls: {}", e),
    }
}

fn cmd_proc(args: &[&str], config: &RootConfig) {
    let mut sys = System::new_all();
    sys.refresh_all();
    // We only refresh processes specifically? new_all refreshes everything once.
    // To be strictly up to date on subsequent calls we might need refresh, but this command runs once per invocation.

    let header_c = get_ansi_from_config(&config.theme.header, config);
    let subheader_c = get_ansi_from_config(&config.theme.subheader, config);
    let body_c = get_ansi_from_config(&config.theme.body, config);
    let active_c = get_ansi_from_config(&config.theme.active, config);
    let reset = "\x1B[0m";

    if args.is_empty() {
        println!("\n{}Process Monitor (proc):{}", header_c, reset);
        println!("{}", "=".repeat(30));

        let commands = [
            ("mem", "", "Show top memory consumers"),
            ("cpu", "", "Show top CPU consumers"),
            ("gpu", "", "Show GPU/System memory info"),
            ("<name>", "", "Search processes by name"),
        ];

        for (cmd, args, desc) in commands {
            println!("  {}{:<10}{} {}{:<10}{} - {}{}{}",
                active_c, cmd, reset,
                subheader_c, args, reset,
                body_c, desc, reset
            );
        }
        println!("\n{}Usage:{} proc [mem|cpu|gpu|<name>]", header_c, reset);
        return;
    }

    // Collect processes into a simplified struct for sorting/printing
    struct ProcInfo {
        pid: sysinfo::Pid,
        name: String,
        memory: u64, // bytes
        cpu: f32,    // usage %
    }

    let mut procs: Vec<ProcInfo> = sys.processes().iter().map(|(pid, p)| {
        ProcInfo {
            pid: *pid,
            name: p.name().to_string_lossy().to_string(),
            memory: p.memory(),
            cpu: p.cpu_usage(),
        }
    }).collect();

    match args[0] {
        "mem" => {
            println!("{}Top Memory Consumers:{}", header_c, reset);
            println!("{:<8} {:<25} {:>15}", "PID", "Name", "Memory (MB)");
            println!("{}", "=".repeat(50));

            procs.sort_by(|a, b| b.memory.cmp(&a.memory));
            for p in procs.iter().take(10) {
                let mem_mb = p.memory as f32 / 1024.0 / 1024.0;
                println!("{:<8} {}{:<25}{} {:>15.2}",
                    p.pid, subheader_c, p.name, reset, mem_mb);
            }
        },
        "cpu" => {
            println!("{}Top CPU Consumers:{}", header_c, reset);
            println!("{:<8} {:<25} {:>10}", "PID", "Name", "CPU %");
            println!("{}", "=".repeat(45));

            procs.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(std::cmp::Ordering::Equal));
            for p in procs.iter().take(10) {
                println!("{:<8} {}{:<25}{} {:>10.2}",
                    p.pid, subheader_c, p.name, reset, p.cpu);
            }
        },
        "gpu" => {
            // sysinfo doesn't support GPU usage directly.
            // On Apple Silicon (Asahi), GPU memory is unified.
            // We can perhaps show total system memory usage as a proxy or just list standard processes
            // emphasizing it's shared memory.
            println!("{}GPU/Unified Memory Info:{}", header_c, reset);
            println!("{}Note: Granular GPU process usage is not standardly available via sysinfo.{}", body_c, reset);
            println!("Showing total system memory usage (Shared):");

            let total_mem = sys.total_memory() as f32 / 1024.0 / 1024.0 / 1024.0; // GB
            let used_mem = sys.used_memory() as f32 / 1024.0 / 1024.0 / 1024.0;

            println!("  Total: {:.2} GB", total_mem);
            println!("  Used:  {:.2} GB", used_mem);
        },
        _ => {
            // Filter by name (comma separated)
            // Join all args to handle spaces if split by shell (though we use comma logic per request)
            // If user types "proc firefox, discord", args might be ["firefox,", "discord"] depending on shlex.
            // Let's rejoin and split by comma.
            let query = args.join(" ");
            let targets: Vec<&str> = query.split(',').map(|s| s.trim()).collect();

            println!("{}Searching processes for: {:?}{}", header_c, targets, reset);
            println!("{:<8} {:<25} {:>10} {:>15}", "PID", "Name", "CPU %", "Memory (MB)");
            println!("{}", "=".repeat(65));

            let matches: Vec<_> = procs.iter().filter(|p| {
                let name_lower = p.name.to_lowercase();
                targets.iter().any(|t| name_lower.contains(&t.to_lowercase()))
            }).collect();

            if matches.is_empty() {
                println!("{}No matching processes found.{}", body_c, reset);
            } else {
                for p in matches {
                    let mem_mb = p.memory as f32 / 1024.0 / 1024.0;
                    println!("{:<8} {}{:<25}{} {:>10.2} {:>15.2}",
                        p.pid, subheader_c, p.name, reset, p.cpu, mem_mb);
                }
            }
        }
    }
}

fn main() {
    ctrlc::set_handler(move || {}).expect("Error setting Ctrl-C handler");

    let mut config = load_config();

    // Export Env Vars
    for (name, hex) in &config.colors {
        env::set_var(format!("AERO_COLOR_{}", name.to_uppercase()), hex);
    }
    let t = &config.theme;
    env::set_var("AERO_THEME_AUTOCOMPLETE", &t.autocomplete);
    env::set_var("AERO_THEME_TYPING", &t.typing);
    env::set_var("AERO_THEME_TYPINGTEXT", &t.typingtext);
    env::set_var("AERO_THEME_HEADER", &t.header);
    env::set_var("AERO_THEME_SUBHEADER", &t.subheader);
    env::set_var("AERO_THEME_BODY", &t.body);
    env::set_var("AERO_THEME_ACTIVE", &t.active);
    env::set_var("AERO_THEME_DISABLE", &t.disable);

    let history_path = env::var("HOME")
        .map(|h| format!("{}/.aeroshell_history", h))
        .unwrap_or_else(|_| ".aeroshell_history".to_string());

    let history = Box::new(
        FileBackedHistory::with_file(2000, history_path.into())
            .expect("Error configuring history with file"),
    );

    let hint_style = get_style_from_config(&config.theme.autocomplete, &config);

    let mut line_editor = Reedline::create()
        .with_history(history)
        .with_hinter(Box::new(DefaultHinter::default().with_style(hint_style)))
        .with_completer(Box::new(AeroCompleter));

    loop {
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
                    "ls" => {
                        cmd_ls(&args, &config);
                    },
                    "proc" => {
                        cmd_proc(&args, &config);
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
                         let header_c = get_ansi_from_config(&config.theme.header, &config);
                         let subheader_c = get_ansi_from_config(&config.theme.subheader, &config);
                         let body_c = get_ansi_from_config(&config.theme.body, &config);
                         let active_c = get_ansi_from_config(&config.theme.active, &config);
                         let err_c = get_ansi_from_config(&config.theme.disable, &config);
                         let reset = "\x1B[0m";

                         if args.is_empty() {
                            println!("\n{}AeroShell Manager (aero):{}", header_c, reset);
                            println!("{}", "=".repeat(30));

                            let commands = [
                                ("config", "", "Open configuration in editor"),
                                ("setdefault", "", "Set AeroShell as default shell"),
                                ("update", "<zipfile>", "Update AeroShell from a source zip"),
                            ];

                            for (cmd, args, desc) in commands {
                                println!("  {}{:<10}{} {}{:<10}{} - {}{}{}",
                                    active_c, cmd, reset,
                                    subheader_c, args, reset,
                                    body_c, desc, reset
                                );
                            }
                            println!("\n{}Usage:{} aero <command>", header_c, reset);
                        } else {
                            match args[0] {
                                "config" => {
                                    open_config(&config);
                                    config = load_config();
                                },
                                "setdefault" => {
                                    println!("{}Setting AeroShell as default shell...{}", header_c, reset);
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
                                            println!("{}Adding {} to /etc/shells (requires sudo)...{}", body_c, path_str, reset);
                                            let status = Command::new("sudo")
                                                .arg("sh")
                                                .arg("-c")
                                                .arg(format!("echo '{}' >> /etc/shells", path_str))
                                                .status();

                                            if let Ok(s) = status {
                                                if !s.success() {
                                                    eprintln!("{}Failed to add to /etc/shells. Aborting.{}", err_c, reset);
                                                    continue;
                                                }
                                            } else {
                                                eprintln!("{}Failed to run sudo. Aborting.{}", err_c, reset);
                                                continue;
                                            }
                                        }

                                        println!("{}Changing shell (requires password)...{}", body_c, reset);
                                        let status = Command::new("chsh")
                                            .arg("-s")
                                            .arg(&exe_path)
                                            .status();

                                        match status {
                                            Ok(s) if s.success() => println!("{}Success! Please log out and back in.{}", active_c, reset),
                                            _ => println!("{}Failed to set default shell.{}", err_c, reset),
                                        }
                                    }
                                },
                                "update" if args.len() > 1 => {
                                    let zip_path = args[1];
                                    if let Err(e) = update_aeroshell(zip_path) {
                                        eprintln!("{}Update failed: {}{}", err_c, e, reset);
                                    } else {
                                        println!("{}Update successful! Restart AeroShell to see changes.{}", active_c, reset);
                                    }
                                },
                                _ => println!("{}Unknown aero command: {}{}", err_c, args[0], reset),
                            }
                        }
                    },
                    "help" => {
                        let header_c = get_ansi_from_config(&config.theme.header, &config);
                        let subheader_c = get_ansi_from_config(&config.theme.subheader, &config);
                        let body_c = get_ansi_from_config(&config.theme.body, &config);
                        let active_c = get_ansi_from_config(&config.theme.active, &config);
                        let reset = "\x1B[0m";

                        println!("\n{}AeroShell Built-in Commands:{}", header_c, reset);
                        println!("{}", "=".repeat(30));

                        let commands = [
                            ("cd", "<dir>", "Change directory"),
                            ("ls", "[dir]", "List files (colored)"),
                            ("proc", "[mem|cpu|name]", "Process monitor"), // Added description
                            ("exit", "", "Exit shell"),
                            ("clear", "", "Clear screen"),
                            ("config", "", "Open configuration"),
                            ("aero", "<cmd>", "Manage AeroShell"),
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
                        println!("  - Use 'proc mem' to check memory usage.");
                        println!("  - Use 'aero update <zip>' to update from source.");
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

    let temp_dir = env::temp_dir().join("aeroshell_update");
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir)?;
    }
    fs::create_dir(&temp_dir)?;

    println!("Extracting {} to {:?}...", zip_path, temp_dir);

    let file = fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    archive.extract(&temp_dir)?;

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

    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(&source_dir)
        .status()?;

    if !status.success() {
        return Err("Build failed".into());
    }

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
