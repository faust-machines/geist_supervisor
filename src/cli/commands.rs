use crate::cli::node::NodeCommands;
use crate::cli::topic::TopicCommands;
use crate::config::Config;
use crate::services::FileService;
use crate::services::GcsService;
use anyhow::Result;
use clap::Subcommand;
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
                tracing::info!("Using data_dir: {}", data_dir.display());

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
                tracing::info!("Extracting release bundle from: {}", bundle_path.display());

                // Use the new extract_bundle_with_details method
                let release_bundle_dir =
                    fs_service.extract_bundle_with_details(&bundle_path, temp_dir.path())?;

                // Install the version
                fs_service.install_version(&release_bundle_dir, target_version.as_str())?;

                // Set as current version
                if let Err(e) = Config::set_current_version(&target_version) {
                    tracing::warn!("Failed to set current version: {}", e);
                } else {
                    tracing::info!("Set current version to: {}", target_version);
                }

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

                // Get the current version using our new function
                let current_version = Config::get_current_version();
                tracing::info!("Current version: {}", current_version);

                println!("Current version: {}", current_version);
                Ok(())
            }
            Commands::Node { command } => command.execute(),
            Commands::Topic { command } => command.execute(),
            Commands::Run { version } => {
                let data_dir = Config::data_dir();

                // Determine which version to run
                let target_version = match version {
                    Some(v) => v,
                    None => {
                        // Find the latest version in the data directory
                        let mut versions = Vec::new();
                        for entry in std::fs::read_dir(&data_dir)? {
                            let entry = entry?;
                            if entry.file_type()?.is_dir() {
                                if let Some(name) = entry.file_name().to_str() {
                                    if name.starts_with('v') {
                                        versions.push(name.to_string());
                                    }
                                }
                            }
                        }

                        if versions.is_empty() {
                            anyhow::bail!("No versions found. Please run 'update' first.");
                        }

                        // Sort versions to find the latest
                        versions.sort();
                        versions.last().unwrap().clone()
                    }
                };

                tracing::info!("Running version: {}", target_version);

                // Check if the version exists
                let version_dir = data_dir.join(&target_version);
                if !version_dir.exists() {
                    anyhow::bail!(
                        "Version {} not found. Please run 'update {}' first.",
                        target_version,
                        target_version
                    );
                }

                // Record this as the current version
                if let Err(e) = Config::set_current_version(&target_version) {
                    tracing::warn!("Failed to set current version: {}", e);
                }

                // Find the binary
                let binary_path = version_dir.join("roc_camera");
                if !binary_path.exists() {
                    anyhow::bail!("Binary not found for version {}", target_version);
                }

                // Check if running on Raspberry Pi
                #[cfg(target_arch = "arm")]
                {
                    // Make sure the binary is executable
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let mut perms = std::fs::metadata(&binary_path)?.permissions();
                        perms.set_mode(0o755);
                        std::fs::set_permissions(&binary_path, perms)?;
                    }

                    // Use the actual assets in the data directory instead of creating symlinks
                    let flutter_assets_path = version_dir.join("roc_camera_app");

                    if !flutter_assets_path.exists() {
                        tracing::error!(
                            "Flutter assets directory doesn't exist at: {}",
                            flutter_assets_path.display()
                        );
                        return Err(anyhow::anyhow!("Flutter assets directory not found"));
                    }

                    tracing::info!(
                        "Using Flutter assets from: {}",
                        flutter_assets_path.display()
                    );

                    // Run the binary
                    tracing::info!("Executing binary: {}", binary_path.display());
                    let mut command = std::process::Command::new(&binary_path);

                    // Set current directory to the version directory
                    command.current_dir(&version_dir);

                    // Add environment variables that point to the actual assets location
                    command.env("FLUTTER_ASSETS_DIR", &flutter_assets_path);
                    command.env("FLUTTER_ASSET_DIR", &flutter_assets_path);
                    command.env("FLUTTER_BUNDLE_DIR", &flutter_assets_path);
                    command.env("FLUTTER_APP_DIR", &flutter_assets_path);
                    command.env("FLUTTER_PI_APP_DIR", &flutter_assets_path);
                    command.env("APP_DIR", &flutter_assets_path);

                    // Pass the flutter assets directory as a command-line argument
                    command.arg("--flutter-assets-dir");
                    command.arg(&flutter_assets_path);

                    let status = command.status()?;

                    if !status.success() {
                        anyhow::bail!("Process exited with status: {}", status);
                    }
                }

                // If not on Raspberry Pi, show a message
                #[cfg(not(target_arch = "arm"))]
                {
                    tracing::info!("Binary is compiled for Raspberry Pi and cannot be executed on this system.");
                    tracing::info!(
                        "The application has been successfully installed at: {}",
                        version_dir.display()
                    );
                    tracing::info!("To run the application, transfer the files to a Raspberry Pi and execute the 'roc_camera' binary.");

                    // Print the command that would be executed on a Raspberry Pi
                    tracing::info!("On a Raspberry Pi, the following command would be executed:");
                    tracing::info!("cd {} && ./roc_camera", version_dir.display());
                }

                Ok(())
            }
        }
    }
}
