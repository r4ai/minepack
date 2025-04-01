use thiserror::Error;

#[derive(Error, Debug)]
pub enum MinepackError {
    #[error("No modpack found in the current directory. Run 'minepack init' first.")]
    NoModpackFound,

    #[error("A modpack already exists in this directory")]
    ModpackAlreadyExists,

    #[error("Failed to access Curseforge API: {0}")]
    CurseforgeApiError(String),

    #[error("No compatible files found for Minecraft version {0}")]
    NoCompatibleModFiles(String),

    #[error("No mods found matching '{0}'")]
    NoModsFound(String),

    #[error("Invalid mod loader selected")]
    InvalidModLoader,

    #[error("Invalid export format selected")]
    InvalidExportFormat,

    #[error("Not a valid CurseForge URL. Expected format: https://www.curseforge.com/minecraft/mc-mods/[mod-name] or https://www.curseforge.com/minecraft/mc-mods/[mod-name]/files/[file-id]")]
    InvalidCurseforgeModUrl,

    #[error(
        "Curseforge API key not found. Please set the CURSEFORGE_API_KEY environment variable"
    )]
    #[allow(dead_code)]
    ApiKeyNotFound,

    #[error("Failed to download mod: {0}")]
    ModDownloadError(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid file format: {0}")]
    InvalidFileFormat(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Deserialization error: {0}")]
    DeserializationError(#[from] toml::de::Error),

    #[error("Serialization error: {0}")]
    TomlSerializationError(#[from] toml::ser::Error),

    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}
