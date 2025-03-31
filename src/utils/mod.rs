pub mod errors;

use anyhow::{anyhow, Context, Result};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::models::config::ModpackConfig;
use crate::utils::errors::MinepackError;

const CONFIG_FILENAME: &str = "minepack.json";

pub trait Env {
    fn current_dir(&self) -> std::io::Result<PathBuf>;
}

pub struct RealEnv;

impl Env for RealEnv {
    fn current_dir(&self) -> std::io::Result<PathBuf> {
        std::env::current_dir()
    }
}

#[cfg(feature = "mock")]
pub struct MockEnv {
    pub tempdir: assert_fs::TempDir,
}

#[cfg(feature = "mock")]
impl MockEnv {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            tempdir: assert_fs::TempDir::new().unwrap(),
        }
    }

    #[allow(dead_code)]
    pub fn close(self) -> Result<(), assert_fs::fixture::FixtureError> {
        self.tempdir.close()
    }
}

#[cfg(feature = "mock")]
impl Env for MockEnv {
    fn current_dir(&self) -> std::io::Result<PathBuf> {
        Ok(self.tempdir.path().to_path_buf())
    }
}

pub fn get_minepack_config_path<E: Env>(env: &E) -> anyhow::Result<PathBuf> {
    let current_dir = env.current_dir()?;
    Ok(current_dir.join(CONFIG_FILENAME))
}

pub fn load_config<E: Env>(env: &E) -> Result<ModpackConfig> {
    let config_path = get_minepack_config_path(env)?;
    if !config_path.exists() {
        return Err(anyhow!(MinepackError::NoModpackFound));
    }

    let config_content = fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;
    let config: ModpackConfig = serde_json::from_str(&config_content)
        .with_context(|| "Failed to parse modpack configuration")?;
    Ok(config)
}

pub fn save_config<E: Env>(env: &E, config: &ModpackConfig) -> Result<()> {
    let config_path = get_minepack_config_path(env)?;
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

pub fn modpack_exists<E: Env>(env: &E) -> bool {
    get_minepack_config_path(env)
        .map(|path| path.exists())
        .unwrap_or(false)
}

pub fn get_build_dir<E: Env>(env: &E) -> anyhow::Result<PathBuf> {
    let current_dir = env.current_dir()?;
    Ok(current_dir.join("build"))
}

pub fn get_mods_dir<E: Env>(env: &E) -> anyhow::Result<PathBuf> {
    let current_dir = env.current_dir()?;
    Ok(current_dir.join("mods"))
}

pub fn get_config_dir<E: Env>(env: &E) -> anyhow::Result<PathBuf> {
    let current_dir = env.current_dir()?;
    Ok(current_dir.join("config"))
}

pub fn get_minepack_dir<E: Env>(env: &E) -> anyhow::Result<PathBuf> {
    let current_dir = env.current_dir()?;
    Ok(current_dir.join(".minepack"))
}

pub fn get_minepack_cache_dir<E: Env>(env: &E) -> anyhow::Result<PathBuf> {
    get_minepack_dir(env).map(|path| path.join("cache"))
}

pub fn get_minepack_cache_mods_dir<E: Env>(env: &E) -> anyhow::Result<PathBuf> {
    get_minepack_cache_dir(env).map(|path| path.join("mods"))
}

pub fn create_modpack_structure<E: Env>(env: &E) -> Result<()> {
    ensure_dir_exists(&get_mods_dir(env)?)?;
    ensure_dir_exists(&get_config_dir(env)?)?;
    ensure_dir_exists(&get_minepack_dir(env)?)?;
    ensure_dir_exists(&get_minepack_cache_dir(env)?)?;
    ensure_dir_exists(&get_minepack_cache_mods_dir(env)?)?;
    Ok(())
}
