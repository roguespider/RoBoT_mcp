// src/cli/commands/migrate.rs
//! Database migration command

use anyhow::Result;
use crate::cli::output;

pub fn run() -> Result<()> {
    output::section_header("Running Database Migrations");
    
    let db = crate::database::sqlite::SqliteDatabase::initialize()?;
    
    // Run migrations
    output::info_msg("Applying migrations...");
    crate::database::migrations::run(&db)?;
    
    output::success_msg("All migrations completed successfully");
    
    println!();
    println!("{}", output::yellow("Database schema is up to date"));
    
    Ok(())
}
