use anyhow::{anyhow, Result};
use reqwest::header::{HeaderMap, HeaderValue};
use std::env;
use std::fs;
use url::Url;

use crate::models::mod_info::{ModInfo, ModResponse, SearchResult};

const CURSEFORGE_API_URL: &str = "https://api.curseforge.com/v1";
const MINECRAFT_GAME_ID: u32 = 432;
const CONFIG_FILE_NAME: &str = ".minepack-config";

pub struct CurseforgeClient {
    client: reqwest::Client,
}

impl CurseforgeClient {
    pub fn new() -> Result<Self> {
        // Try to get API key from different sources
        let api_key = Self::get_api_key()?;

        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", HeaderValue::from_str(&api_key)?);

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self { client })
    }

    fn get_api_key() -> Result<String> {
        // First try environment variable
        if let Ok(key) = env::var("CURSEFORGE_API_KEY") {
            return Ok(key);
        }

        // Then try config file in home directory
        if let Some(mut home_dir) = dirs::home_dir() {
            home_dir.push(CONFIG_FILE_NAME);
            if home_dir.exists() {
                let content = fs::read_to_string(home_dir)?;
                for line in content.lines() {
                    if let Some(key) = line.strip_prefix("api_key=") {
                        return Ok(key.trim().to_string());
                    }
                }
            }
        }

        // For development/demo, provide a placeholder key
        #[cfg(debug_assertions)]
        {
            eprintln!("WARNING: Using placeholder API key for development. This will not work with the real API.");
            Ok("$2a$10$This.Is.A.Development.Key.For.Local.Testing.Only".to_string())
        }

        // In release mode, we require a real API key
        #[cfg(not(debug_assertions))]
        Err(anyhow!("Curseforge API key not found. Please set the CURSEFORGE_API_KEY environment variable or create a {CONFIG_FILE_NAME} file in your home directory with api_key=YOUR_KEY"))
    }

    pub async fn search_mods(
        &self,
        query: &str,
        minecraft_version: Option<&str>,
    ) -> Result<Vec<ModInfo>> {
        let mut url = Url::parse(CURSEFORGE_API_URL)?;
        url.path_segments_mut()
            .map_err(|_| anyhow!("Cannot modify URL path"))?
            .push("mods")
            .push("search");

        // Add query parameters
        url.query_pairs_mut()
            .append_pair("gameId", &MINECRAFT_GAME_ID.to_string())
            .append_pair("searchFilter", query);

        if let Some(version) = minecraft_version {
            url.query_pairs_mut().append_pair("gameVersion", version);
        }

        let response = self.client.get(url).send().await?;
        let result: SearchResult = response.json().await?;

        Ok(result.data)
    }

    pub async fn get_mod_info(&self, mod_id: u32) -> Result<ModInfo> {
        let mut url = Url::parse(CURSEFORGE_API_URL)?;
        url.path_segments_mut()
            .map_err(|_| anyhow!("Cannot modify URL path"))?
            .push("mods")
            .push(&mod_id.to_string());

        let response = self.client.get(url).send().await?;
        let mod_response: ModResponse = response.json().await?;

        Ok(mod_response.data)
    }

    pub async fn download_mod_file(&self, mod_id: u32, file_id: u32) -> Result<Vec<u8>> {
        let mut url = Url::parse(CURSEFORGE_API_URL)?;
        url.path_segments_mut()
            .map_err(|_| anyhow!("Cannot modify URL path"))?
            .push("mods")
            .push(&mod_id.to_string())
            .push("files")
            .push(&file_id.to_string())
            .push("download-url");

        let response = self.client.get(url).send().await?;
        let download_url: String = response.json().await?;

        // Download the actual file
        let mod_file = reqwest::get(&download_url).await?;
        let bytes = mod_file.bytes().await?;

        Ok(bytes.to_vec())
    }
}
