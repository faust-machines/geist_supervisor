use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

pub struct Config;

impl Config {
    // Application version from Cargo.toml
    pub const PKG_VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // GCS (Google Cloud Storage) settings
    pub const REGISTRY_BASE_URL: &'static str =
        "https://storage.googleapis.com/roc-camera-releases";

    // Installation paths
    pub fn data_dir() -> PathBuf {
        let home = env::var("HOME").expect("Failed to get HOME directory");
        tracing::info!("Home directory: {}", home);
        let dir = PathBuf::from(home).join(".local/share/roc-supervisor");
        tracing::info!("Attempting to create data directory at: {}", dir.display());
        match fs::create_dir_all(&dir) {
            Ok(_) => tracing::info!("Successfully created or verified data directory"),
            Err(e) => {
                tracing::error!("Error creating data directory: {}", e);
                tracing::error!(
                    "Current permissions: {:?}",
                    fs::metadata(dir.parent().unwrap())
                );
                panic!("Failed to create data directory: {}", e);
            }
        }
        dir
    }

    // Version file
    pub const CURRENT_VERSION_FILE: &'static str = "current_version";

    // Release artifact names
    pub const RELEASE_BUNDLE_NAME: &'static str = "release_bundle.tar.gz";
    pub const CHECKSUM_FILE_NAME: &'static str = "checksums.txt";

    // Version related
    pub const DEFAULT_VERSION: &'static str = "latest";

    /// Normalizes a version string by removing the 'v' prefix if present
    pub fn normalize_version(version: &str) -> String {
        version.trim_start_matches('v').to_string()
    }

    /// Gets the current installed version
    pub fn get_current_version() -> String {
        // First check if it's set in environment
        if let Ok(version) = env::var("GEIST_CURRENT_VERSION") {
            return version;
        }

        // Then check the version file
        let version_file = Self::data_dir().join(Self::CURRENT_VERSION_FILE);
        match fs::read_to_string(version_file) {
            Ok(version) => version.trim().to_string(),
            Err(_) => format!("v{}", Self::PKG_VERSION), // Default to package version if no version file
        }
    }

    /// Sets the current version
    pub fn set_current_version(version: &str) -> io::Result<()> {
        let version_file = Self::data_dir().join(Self::CURRENT_VERSION_FILE);
        let mut file = fs::File::create(version_file)?;
        file.write_all(version.as_bytes())?;
        Ok(())
    }
}
