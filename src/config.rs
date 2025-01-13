use std::path::PathBuf;

pub struct Config;

impl Config {
    // GCS (Google Cloud Storage) settings
    pub const REGISTRY_BASE_URL: &'static str =
        "https://storage.googleapis.com/roc-camera-releases";

    // Installation paths
    pub fn bin_dir() -> PathBuf {
        PathBuf::from("/usr/local/bin")
    }

    pub fn app_dir() -> PathBuf {
        PathBuf::from("/opt/roc_camera_app")
    }

    // Binary names and paths
    pub const BINARY_NAME: &'static str = "roc_camera";

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
