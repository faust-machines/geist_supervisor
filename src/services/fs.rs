use anyhow::{Context, Result};
use fs_extra::dir::copy as copy_dir;
use fs_extra::dir::CopyOptions;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile;
use tracing::info;

pub struct FileService {
    pub data_dir: PathBuf,
}

impl FileService {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    pub fn extract_bundle(&self, bundle_path: &Path, temp_dir: &Path) -> Result<PathBuf> {
        info!("Extracting release bundle...");
        let status = Command::new("tar")
            .args(["xzf", bundle_path.to_str().unwrap()])
            .current_dir(temp_dir)
            .output()
            .context("Failed to execute tar command")?;

        if !status.status.success() {
            let error = String::from_utf8_lossy(&status.stderr);
            return Err(anyhow::anyhow!(
                "Failed to extract release bundle: {}",
                error
            ));
        }

        Ok(temp_dir.join("release_bundle"))
    }

    pub fn extract_bundle_with_details(
        &self,
        bundle_path: &Path,
        temp_dir: &Path,
    ) -> Result<PathBuf> {
        info!("Extracting release bundle...");

        // Create the release bundle directory
        let release_bundle_dir = temp_dir.join("release_bundle");
        fs::create_dir_all(&release_bundle_dir)?;

        // Check if the bundle file exists and log its size
        if bundle_path.exists() {
            let metadata = fs::metadata(bundle_path)?;
            info!("Bundle file exists, size: {} bytes", metadata.len());
        } else {
            return Err(anyhow::anyhow!(
                "Bundle file does not exist: {}",
                bundle_path.display()
            ));
        }

        // List the contents of the tarball before extraction
        info!("Listing contents of the tarball:");
        let list_output = Command::new("tar").arg("-tvf").arg(bundle_path).output()?;

        if list_output.status.success() {
            let stdout = String::from_utf8_lossy(&list_output.stdout);
            info!("Tarball contents:\n{}", stdout);
        } else {
            let stderr = String::from_utf8_lossy(&list_output.stderr);
            info!("Failed to list tarball contents: {}", stderr);
        }

        // Extract the tarball directly to the release_bundle_dir
        info!("Extracting tarball to: {}", release_bundle_dir.display());
        let output = Command::new("tar")
            .arg("-xzf")
            .arg(bundle_path)
            .arg("-C")
            .arg(&release_bundle_dir)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to extract tarball: {}", stderr));
        }

        // List the contents of the extracted directory for debugging
        info!("Contents of release_bundle_dir:");
        self.walk_directory(&release_bundle_dir)?;

        Ok(release_bundle_dir)
    }

    // Helper function to walk directories and log contents
    fn walk_directory(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        if dir.exists() && dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                info!("  {}", path.display());
                files.push(path.clone());

                if path.is_dir() {
                    let subdir_files = FileService::walk_directory_static(path.as_path())?;
                    files.extend(subdir_files);
                }
            }
        }
        Ok(files)
    }

    // Static version of walk_directory to avoid self parameter
    fn walk_directory_static(dir: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        if dir.exists() && dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                info!("  {}", path.display());
                files.push(path.clone());

                if path.is_dir() {
                    let subdir_files = FileService::walk_directory_static(path.as_path())?;
                    files.extend(subdir_files);
                }
            }
        }
        Ok(files)
    }

    // Helper function to recursively copy directories
    pub fn copy_dir_all(&self, src: &Path, dst: &Path) -> Result<()> {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if ty.is_dir() {
                FileService::copy_dir_all_static(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
        }
        Ok(())
    }

    // Static version of copy_dir_all to avoid self parameter
    fn copy_dir_all_static(src: &Path, dst: &Path) -> Result<()> {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if ty.is_dir() {
                FileService::copy_dir_all_static(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
        }
        Ok(())
    }

    pub fn install_version(&self, release_bundle_dir: &Path, version: &str) -> Result<()> {
        // Find the binary and other required files in the extracted contents
        let found_files = self.walk_directory(release_bundle_dir)?;

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
        let binary_path = binary_path.unwrap_or_else(|| release_bundle_dir.join("roc_camera"));
        let manifest_path =
            manifest_path.unwrap_or_else(|| release_bundle_dir.join("manifest.yaml"));
        let assets_dir = assets_dir.unwrap_or_else(|| release_bundle_dir.join("roc_camera_app"));

        info!("Using binary path: {}", binary_path.display());
        info!("Using manifest path: {}", manifest_path.display());
        info!("Using assets directory: {}", assets_dir.display());

        // Check if the files exist
        if !binary_path.exists() {
            return Err(anyhow::anyhow!(
                "Binary path does not exist: {}",
                binary_path.display()
            ));
        }

        if !manifest_path.exists() {
            return Err(anyhow::anyhow!(
                "Manifest path does not exist: {}",
                manifest_path.display()
            ));
        }

        if !assets_dir.exists() {
            return Err(anyhow::anyhow!(
                "Assets directory does not exist: {}",
                assets_dir.display()
            ));
        }

        // Create version directory in data_dir
        let version_dir = self.data_dir.join(version);
        info!("Installing to version directory: {}", version_dir.display());

        // Remove existing directory if it exists to avoid "Directory not empty" error
        if version_dir.exists() {
            info!(
                "Removing existing version directory: {}",
                version_dir.display()
            );
            fs::remove_dir_all(&version_dir)?;
        }

        // Create the version directory
        fs::create_dir_all(&version_dir)?;

        // Copy files to the version directory
        let dest_binary = version_dir.join("roc_camera");
        let dest_manifest = version_dir.join("manifest.yaml");
        let dest_assets = version_dir.join("roc_camera_app");

        info!("Copying binary to: {}", dest_binary.display());
        fs::copy(&binary_path, &dest_binary)?;

        info!("Copying manifest to: {}", dest_manifest.display());
        fs::copy(&manifest_path, &dest_manifest)?;

        info!("Copying assets to: {}", dest_assets.display());
        self.copy_dir_all(&assets_dir, &dest_assets)?;

        info!("Successfully installed version: {}", version);

        Ok(())
    }

    pub fn update_files(&self, bundle_path: &Path) -> Result<()> {
        info!("Updating application files...");

        // Create temp directory for extraction
        let temp_dir = tempfile::tempdir()?;
        let release_dir = self.extract_bundle(bundle_path, temp_dir.path())?;

        // Update application files if they exist
        if Path::exists(&release_dir.join("roc_camera_app")) {
            let app_dir = self.data_dir.join("roc_camera_app");

            // Remove old app if exists
            fs::remove_dir_all(&app_dir).ok();

            // Create app directory
            fs::create_dir_all(&app_dir).context("Failed to create app directory")?;

            // Copy new app files
            let options = CopyOptions::new();
            copy_dir(release_dir.join("roc_camera_app"), &app_dir, &options)
                .context("Failed to update roc_camera_app")?;
            info!("Updated roc_camera_app at: {}", app_dir.display());
        }

        Ok(())
    }

    pub fn verify_permissions(&self) -> Result<()> {
        // Check if we have write permissions to data directory
        if !self.data_dir.exists() {
            fs::create_dir_all(&self.data_dir).context("Failed to create data directory")?;
        }

        // Try to write a test file
        let test_file = self.data_dir.join(".write_test");
        fs::write(&test_file, "test").context("No write permission in data directory")?;
        fs::remove_file(test_file).context("Failed to clean up test file")?;

        Ok(())
    }
}
