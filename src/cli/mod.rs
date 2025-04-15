use anyhow::Result;
use clap::Parser;

pub mod commands;
pub mod node;
pub mod topic;

use commands::Commands;

#[derive(Parser)]
#[command(
    author,
    version,
    about = "Geist Supervisor CLI - Manage and control your Geist Camera",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        match self.command {
            Some(cmd) => cmd.execute(),
            None => {
                println!("No command specified. Use --help for usage information.");
                Ok(())
            }
        }
    }
}
