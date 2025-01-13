use anyhow::Result;
use clap::Parser;

pub mod commands;
pub mod node;
pub mod topic;

#[derive(Parser)]
#[command(
    author,
    version,
    about = "Roc Supervisor CLI - Manage and control your Roc Camera",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    command: commands::Commands,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        self.command.execute()
    }
}
