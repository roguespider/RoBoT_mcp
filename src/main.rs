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

use app::App;
use logging::init_logging;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging();

    App::new().await?.run().await?;

    Ok(())
}

