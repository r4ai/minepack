use anyhow::{anyhow, Context, Result};
use dialoguer::{Input, Select};

use crate::models::config::{ModLoader, ModpackConfig};
use crate::utils;
use crate::utils::errors::MinepackError;

pub async fn run(
    name_opt: Option<String>,
    version_opt: Option<String>,
    author_opt: Option<String>,
    description_opt: Option<String>,
    loader_opt: Option<String>,
    minecraft_version_opt: Option<String>,
) -> Result<()> {
    if utils::modpack_exists() {
        return Err(anyhow!(MinepackError::ModpackAlreadyExists));
    }

    println!("📦 Creating a new Minecraft modpack...");

    // If all options are provided, skip interactive mode
    let _non_interactive = name_opt.is_some() && author_opt.is_some();

    // Gather modpack information with interactive prompts if options aren't provided
    let name = if let Some(name) = name_opt {
        name
    } else {
        Input::new()
            .with_prompt("Modpack name")
            .interact_text()
            .context("Failed to get modpack name")?
    };

    let version = if let Some(version) = version_opt {
        version
    } else {
        Input::new()
            .with_prompt("Modpack version")
            .default("1.0.0".to_string())
            .interact_text()
            .context("Failed to get modpack version")?
    };

    let author = if let Some(author) = author_opt {
        author
    } else {
        Input::new()
            .with_prompt("Author")
            .interact_text()
            .context("Failed to get author name")?
    };

    let description = if let Some(desc) = description_opt {
        if desc.is_empty() {
            None
        } else {
            Some(desc)
        }
    } else {
        let desc: String = Input::new()
            .with_prompt("Description (optional)")
            .allow_empty(true)
            .interact_text()
            .context("Failed to get description")?;

        if desc.is_empty() {
            None
        } else {
            Some(desc)
        }
    };

    // Mod loader selection
    let mod_loader = if let Some(loader) = loader_opt {
        match loader.to_lowercase().as_str() {
            "forge" => ModLoader::Forge,
            "fabric" => ModLoader::Fabric,
            "quilt" => ModLoader::Quilt,
            "neoforge" => ModLoader::Forge, // Adding support for neoforge as forge
            _ => return Err(anyhow!(MinepackError::InvalidModLoader)),
        }
    } else {
        let mod_loader_options = &["Forge", "Fabric", "Quilt"];
        let mod_loader_index = Select::new()
            .with_prompt("Select mod loader")
            .items(mod_loader_options)
            .default(0)
            .interact()
            .context("Failed to select mod loader")?;

        match mod_loader_index {
            0 => ModLoader::Forge,
            1 => ModLoader::Fabric,
            2 => ModLoader::Quilt,
            _ => return Err(anyhow!(MinepackError::InvalidModLoader)),
        }
    };

    // Minecraft version
    let minecraft_version = if let Some(version) = minecraft_version_opt {
        version
    } else {
        Input::new()
            .with_prompt("Minecraft version")
            .default("1.20.1".to_string())
            .interact_text()
            .context("Failed to get Minecraft version")?
    };

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
