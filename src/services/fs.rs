use anyhow::{Context, Result};
use fs_extra::dir::copy as copy_dir;
use fs_extra::dir::CopyOptions;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::info;

pub struct FileService {
    pub bin_dir: PathBuf,
    pub app_dir: PathBuf,
}

impl FileService {
    pub fn new(bin_dir: PathBuf, app_dir: PathBuf) -> Self {
        Self { bin_dir, app_dir }
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

    pub fn update_binaries(&self, release_dir: &Path) -> Result<()> {
        info!("Updating binaries...");

        // Update roc_camera binary
        if Path::exists(&release_dir.join("roc_camera")) {
            let target_path = self.bin_dir.join("roc_camera");
            fs::copy(release_dir.join("roc_camera"), &target_path)
                .context("Failed to update roc_camera binary")?;
            info!("Updated roc_camera binary at: {}", target_path.display());
        }

        // Update geist-supervisor binary
        if Path::exists(&release_dir.join("geist-supervisor")) {
            let target_path = self.bin_dir.join("geist-supervisor");
            fs::copy(release_dir.join("geist-supervisor"), &target_path)
                .context("Failed to update geist-supervisor binary")?;
            info!(
                "Updated geist-supervisor binary at: {}",
                target_path.display()
            );
        }

        Ok(())
    }

    pub fn update_app(&self, release_dir: &Path) -> Result<()> {
        info!("Updating application files...");

        if Path::exists(&release_dir.join("roc_camera_app")) {
            // Remove old app if exists
            fs::remove_dir_all(&self.app_dir).ok();

            // Create app directory
            fs::create_dir_all(&self.app_dir).context("Failed to create app directory")?;

            // Copy new app files
            let options = CopyOptions::new();
            copy_dir(release_dir.join("roc_camera_app"), &self.app_dir, &options)
                .context("Failed to update roc_camera_app")?;
            info!("Updated roc_camera_app at: {}", self.app_dir.display());
        }

        Ok(())
    }

    pub fn verify_permissions(&self) -> Result<()> {
        // Check if we have write permissions to binary directory
        if !self.bin_dir.exists() {
            fs::create_dir_all(&self.bin_dir).context("Failed to create binary directory")?;
        }

        // Try to write a test file
        let test_file = self.bin_dir.join(".write_test");
        fs::write(&test_file, "test").context("No write permission in binary directory")?;
        fs::remove_file(test_file).context("Failed to clean up test file")?;

        // Check app directory permissions
        if !self.app_dir.exists() {
            fs::create_dir_all(&self.app_dir).context("Failed to create app directory")?;
        }

        let test_file = self.app_dir.join(".write_test");
        fs::write(&test_file, "test").context("No write permission in app directory")?;
        fs::remove_file(test_file).context("Failed to clean up test file")?;

        Ok(())
    }
}
