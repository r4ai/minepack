use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ModpackConfig {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: Option<String>,
    pub minecraft: Minecraft,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Minecraft {
    pub version: String,
    pub mod_loaders: Vec<ModLoader>,
}

impl Minecraft {
    pub fn new(version: String, mod_loaders: Vec<ModLoader>) -> Self {
        Self {
            version,
            mod_loaders,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModLoader {
    pub id: String,
    pub version: String,
    pub primary: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModEntry {
    pub name: String,
    pub project_id: u32,
    pub file_id: u32,
    pub version: String,
    pub download_url: String,
    pub required: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Reference {
    pub name: String,
    pub filename: String,
    pub side: Side,
    pub link: Link,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Side {
    #[serde(rename = "both")]
    Both,
    #[serde(rename = "client")]
    Client,
    #[serde(rename = "server")]
    Server,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "site")]
pub enum Link {
    #[serde(rename = "curseforge")]
    CurseForge {
        project_id: u32,
        file_id: u32,
        download_url: Option<String>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Curseforge {}

impl ModpackConfig {
    pub fn new(
        name: String,
        version: String,
        author: String,
        description: Option<String>,
        minecraft: Minecraft,
    ) -> Self {
        Self {
            name,
            version,
            author,
            description,
            minecraft,
        }
    }
}
