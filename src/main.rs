use anyhow::Result;
use clap::Parser;
use std::env;
pub mod cli;
pub mod config;
pub mod services;
pub mod utils;

use cli::Cli;

fn main() -> Result<()> {
    // Set default log level if not already set
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }

    // Initialize logging
    utils::logging::init_logging();

    tracing::info!("Starting Roc Supervisor");
    let cli = Cli::parse();
    cli.execute()
}
