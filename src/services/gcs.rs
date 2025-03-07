use crate::config::Config;
use anyhow::{Context, Result};
use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
};
use std::fs;
use std::path::Path;

pub struct GcsService {
    client: Client,
    token: String,
    registry_path: String,
}

impl GcsService {
    pub fn new(token: String, registry_path: String) -> Self {
        Self {
            client: Client::new(),
            token,
            registry_path,
        }
    }

    pub fn download_binary(&self, version: &str, output_path: &Path) -> Result<()> {
        let normalized_version = Config::normalize_version(version);
        let url = format!(
            "{}/releases/{}/{}",
            self.registry_path,
            normalized_version,
            Config::RELEASE_BUNDLE_NAME
        );

        let mut request = self.client.get(&url);

        // Only add authorization if token is not empty
        if !self.token.is_empty() {
            let mut headers = HeaderMap::new();
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", self.token))?,
            );
            request = request.headers(headers);
        }

        let response = request.send().context("Failed to download binary")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to download binary: HTTP {}", response.status());
        }

        let content = response
            .bytes()
            .context("Failed to read response content")?;

        fs::write(output_path, content).context("Failed to save binary")?;

        Ok(())
    }

    pub fn verify_version(&self, version: &str) -> Result<bool> {
        let normalized_version = Config::normalize_version(version);
        let url = format!(
            "{}/releases/{}/{}",
            self.registry_path,
            normalized_version,
            Config::CHECKSUM_FILE_NAME
        );

        let mut request = self.client.head(&url);

        // Only add authorization if token is not empty
        if !self.token.is_empty() {
            let mut headers = HeaderMap::new();
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", self.token))?,
            );
            request = request.headers(headers);
        }

        let response = request.send().context("Failed to verify version")?;

        Ok(response.status().is_success())
    }

    pub fn get_latest_version(&self) -> Result<String> {
        let url = format!("{}/releases/latest", self.registry_path);

        let mut request = self.client.get(&url);

        // Only add authorization if token is not empty
        if !self.token.is_empty() {
            let mut headers = HeaderMap::new();
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", self.token))?,
            );
            request = request.headers(headers);
        }

        let response = request.send().context("Failed to fetch latest version")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch latest version: HTTP {}", response.status());
        }

        let version = response
            .text()
            .context("Failed to read latest version from response")?;

        Ok(version.trim().to_string())
    }

    pub fn download_release_bundle(&self, version: &str, output_path: &Path) -> Result<()> {
        let normalized_version = Config::normalize_version(version);

        let url = format!(
            "{}/releases/{}/release_bundle-{}.tar.gz",
            self.registry_path, normalized_version, version
        );

        tracing::debug!("Attempting to download from URL: {}", url);

        let mut request = self.client.get(&url);

        // Only add authorization if token is not empty
        if !self.token.is_empty() {
            let mut headers = HeaderMap::new();
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", self.token))?,
            );
            request = request.headers(headers);
        }

        let response = request
            .send()
            .context("Failed to download release bundle")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Failed to download release bundle: HTTP {}",
                response.status()
            );
        }

        let content = response
            .bytes()
            .context("Failed to read response content")?;

        fs::write(output_path, content).context("Failed to save release bundle")?;

        Ok(())
    }
}
