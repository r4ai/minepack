use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ModpackConfig {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: Option<String>,
    pub mod_loader: ModLoader,
    pub minecraft_version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ModLoader {
    #[serde(rename = "forge")]
    Forge,
    #[serde(rename = "fabric")]
    Fabric,
    #[serde(rename = "quilt")]
    Quilt,
}

impl std::fmt::Display for ModLoader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Forge => write!(f, "forge"),
            Self::Fabric => write!(f, "fabric"),
            Self::Quilt => write!(f, "quilt"),
        }
    }
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

impl ModpackConfig {
    pub fn new(
        name: String,
        version: String,
        author: String,
        description: Option<String>,
        mod_loader: ModLoader,
        minecraft_version: String,
    ) -> Self {
        Self {
            name,
            version,
            author,
            description,
            mod_loader,
            minecraft_version,
        }
    }
}
