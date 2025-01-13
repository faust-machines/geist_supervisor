use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum TopicCommands {
    /// List all available topics
    List,
    /// Echo messages from a specific topic
    Echo {
        /// Name of the topic to echo
        #[arg(value_name = "TOPIC_NAME")]
        name: String,
    },
}

impl TopicCommands {
    pub fn execute(self) -> Result<()> {
        match self {
            TopicCommands::List => {
                println!("Listing topics");
                Ok(())
            }
            TopicCommands::Echo { name } => {
                println!("Echoing topic: {}", name);
                Ok(())
            }
        }
    }
}
