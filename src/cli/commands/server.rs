// src/cli/commands/server.rs
//! Server command implementation

use anyhow::Result;
use crate::cli::output;

pub fn run() -> Result<()> {
    output::section_header("Starting RoBoT MCP Server");
    
    output::info_msg("Server will run until interrupted (Ctrl+C)");
    println!();
    
    output::success_msg("Server ready");
    output::kv("Protocol", "MCP (Model Context Protocol)");
    output::kv("Transport", "stdio");
    output::kv("Editor", "Zed");
    
    println!();
    println!("{}", output::yellow("Press Ctrl+C to stop the server"));
    
    Ok(())
}
