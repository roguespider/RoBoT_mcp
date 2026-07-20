// src/cli/mod.rs
//! Command-line interface module

pub mod commands;
pub mod output;

use anyhow::Result;
use output::{section_header, bold, list_item};

/// Run the CLI with the given arguments
pub fn run() -> Result<()> {
    cli()
}

/// Main CLI entry point
fn cli() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }
    
    match args[1].as_str() {
        "server" => commands::server::run(),
        "init" => commands::init::run(),
        "status" => commands::status::run(),
        "memory" => commands::memory::run(&args[2..]),
        "experience" => commands::experience::run(),
        "config" => commands::config::run(),
        "migrate" => commands::migrate::run(),
        "help" | "-h" | "--help" => {
            print_usage();
            Ok(())
        }
        _ => {
            output::error_msg(format!("Unknown command: {}", args[1]));
            print_usage();
            std::process::exit(1);
        }
    }
}

/// Print CLI usage information
fn print_usage() {
    section_header("RoBoT MCP - Command Line Interface");
    
    println!("{}", bold("Usage:"));
    println!("  robot <command> [options]");
    println!();
    println!("{}", bold("Commands:"));
    list_item("server       - Start the MCP server");
    list_item("init         - Initialize the database");
    list_item("status       - Check system status");
    list_item("memory       - Memory management commands");
    list_item("experience   - Show experience statistics");
    list_item("config       - Show configuration");
    list_item("migrate      - Run database migrations");
    list_item("help         - Show this help message");
    println!();
    println!("{}", bold("Memory subcommands:"));
    list_item("memory list [limit]    - List memories");
    list_item("memory search <query>  - Search memories");
    list_item("memory add <content>  - Add a new memory");
    list_item("memory stats          - Show memory statistics");
}
