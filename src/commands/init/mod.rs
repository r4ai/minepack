use anyhow::{anyhow, Context, Result};
use dialoguer::{Input, Select};

use crate::models::config::{Minecraft, ModLoader, ModpackConfig};
use crate::utils;
use crate::utils::errors::MinepackError;

#[allow(clippy::too_many_arguments)]
pub async fn run<E: utils::Env>(
    env: &E,
    name_opt: Option<String>,
    version_opt: Option<String>,
    author_opt: Option<String>,
    description_opt: Option<String>,
    loader_opt: Option<String>,
    minecraft_version_opt: Option<String>,
    loader_version_opt: Option<String>,
) -> Result<()> {
    if utils::modpack_exists(env) {
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
    let mod_loader_id = if let Some(loader) = loader_opt {
        match loader.to_lowercase().as_str() {
            "forge" => "forge".to_string(),
            "fabric" => "fabric".to_string(),
            "quilt" => "quilt".to_string(),
            "neoforge" => "neoforge".to_string(),
            _ => return Err(anyhow!(MinepackError::InvalidModLoader)),
        }
    } else {
        let mod_loader_options = &["Forge", "Fabric", "Quilt", "NeoForge"];
        let mod_loader_index = Select::new()
            .with_prompt("Select mod loader")
            .items(mod_loader_options)
            .default(0)
            .interact()
            .context("Failed to select mod loader")?;

        match mod_loader_index {
            0 => "forge".to_string(),
            1 => "fabric".to_string(),
            2 => "quilt".to_string(),
            3 => "neoforge".to_string(),
            _ => return Err(anyhow!(MinepackError::InvalidModLoader)),
        }
    };

    // Mod loader version
    let mod_loader_version = if let Some(version) = loader_version_opt {
        version
    } else {
        Input::new()
            .with_prompt("Mod loader version")
            .interact_text()
            .context("Failed to get mod loader version")?
    };

    // Create ModLoader struct
    let mod_loader = ModLoader {
        id: mod_loader_id,
        version: mod_loader_version,
        primary: true,
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

    // Create the minecraft configuration
    let minecraft = Minecraft::new(minecraft_version, vec![mod_loader]);

    // Create the modpack configuration
    let config = ModpackConfig::new(name, version, author, description, minecraft);

    // Create directory structure
    utils::create_modpack_structure(env)?;

    // Save the configuration file
    utils::save_config(env, &config)?;

    println!("✅ Modpack initialized successfully!");
    println!("Run 'minepack add <mod>' to add mods to your modpack.");

    Ok(())
}
