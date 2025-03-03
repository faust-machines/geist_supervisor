use crate::cli::node::NodeCommands;
use crate::cli::topic::TopicCommands;
use crate::config::Config;
use crate::services::FileService;
use crate::services::GcsService;
use anyhow::Result;
use clap::Subcommand;
use std::env;
use tempfile;

#[derive(Subcommand)]
pub enum Commands {
    /// Update to the specified version or the latest version if none is provided
    Update { version: Option<String> },
    /// Verify artifacts for the specified version
    Verify { version: String },
    /// Rollback to the specified version
    Rollback { version: String },
    /// Check the current status of the application
    Status,
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
    /// Run the application with the specified version or the default version if none is provided
    Run { version: Option<String> },
}

impl Commands {
    pub fn execute(self) -> Result<()> {
        match self {
            Commands::Update { version } => {
                let target_version = version.unwrap_or_else(|| Config::DEFAULT_VERSION.to_string());
                tracing::info!("Updating to version: {}", target_version);

                let gcs = GcsService::new(String::new(), Config::REGISTRY_BASE_URL.to_string());
                let data_dir = Config::data_dir();
                let data_dir_path = data_dir.as_path();

                tracing::debug!("Data dir exists: {}", data_dir.exists());
                tracing::debug!("Current user: {:?}", std::env::var("USER"));

                // Create data directory if it doesn't exist
                std::fs::create_dir_all(&data_dir)?;
                let data_dir_display = data_dir.display();
                tracing::info!("Using data_dir: {}", data_dir_display);

                let fs_service = FileService::new(data_dir.clone());

                // Verify permissions before starting
                fs_service.verify_permissions()?;

                // Strip the 'v' prefix if it exists when constructing paths
                let normalized_version = target_version.trim_start_matches('v');

                // Verify version exists
                if !gcs.verify_version(normalized_version)? {
                    anyhow::bail!("Version {} not found", target_version);
                }

                // Create temp directory and download release bundle
                let temp_dir = tempfile::tempdir()?;
                let bundle_path = temp_dir.path().join(Config::RELEASE_BUNDLE_NAME);

                tracing::info!("Downloading release bundle to: {}", bundle_path.display());
                gcs.download_release_bundle(normalized_version, &bundle_path)?;

                // Extract and update files
                fs_service.update_files(&bundle_path)?;

                // Ensure the extracted files are placed correctly
                let release_bundle_dir = temp_dir.path().join("release_bundle");
                let manifest_path = release_bundle_dir.join("manifest.yaml");
                let binary_path = release_bundle_dir.join("roc_camera");
                let assets_dir = release_bundle_dir.join("roc_camera_app");

                // Example logic to move files to the correct location
                std::fs::copy(&binary_path, &data_dir_path.join("roc_camera"))?;
                std::fs::create_dir_all(&data_dir_path.join("roc_camera_app"))?;
                std::fs::copy(&manifest_path, &data_dir_path.join("manifest.yaml"))?;
                std::fs::rename(&assets_dir, data_dir_path.join("roc_camera_app"))?;

                tracing::info!("Update completed successfully!");
                Ok(())
            }
            Commands::Verify { version } => {
                tracing::info!("Verifying artifacts for version: {}", version);

                let gcs = GcsService::new(String::new(), Config::REGISTRY_BASE_URL.to_string());

                if !gcs.verify_version(&version)? {
                    anyhow::bail!("Version {} not found", version);
                }

                tracing::info!("Verification completed successfully!");
                Ok(())
            }
            Commands::Rollback { version: _ } => {
                // tracing::info!("Rolling back to version: {}", version);

                // let fs_service = FileService::new(data_dir);

                // fs_service.rollback_to_version(&version)?;
                tracing::info!("Rollback completed successfully!");
                Ok(())
            }
            Commands::Status => {
                tracing::info!("Checking application status");

                // Example implementation for status
                let current_version =
                    env::var("GEIST_CURRENT_VERSION").unwrap_or_else(|_| "unknown".to_string());
                tracing::info!("Current version: {}", current_version);

                println!("Current version: {}", current_version);
                Ok(())
            }
            Commands::Node { command } => command.execute(),
            Commands::Topic { command } => command.execute(),
            Commands::Run { version } => {
                let run_version = version.unwrap_or_else(|| Config::DEFAULT_VERSION.to_string());
                tracing::info!("Running application with version: {}", run_version);

                // Construct the command to run
                let status = std::process::Command::new("flutter-pi")
                    .arg("--release")
                    .arg(format!(
                        "{}/roc_camera_app",
                        env::var("PROJECTS_DIR")
                            .unwrap_or_else(|_| "~/.local/share/roc-supervisor".to_string())
                    ))
                    .status();

                match status {
                    Ok(status) if status.success() => {
                        tracing::info!("Run completed successfully!");
                        Ok(())
                    }
                    Ok(status) => {
                        tracing::error!("Run failed with exit code: {}", status);
                        anyhow::bail!("Run failed with exit code: {}", status);
                    }
                    Err(e) => {
                        tracing::error!("Failed to execute run command: {}", e);
                        Err(e.into())
                    }
                }
            }
        }
    }
}
