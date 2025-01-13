use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum NodeCommands {
    /// Start a specific node
    Start {
        /// Name of the node to start
        #[arg(value_name = "NODE_NAME")]
        name: String,
    },
    /// Stop a specific node
    Stop {
        /// Name of the node to stop
        #[arg(value_name = "NODE_NAME")]
        name: String,
    },
    /// List all available nodes
    List,
}

impl NodeCommands {
    pub fn execute(self) -> Result<()> {
        match self {
            NodeCommands::Start { name } => {
                println!("Starting node: {}", name);
                Ok(())
            }
            NodeCommands::Stop { name } => {
                println!("Stopping node: {}", name);
                Ok(())
            }
            NodeCommands::List => {
                println!("Listing nodes");
                Ok(())
            }
        }
    }
}
