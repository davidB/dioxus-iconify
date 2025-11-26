use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const API_BASE_URL: &str = "https://api.iconify.design";

/// Icon data returned from the Iconify API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconifyIcon {
    pub body: String,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
    #[serde(default, rename = "viewBox")]
    pub view_box: Option<String>,
}

/// API response structure for icon data
#[derive(Debug, Deserialize)]
struct IconifyApiResponse {
    prefix: String,
    icons: HashMap<String, IconifyIcon>,
    #[serde(default)]
    width: Option<u32>,
    #[serde(default)]
    height: Option<u32>,
}

/// Iconify API client
pub struct IconifyClient {
    client: reqwest::blocking::Client,
    base_url: String,
}

impl IconifyClient {
    /// Create a new Iconify API client
    pub fn new() -> Result<Self> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            base_url: API_BASE_URL.to_string(),
        })
    }

    /// Fetch a single icon from the Iconify API
    pub fn fetch_icon(&self, collection: &str, icon_name: &str) -> Result<IconifyIcon> {
        let url = format!("{}/{}.json?icons={}", self.base_url, collection, icon_name);

        let response = self.client.get(&url).send().context(format!(
            "Failed to fetch icon {

}:{}",
            collection, icon_name
        ))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "API request failed with status {}: {}",
                response.status(),
                response.text().unwrap_or_default()
            ));
        }

        let api_response: IconifyApiResponse =
            response.json().context("Failed to parse API response")?;

        let icon = api_response
            .icons
            .get(icon_name)
            .ok_or_else(|| {
                anyhow!(
                    "Icon '{}' not found in collection '{}'",
                    icon_name,
                    collection
                )
            })?
            .clone();

        // Use icon-specific dimensions or fall back to collection defaults
        let width = icon.width.or(api_response.width).unwrap_or(24);
        let height = icon.height.or(api_response.height).unwrap_or(24);

        // Generate viewBox if not provided
        let view_box = icon
            .view_box
            .clone()
            .unwrap_or_else(|| format!("0 0 {} {}", width, height));

        Ok(IconifyIcon {
            body: icon.body,
            width: Some(width),
            height: Some(height),
            view_box: Some(view_box),
        })
    }

    /// Fetch multiple icons from the same collection
    pub fn fetch_icons(
        &self,
        collection: &str,
        icon_names: &[String],
    ) -> Result<HashMap<String, IconifyIcon>> {
        if icon_names.is_empty() {
            return Ok(HashMap::new());
        }

        let icons_param = icon_names.join(",");
        let url = format!(
            "{}/{}.json?icons={}",
            self.base_url, collection, icons_param
        );

        let response = self.client.get(&url).send().context(format!(
            "Failed to fetch icons from collection '{}'",
            collection
        ))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "API request failed with status {}: {}",
                response.status(),
                response.text().unwrap_or_default()
            ));
        }

        let api_response: IconifyApiResponse =
            response.json().context("Failed to parse API response")?;

        let default_width = api_response.width.unwrap_or(24);
        let default_height = api_response.height.unwrap_or(24);

        // Process each icon and ensure they have dimensions
        let mut result = HashMap::new();
        for (name, mut icon) in api_response.icons {
            let width = icon.width.unwrap_or(default_width);
            let height = icon.height.unwrap_or(default_height);

            icon.width = Some(width);
            icon.height = Some(height);
            icon.view_box = Some(
                icon.view_box
                    .clone()
                    .unwrap_or_else(|| format!("0 0 {} {}", width, height)),
            );

            result.insert(name, icon);
        }

        Ok(result)
    }
}

impl Default for IconifyClient {
    fn default() -> Self {
        Self::new().expect("Failed to create Iconify API client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires internet connection
    fn test_fetch_icon() {
        let client = IconifyClient::new().unwrap();
        let icon = client.fetch_icon("mdi", "home").unwrap();

        assert!(!icon.body.is_empty());
        assert!(icon.width.is_some());
        assert!(icon.height.is_some());
        assert!(icon.view_box.is_some());
    }

    #[test]
    #[ignore] // Requires internet connection
    fn test_fetch_nonexistent_icon() {
        let client = IconifyClient::new().unwrap();
        let result = client.fetch_icon("mdi", "this-icon-does-not-exist-12345");

        assert!(result.is_err());
    }
}
