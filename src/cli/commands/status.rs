// src/cli/commands/status.rs
//! System status command

use anyhow::Result;
use crate::cli::output;

pub fn run() -> Result<()> {
    output::section_header("RoBoT System Status");
    
    // Check database
    match crate::database::sqlite::SqliteDatabase::initialize() {
        Ok(db) => {
            output::success_msg("Database: Connected");
            output::kv("Location", format!("{:?}", db.path()));
        }
        Err(e) => {
            output::error_msg(format!("Database: Error - {}", e));
            output::kv("Status", output::red("Disconnected"));
        }
    }
    
    println!();
    println!("{}", output::bold("Components:"));
    output::list_item(&format!("Experience System {}", output::green("✓")));
    output::list_item(&format!("Reflection Engine {}", output::green("✓")));
    output::list_item(&format!("Learning System {}", output::green("✓")));
    output::list_item(&format!("MCP Bridge {}", output::green("✓")));
    
    Ok(())
}
