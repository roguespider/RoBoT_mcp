// src/cli/commands/init.rs
//! Database initialization command

use anyhow::Result;
use crate::cli::output;

pub fn run() -> Result<()> {
    output::section_header("Initializing RoBoT Database");
    
    // Initialize database
    let db = crate::database::sqlite::SqliteDatabase::initialize()?;
    
    output::success_msg("Database initialized successfully");
    output::kv("Location", format!("{:?}", db.path()));
    
    println!();
    println!("{}", output::yellow("Database is ready for use"));
    
    Ok(())
}
