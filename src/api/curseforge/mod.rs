mod schema;

use anyhow::{anyhow, Context, Result};
use reqwest::header::{HeaderMap, HeaderValue};
use schema::GetDownloadUrlResponse;
use std::env;
use std::fs;
use url::Url;

use crate::utils::errors::MinepackError;

// テスト環境でない場合は本番のAPIを使用
const CURSEFORGE_API_URL_PROD: &str = "https://api.curseforge.com/v1";
const MINECRAFT_GAME_ID: u32 = 432;
const CONFIG_FILE_NAME: &str = ".minepack-config";

pub struct CurseforgeClient {
    client: reqwest::Client,
    base_url: String,
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

        // テスト環境の場合はモックサーバーのURLを使用
        let base_url = if cfg!(test) {
            match env::var("MOCK_SERVER_URL") {
                Ok(url) => format!("{}/api.curseforge.com/v1", url),
                Err(_) => "http://127.0.0.1:25569/api.curseforge.com/v1".to_string(),
            }
        } else {
            CURSEFORGE_API_URL_PROD.to_string()
        };

        Ok(Self { client, base_url })
    }

    /// Creates a new client with a custom base URL (useful for testing with mock server)
    #[allow(dead_code)]
    #[cfg(test)]
    pub fn new_with_base_url(base_url: &str) -> Result<Self> {
        let api_key = Self::get_api_key()?;

        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", HeaderValue::from_str(&api_key)?);

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self {
            client,
            base_url: base_url.to_string(),
        })
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
        Err(anyhow!(MinepackError::ApiKeyNotFound))
    }

    pub async fn search_mods(
        &self,
        query: &str,
        minecraft_version: Option<&str>,
    ) -> Result<Vec<schema::Mod>> {
        let mut url = Url::parse(&self.base_url)?;
        url.path_segments_mut()
            .map_err(|_| anyhow!(MinepackError::Unknown("Cannot modify URL path".to_string())))?
            .push("mods")
            .push("search");

        // Add query parameters
        url.query_pairs_mut()
            .append_pair("gameId", &MINECRAFT_GAME_ID.to_string())
            .append_pair("searchFilter", query);

        if let Some(version) = minecraft_version {
            url.query_pairs_mut().append_pair("gameVersion", version);
        }

        let response = self.client.get(url).send().await.with_context(|| {
            format!(
                "Failed to send request to Curseforge API for search query '{}'",
                query
            )
        })?;

        if !response.status().is_success() {
            return Err(anyhow!(MinepackError::CurseforgeApiError(format!(
                "API request failed with status: {}",
                response.status()
            ))));
        }

        let result: schema::SearchModsResponse = response
            .json()
            .await
            .with_context(|| "Failed to parse search results from Curseforge API")?;

        Ok(result.data)
    }

    pub async fn get_mod_info(&self, mod_id: u32) -> Result<schema::Mod> {
        let mut url = Url::parse(&self.base_url)?;
        url.path_segments_mut()
            .map_err(|_| anyhow!(MinepackError::Unknown("Cannot modify URL path".to_string())))?
            .push("mods")
            .push(&mod_id.to_string());

        let response = self.client.get(url).send().await.with_context(|| {
            format!(
                "Failed to send request to Curseforge API for mod ID {}",
                mod_id
            )
        })?;

        if !response.status().is_success() {
            return Err(anyhow!(MinepackError::CurseforgeApiError(format!(
                "API request failed with status: {}",
                response.status()
            ))));
        }

        let mod_response: schema::GetModResponse = response
            .json()
            .await
            .with_context(|| format!("Failed to parse mod info for ID {}", mod_id))?;

        Ok(mod_response.data)
    }

    pub async fn download_mod_file(&self, mod_id: u32, file_id: u32) -> Result<Vec<u8>> {
        let mut url = Url::parse(&self.base_url)?;
        url.path_segments_mut()
            .map_err(|_| anyhow!(MinepackError::Unknown("Cannot modify URL path".to_string())))?
            .push("mods")
            .push(&mod_id.to_string())
            .push("files")
            .push(&file_id.to_string())
            .push("download-url");

        let response = self.client.get(url).send().await.with_context(|| {
            format!(
                "Failed to get download URL for mod ID {} file ID {}",
                mod_id, file_id
            )
        })?;

        if !response.status().is_success() {
            return Err(anyhow!(MinepackError::ModDownloadError(format!(
                "Failed to get download URL with status: {}",
                response.status()
            ))));
        }

        let download_url_response: GetDownloadUrlResponse = response
            .json()
            .await
            .with_context(|| "Failed to parse download URL")?;
        let download_url = download_url_response
            .data
            .context("Failed to get download URL from response")?;

        dbg!(&download_url);

        // Download the actual file
        let mod_file = reqwest::get(&download_url)
            .await
            .with_context(|| format!("Failed to download mod file from {}", download_url))?;

        if !mod_file.status().is_success() {
            return Err(anyhow!(MinepackError::ModDownloadError(format!(
                "Failed to download mod file with status: {}",
                mod_file.status()
            ))));
        }

        let bytes = mod_file
            .bytes()
            .await
            .with_context(|| "Failed to read mod file bytes")?;

        Ok(bytes.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_mod_info() {
        // モックサーバーの起動はDenoスクリプトで行うため、Rust側でのモック処理コードを削除
        // 環境変数 MOCK_SERVER_URL を使って接続先を設定
        let client = CurseforgeClient::new().unwrap();

        // Test search mods
        let mods = client.get_mod_info(1030830).await.unwrap();
        assert_eq!(mods.id, 1030830);
        assert_eq!(mods.game_id, MINECRAFT_GAME_ID);
        assert_eq!(mods.name, "Oritech");
    }

    #[tokio::test]
    async fn test_search_mods() {
        // モックサーバーの起動はDenoスクリプトで行うため、Rust側でのモック処理コードを削除
        let client = CurseforgeClient::new().unwrap();

        // Test search mods
        let mods = client.search_mods("oritech", None).await.unwrap();
        assert!(!mods.is_empty());
        assert!(mods.iter().any(|m| m.id == 1030830));
    }

    #[tokio::test]
    async fn test_download_mod_file() {
        // モックサーバーの起動はDenoスクリプトで行うため、Rust側でのモック処理コードを削除
        let client = CurseforgeClient::new().unwrap();

        // Test download mod file
        let bytes = client.download_mod_file(1030830, 6332315).await.unwrap();
        assert!(!bytes.is_empty());
    }
}
