use crate::cli::node::NodeCommands;
use crate::cli::topic::TopicCommands;
use crate::services::FileService;
use crate::services::GitHubService;
use anyhow::Context;
use anyhow::Result;
use clap::Subcommand;
use std::env;
use std::path::PathBuf;
use tempfile;

#[derive(Subcommand)]
pub enum Commands {
    /// Update all components to the specified version
    UpdateAll { version: String },
    /// Verify artifacts for the specified version
    Verify { version: String },
    /// Rollback to the specified version
    Rollback { version: String },
    /// Update the supervisor itself to the specified version
    UpdateSelf { version: String },
    /// Delegate to node command implementation
    Node {
        #[command(subcommand)]
        command: NodeCommands,
    },
    /// Delegate to topic command implementation
    Topic {
        #[command(subcommand)]
        command: TopicCommands,
    },
}

impl Commands {
    pub fn execute(self) -> Result<()> {
        match self {
            Commands::UpdateAll { version } => {
                tracing::info!("Updating to version: {}", version);

                // Initialize services
                let github_token = std::env::var("GITHUB_TOKEN")
                    .context("GITHUB_TOKEN environment variable not set")?;
                let github = GitHubService::new(github_token);

                // Get paths from environment variables or use defaults
                let bin_dir = env::var("GEIST_BIN_DIR")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| PathBuf::from("/usr/local/bin"));

                let app_dir = env::var("GEIST_APP_DIR")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| PathBuf::from("/opt/roc_camera_app"));

                tracing::info!("Using bin_dir: {}", bin_dir.display());
                tracing::info!("Using app_dir: {}", app_dir.display());

                let fs_service = FileService::new(bin_dir, app_dir);

                // Verify permissions before starting
                fs_service.verify_permissions()?;

                // Verify release exists
                if !github.verify_release(&version)? {
                    anyhow::bail!("Release {} not found", version);
                }

                // Create temp directory and download bundle
                let temp_dir = tempfile::tempdir()?;
                let bundle_path = temp_dir
                    .path()
                    .join(format!("release_bundle-{}.tar.gz", version));

                tracing::info!("Downloading release bundle to: {}", bundle_path.display());
                github.download_release_bundle(&version, &bundle_path)?;

                // Extract and update
                let release_dir = fs_service.extract_bundle(&bundle_path, temp_dir.path())?;
                fs_service.update_binaries(&release_dir)?;
                fs_service.update_app(&release_dir)?;

                tracing::info!("Update completed successfully!");
                Ok(())
            }
            Commands::Verify { version } => {
                println!("Verifying artifacts for version: {}", version);
                Ok(())
            }
            Commands::Rollback { version } => {
                println!("Rolling back to version: {}", version);
                Ok(())
            }
            Commands::UpdateSelf { version } => {
                println!("Updating supervisor itself to version: {}", version);
                Ok(())
            }
            Commands::Node { command } => command.execute(),
            Commands::Topic { command } => command.execute(),
        }
    }
}
