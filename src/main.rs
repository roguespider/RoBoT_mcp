// src/main.rs

mod app;
mod logging;

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

use app::App;
use logging::init_logging;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

