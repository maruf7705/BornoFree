#![windows_subsystem = "windows"]

mod gui;
mod windows_hook;

use borno_core::{
    BornoEngine, EngineAction, InputMode, PhoneticEngine,
};
use clap::{Parser, Subcommand, ValueEnum};
use colored::*;
use rustyline::DefaultEditor;
use std::io;

#[derive(Parser)]
#[command(name = "BornoFree")]
#[command(author = "BornoFree Contributors")]
#[command(version = "1.0.0")]
#[command(about = "BornoFree: High-Performance Bengali Typing Engine for Windows", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the native floating toolbar and background typing engine (default)
    Run,
    /// Interactive terminal typing REPL
    Repl {
        #[arg(short, long, default_value = "borno-unicode")]
        mode: CliMode,
    },
    /// Transliterate phonetic English into Bengali
    Phonetic {
        /// English phonetic text to transliterate
        text: String,
    },
    /// Show layout status and active configuration
    Info,
}

#[derive(Clone, Copy, ValueEnum)]
enum CliMode {
    English,
    BornoUnicode,
    Phonetic,
}

fn main() -> io::Result<()> {
    let cli = Cli::try_parse().unwrap_or(Cli { command: Some(Commands::Run) });
    let command = cli.command.unwrap_or(Commands::Run);

    match command {
        Commands::Run => {
            #[cfg(target_os = "windows")]
            {
                windows_hook::install_keyboard_hook();
                gui::launch_gui_and_message_loop();
            }
            #[cfg(not(target_os = "windows"))]
            {
                println!("System-wide hook is currently enabled for Windows.");
            }
        }
        Commands::Info => {
            println!("{}", "=== BornoFree Engine Information ===".bold().cyan());
            println!("Version: {}", "1.0.0".green());
            println!("Platform: Cross-Platform (Windows, Linux, macOS)");
            println!("Supported Layouts:");
            println!("  - Borno Unicode (Ctrl+Alt+V)");
            println!("  - Avro Phonetic");
            println!("  - English (Ctrl+Alt+E / F12 Toggle)");
        }
        Commands::Phonetic { text } => {
            let engine = PhoneticEngine::new();
            let result = engine.transliterate(&text);
            println!("{}", result.bold().green());
        }
        Commands::Repl { mode } => {
            let mut engine = BornoEngine::new();
            let input_mode = match mode {
                CliMode::English => InputMode::English,
                CliMode::BornoUnicode => InputMode::BornoUnicode,
                CliMode::Phonetic => InputMode::Phonetic,
            };
            engine.set_mode(input_mode);

            println!("{}", "=== BornoFree Interactive REPL ===".bold().cyan());
            println!("Active Mode: {}", input_mode.name().yellow());
            println!("Type text and press Enter. Press Ctrl+D or Ctrl+C to exit.\n");

            let mut rl = DefaultEditor::new().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

            loop {
                let prompt = format!("{}> ", engine.mode.badge().magenta());
                match rl.readline(&prompt) {
                    Ok(line) => {
                        let _ = rl.add_history_entry(line.as_str());

                        if engine.mode == InputMode::Phonetic {
                            let phonetic = PhoneticEngine::new();
                            println!("=> {}", phonetic.transliterate(&line).green());
                        } else {
                            let mut converted = String::new();
                            for c in line.chars() {
                                let shift = c.is_ascii_uppercase();
                                match engine.handle_key(c, shift, false, false) {
                                    EngineAction::Emit(s) => converted.push_str(&s),
                                    EngineAction::Replace { backspaces, text } => {
                                        for _ in 0..backspaces {
                                            converted.pop();
                                        }
                                        converted.push_str(&text);
                                    }
                                    EngineAction::Passthrough => converted.push(c),
                                    EngineAction::ModeSwitch(_) => {}
                                }
                            }
                            println!("=> {}", converted.green());
                        }
                    }
                    Err(_) => break,
                }
            }
        }
    }

    Ok(())
}
