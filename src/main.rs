// src/main.rs

mod database;
mod experience;
mod bridge;
mod tools;
mod planner;
mod skills;
mod workflows;
mod learning;
mod knowledge;
mod memory;
mod cli;

use bridge::app::App;
use bridge::logging::init_logging;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // On Windows, attach to parent console if running without one
    // This fixes issues with GUI applications (like Zed Editor) that spawn
    // subprocesses without a console, causing stdio to fail
    #[cfg(target_os = "windows")]
    {
        bridge::windows_console::attach_console();
    }
    
    init_logging();

    // Check if CLI mode is requested
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "server" => {
                App::new().await?.run().await?;
            }
            _ => {
                // Run CLI commands
                cli::run()?;
            }
        }
    } else {
        // Default: run as MCP server
        App::new().await?.run().await?;
    }

    Ok(())
}

