use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModInfo {
    pub id: u32,
    pub name: String,
    pub slug: String,
    pub summary: String,
    pub download_count: u64,
    #[serde(rename = "latestFiles")]
    pub latest_files: Vec<ModFile>,
    #[serde(rename = "gameVersionLatestFiles")]
    pub game_version_latest_files: Vec<GameVersionFile>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModFile {
    pub id: u32,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "fileDate")]
    pub file_date: String,
    #[serde(rename = "downloadUrl")]
    pub download_url: String,
    #[serde(rename = "gameVersions")]
    pub game_versions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameVersionFile {
    #[serde(rename = "gameVersion")]
    pub game_version: String,
    #[serde(rename = "fileId")]
    pub file_id: u32,
}

// Search result from Curseforge API
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub data: Vec<ModInfo>,
}

// Response format for a single mod from the Curseforge API
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModResponse {
    pub data: ModInfo,
}
