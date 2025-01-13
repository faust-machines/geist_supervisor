use anyhow::Result;
use clap::Parser;
pub mod cli;
pub mod config;
pub mod services;
pub mod utils;

use cli::Cli;

fn main() -> Result<()> {
    // Initialize logging
    utils::logging::init_logging();

    log::info!("Starting Geist Supervisor");
    let cli = Cli::parse();
    cli.execute()
}
