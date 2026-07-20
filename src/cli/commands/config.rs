// src/cli/commands/config.rs
//! Configuration display command

use anyhow::Result;
use crate::cli::output;

pub fn run() -> Result<()> {
    output::section_header("RoBoT Configuration");
    
    println!("{}", output::bold("Package:"));
    output::kv("Name", env!("CARGO_PKG_NAME"));
    output::kv("Version", env!("CARGO_PKG_VERSION"));
    output::kv("Description", env!("CARGO_PKG_DESCRIPTION"));
    println!();
    
    println!("{}", output::bold("Database:"));
    if let Ok(db) = crate::database::sqlite::SqliteDatabase::initialize() {
        output::success_msg("Database initialized");
        output::kv("Path", format!("{:?}", db.path()));
    } else {
        output::error_msg("Database not initialized");
    }
    println!();
    
    println!("{}", output::bold("Features:"));
    output::list_item(&format!("Experience System {}", output::green("✓")));
    output::list_item(&format!("Reflection Engine {}", output::green("✓")));
    output::list_item(&format!("Learning System {}", output::green("✓")));
    output::list_item(&format!("MCP Bridge {}", output::green("✓")));
    output::list_item(&format!("CLI Interface {}", output::green("✓")));
    
    Ok(())
}
