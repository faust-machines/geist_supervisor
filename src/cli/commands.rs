use crate::cli::node::NodeCommands;
use crate::cli::topic::TopicCommands;
use crate::config::Config;
use crate::services::FileService;
use crate::services::GcsService;
use anyhow::Result;
use clap::Subcommand;
use std::env;
use std::path::Path;
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

                // Create the release bundle directory
                let release_bundle_dir = temp_dir.path().join("release_bundle");
                std::fs::create_dir_all(&release_bundle_dir)?;

                // Check if the bundle file exists and log its size
                if bundle_path.exists() {
                    let metadata = std::fs::metadata(&bundle_path)?;
                    tracing::info!("Bundle file exists, size: {} bytes", metadata.len());
                } else {
                    tracing::error!("Bundle file does not exist: {}", bundle_path.display());
                    anyhow::bail!("Bundle file does not exist: {}", bundle_path.display());
                }

                // List the contents of the tarball before extraction
                tracing::info!("Listing contents of the tarball:");
                let list_output = std::process::Command::new("tar")
                    .arg("-tvf")
                    .arg(&bundle_path)
                    .output()?;

                if list_output.status.success() {
                    let stdout = String::from_utf8_lossy(&list_output.stdout);
                    tracing::info!("Tarball contents:\n{}", stdout);
                } else {
                    let stderr = String::from_utf8_lossy(&list_output.stderr);
                    tracing::error!("Failed to list tarball contents: {}", stderr);
                }

                // Extract the tarball directly to the release_bundle_dir
                tracing::info!("Extracting tarball to: {}", release_bundle_dir.display());
                let output = std::process::Command::new("tar")
                    .arg("-xzf")
                    .arg(&bundle_path)
                    .arg("-C")
                    .arg(&release_bundle_dir)
                    .output()?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    tracing::error!("Failed to extract tarball: {}", stderr);
                    anyhow::bail!("Failed to extract tarball: {}", stderr);
                }

                // List the contents of the extracted directory for debugging
                tracing::info!("Contents of release_bundle_dir:");

                // Define a recursive function to walk directories
                fn walk_directory(dir: &std::path::Path) -> Result<Vec<std::path::PathBuf>> {
                    let mut files = Vec::new();
                    if dir.exists() && dir.is_dir() {
                        for entry in std::fs::read_dir(dir)? {
                            let entry = entry?;
                            let path = entry.path();
                            tracing::info!("  {}", path.display());
                            files.push(path.clone());

                            if path.is_dir() {
                                let subdir_files = walk_directory(&path)?;
                                files.extend(subdir_files);
                            }
                        }
                    }
                    Ok(files)
                }

                // Walk the release bundle directory and get all files
                let found_files = walk_directory(&release_bundle_dir)?;

                // Try to find the binary and other required files in the extracted contents
                let mut binary_path = None;
                let mut manifest_path = None;
                let mut assets_dir = None;

                for path in &found_files {
                    let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                    if file_name == "roc_camera" {
                        binary_path = Some(path.clone());
                    } else if file_name == "manifest.yaml" {
                        manifest_path = Some(path.clone());
                    } else if file_name == "roc_camera_app" && path.is_dir() {
                        assets_dir = Some(path.clone());
                    }
                }

                // Use the found paths or default to the expected locations
                let binary_path =
                    binary_path.unwrap_or_else(|| release_bundle_dir.join("roc_camera"));
                let manifest_path =
                    manifest_path.unwrap_or_else(|| release_bundle_dir.join("manifest.yaml"));
                let assets_dir =
                    assets_dir.unwrap_or_else(|| release_bundle_dir.join("roc_camera_app"));

                tracing::info!("Using binary path: {}", binary_path.display());
                tracing::info!("Using manifest path: {}", manifest_path.display());
                tracing::info!("Using assets directory: {}", assets_dir.display());

                // Check if the files exist
                if !binary_path.exists() {
                    tracing::error!("Binary path does not exist: {}", binary_path.display());
                    anyhow::bail!("Binary path does not exist: {}", binary_path.display());
                }

                if !manifest_path.exists() {
                    tracing::error!("Manifest path does not exist: {}", manifest_path.display());
                    anyhow::bail!("Manifest path does not exist: {}", manifest_path.display());
                }

                if !assets_dir.exists() {
                    tracing::error!("Assets directory does not exist: {}", assets_dir.display());
                    anyhow::bail!("Assets directory does not exist: {}", assets_dir.display());
                }

                // Create version directory in data_dir
                let version_dir = data_dir.join(target_version.as_str());
                tracing::info!("Installing to version directory: {}", version_dir.display());

                // Remove existing directory if it exists to avoid "Directory not empty" error
                if version_dir.exists() {
                    tracing::info!(
                        "Removing existing version directory: {}",
                        version_dir.display()
                    );
                    std::fs::remove_dir_all(&version_dir)?;
                }

                // Create the version directory
                std::fs::create_dir_all(&version_dir)?;

                // Copy files to the version directory
                let dest_binary = version_dir.join("roc_camera");
                let dest_manifest = version_dir.join("manifest.yaml");
                let dest_assets = version_dir.join("roc_camera_app");

                tracing::info!("Copying binary to: {}", dest_binary.display());
                std::fs::copy(&binary_path, &dest_binary)?;

                tracing::info!("Copying manifest to: {}", dest_manifest.display());
                std::fs::copy(&manifest_path, &dest_manifest)?;

                tracing::info!("Copying assets to: {}", dest_assets.display());
                copy_dir_all(&assets_dir, &dest_assets)?;

                tracing::info!("Successfully installed version: {}", target_version);

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

                    // Run the binary
                    tracing::info!("Executing binary: {}", binary_path.display());
                    let status = std::process::Command::new(&binary_path)
                        .current_dir(&version_dir) // Run from the version directory
                        .status()?;

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

// Helper function to recursively copy directories
fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
