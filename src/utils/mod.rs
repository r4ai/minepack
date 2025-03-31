pub mod errors;

use anyhow::{anyhow, Context, Result};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::models::config::ModpackConfig;
use crate::utils::errors::MinepackError;

const CONFIG_FILENAME: &str = "minepack.json";

pub fn get_config_path() -> PathBuf {
    Path::new(CONFIG_FILENAME).to_path_buf()
}

pub fn load_config() -> Result<ModpackConfig> {
    let config_path = get_config_path();
    if !config_path.exists() {
        return Err(anyhow!(MinepackError::NoModpackFound));
    }

    let config_content = fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;
    let config: ModpackConfig = serde_json::from_str(&config_content)
        .with_context(|| "Failed to parse modpack configuration")?;
    Ok(config)
}

pub fn save_config(config: &ModpackConfig) -> Result<()> {
    let config_path = get_config_path();
    let config_content = serde_json::to_string_pretty(config)
        .with_context(|| "Failed to serialize modpack configuration")?;
    let mut file = File::create(&config_path)
        .with_context(|| format!("Failed to create config file: {}", config_path.display()))?;
    file.write_all(config_content.as_bytes())
        .with_context(|| "Failed to write to config file")?;
    Ok(())
}

pub fn ensure_dir_exists(dir_path: &Path) -> Result<()> {
    if !dir_path.exists() {
        fs::create_dir_all(dir_path)
            .with_context(|| format!("Failed to create directory: {}", dir_path.display()))?;
    }
    Ok(())
}

pub fn modpack_exists() -> bool {
    get_config_path().exists()
}

pub fn get_mods_dir() -> PathBuf {
    Path::new("mods").to_path_buf()
}

pub fn get_minepack_dir() -> PathBuf {
    Path::new(".minepack").to_path_buf()
}

pub fn get_cache_dir() -> PathBuf {
    get_minepack_dir().join("cache")
}

pub fn get_cache_mods_dir() -> PathBuf {
    get_cache_dir().join("mods")
}

pub fn create_modpack_structure() -> Result<()> {
    ensure_dir_exists(Path::new("mods"))?;
    ensure_dir_exists(Path::new("config"))?;
    ensure_dir_exists(&get_minepack_dir())?;
    ensure_dir_exists(&get_cache_dir())?;
    ensure_dir_exists(&get_cache_mods_dir())?;
    Ok(())
}
