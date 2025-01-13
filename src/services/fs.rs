use anyhow::{Context, Result};
use fs_extra::dir::copy as copy_dir;
use fs_extra::dir::CopyOptions;
use std::fs;
use std::os::unix::fs::PermissionsExt;
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
