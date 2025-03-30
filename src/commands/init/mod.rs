use anyhow::{anyhow, Context, Result};
use dialoguer::{Input, Select};

use crate::models::config::{ModLoader, ModpackConfig};
use crate::utils;
use crate::utils::errors::MinepackError;

pub async fn run() -> Result<()> {
    if utils::modpack_exists() {
        return Err(anyhow!(MinepackError::ModpackAlreadyExists));
    }

    println!("📦 Creating a new Minecraft modpack...");

    // Gather modpack information with interactive prompts
    let name: String = Input::new()
        .with_prompt("Modpack name")
        .interact_text()
        .context("Failed to get modpack name")?;

    let version: String = Input::new()
        .with_prompt("Modpack version")
        .default("1.0.0".to_string())
        .interact_text()
        .context("Failed to get modpack version")?;

    let author: String = Input::new()
        .with_prompt("Author")
        .interact_text()
        .context("Failed to get author name")?;

    let description: String = Input::new()
        .with_prompt("Description (optional)")
        .allow_empty(true)
        .interact_text()
        .context("Failed to get description")?;
    let description = if description.is_empty() {
        None
    } else {
        Some(description)
    };

    // Mod loader selection
    let mod_loader_options = &["Forge", "Fabric", "Quilt"];
    let mod_loader_index = Select::new()
        .with_prompt("Select mod loader")
        .items(mod_loader_options)
        .default(0)
        .interact()
        .context("Failed to select mod loader")?;

    let mod_loader = match mod_loader_index {
        0 => ModLoader::Forge,
        1 => ModLoader::Fabric,
        2 => ModLoader::Quilt,
        _ => return Err(anyhow!(MinepackError::InvalidModLoader)),
    };

    // Minecraft version
    let minecraft_version: String = Input::new()
        .with_prompt("Minecraft version")
        .default("1.20.1".to_string())
        .interact_text()
        .context("Failed to get Minecraft version")?;

    // Create the modpack configuration
    let config = ModpackConfig::new(
        name,
        version,
        author,
        description,
        mod_loader,
        minecraft_version,
    );

    // Create directory structure
    utils::create_modpack_structure()?;

    // Save the configuration file
    utils::save_config(&config)?;

    println!("✅ Modpack initialized successfully!");
    println!("Run 'minepack add <mod>' to add mods to your modpack.");

    Ok(())
}
