use thiserror::Error;

#[derive(Error, Debug)]
pub enum MinepackError {
    #[error("No modpack found in the current directory. Run 'minepack init' first.")]
    NoModpackFound,

    #[error("A modpack already exists in this directory")]
    ModpackAlreadyExists,

    #[error("Failed to access Curseforge API: {0}")]
    CurseforgeApiError(String),

    #[error("Mod '{0}' is already in the modpack")]
    ModAlreadyExists(String),

    #[error("No compatible files found for Minecraft version {0}")]
    NoCompatibleModFiles(String),

    #[error("No mods found matching '{0}'")]
    NoModsFound(String),

    #[error("Invalid mod loader selected")]
    InvalidModLoader,

    #[error("Invalid export format selected")]
    InvalidExportFormat,

    #[error(
        "Curseforge API key not found. Please set the CURSEFORGE_API_KEY environment variable"
    )]
    ApiKeyNotFound,

    #[error("Failed to download mod: {0}")]
    ModDownloadError(String),

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

// Helper function to convert any error to the application-specific error
pub fn to_minepack_error<E: std::error::Error>(err: E, context: &str) -> MinepackError {
    MinepackError::Unknown(format!("{}: {}", context, err))
}
