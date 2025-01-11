use anyhow::{Context, Result};
use reqwest::{
    blocking::Client,
    header::{ACCEPT, AUTHORIZATION, USER_AGENT},
};
use std::fs;
use std::path::Path;

pub struct GitHubService {
    client: Client,
    token: String,
}

impl GitHubService {
    pub fn new(token: String) -> Self {
        Self {
            client: Client::new(),
            token,
        }
    }

    pub fn download_release_bundle(&self, version: &str, output_path: &Path) -> Result<()> {
        let normalized_version = format!("v{}", version.trim_start_matches('v'));
        let url = format!(
            "https://api.github.com/repos/faust-machines/roc_camera/releases/tags/{}",
            normalized_version
        );

        // First get the release info to get the asset download URL
        let response = self
            .client
            .get(&url)
            .header(ACCEPT, "application/vnd.github.v3+json")
            .header(AUTHORIZATION, format!("Bearer {}", self.token))
            .header(USER_AGENT, "geist-supervisor")
            .send()
            .context("Failed to fetch release info")?;

        let release_info: serde_json::Value =
            response.json().context("Failed to parse release info")?;

        // Find the release bundle asset
        let assets = release_info["assets"]
            .as_array()
            .context("No assets found in release")?;

        let bundle_asset = assets
            .iter()
            .find(|asset| {
                asset["name"]
                    .as_str()
                    .map(|name| name == format!("release_bundle-{}.tar.gz", normalized_version))
                    .unwrap_or(false)
            })
            .context(format!(
                "Release bundle not found in assets for version {}",
                normalized_version
            ))?;

        // Get the asset ID instead of browser_download_url
        let asset_id = bundle_asset["id"].as_u64().context("Invalid asset ID")?;

        // Use the API endpoint for downloading the asset
        let download_url = format!(
            "https://api.github.com/repos/faust-machines/roc_camera/releases/assets/{}",
            asset_id
        );

        // Download the actual release bundle
        let response = self
            .client
            .get(&download_url)
            .header(ACCEPT, "application/octet-stream") // Important: Required for asset downloads
            .header(AUTHORIZATION, format!("Bearer {}", self.token))
            .header(USER_AGENT, "geist-supervisor")
            .send()
            .context("Failed to download release bundle")?;

        let content = response
            .bytes()
            .context("Failed to read response content")?;

        fs::write(output_path, content).context("Failed to save release bundle")?;

        Ok(())
    }

    pub fn verify_release(&self, version: &str) -> Result<bool> {
        let url = format!(
            "https://api.github.com/repos/faust-machines/roc_camera/releases/tags/{}",
            version
        );

        let response = self
            .client
            .get(&url)
            .header(ACCEPT, "application/vnd.github.v3+json")
            .header(AUTHORIZATION, format!("Bearer {}", self.token))
            .header(USER_AGENT, "geist-supervisor")
            .send()
            .context("Failed to fetch release info")?;

        Ok(response.status().is_success())
    }
}
