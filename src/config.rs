use std::env;
use std::fs;
use std::path::PathBuf;

pub struct Config;

impl Config {
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

    // Release artifact names
    pub const RELEASE_BUNDLE_NAME: &'static str = "release_bundle.tar.gz";
    pub const CHECKSUM_FILE_NAME: &'static str = "checksums.txt";

    // Version related
    pub const DEFAULT_VERSION: &'static str = "latest";

    /// Normalizes a version string by removing the 'v' prefix if present
    pub fn normalize_version(version: &str) -> String {
        version.trim_start_matches('v').to_string()
    }
}
