use anyhow::{Context, Result};
use dialoguer::{Confirm, Select};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::File;
use std::io::Write;

use crate::api::curseforge::CurseforgeClient;
use crate::models::config::ModEntry;
use crate::utils;
use crate::utils::errors::MinepackError;

pub async fn run(mod_query: Option<String>) -> Result<()> {
    // Check if we're in a modpack directory
    if !utils::modpack_exists() {
        return Err(MinepackError::NoModpackFound.into());
    }

    let mut config = utils::load_config()?;
    let client = CurseforgeClient::new().context("Failed to initialize Curseforge API client")?;

    // If no mod query is provided, prompt the user for one
    let query = match mod_query {
        Some(q) => q,
        None => {
            let query: String = dialoguer::Input::new()
                .with_prompt("Enter mod name or ID to search")
                .interact_text()
                .context("Failed to get mod query")?;
            query
        }
    };

    // Try parsing as mod ID first
    let mod_id = query.parse::<u32>();
    let mod_info = if let Ok(id) = mod_id {
        // Directly fetch the mod info using the ID
        println!("🔍 Fetching mod with ID {}...", id);
        match client.get_mod_info(id).await {
            Ok(info) => info,
            Err(e) => {
                // If failed to find by ID, fall back to search
                println!("Mod ID not found, searching by name instead...");
                let search_results = client
                    .search_mods(&query, Some(&config.minecraft_version))
                    .await
                    .context("Failed to search for mods")?;

                if search_results.is_empty() {
                    return Err(MinepackError::NoModsFound(query).into());
                }

                // Display the results for selection
                let options: Vec<String> = search_results
                    .iter()
                    .map(|m| format!("{}: {}", m.id, m.name))
                    .collect();

                let selection = Select::new()
                    .with_prompt("Select a mod to add")
                    .items(&options)
                    .default(0)
                    .interact()
                    .context("Failed to select mod")?;

                search_results[selection].clone()
            }
        }
    } else {
        // Search for mods by name
        println!("🔍 Searching for mods matching '{}'...", query);
        let search_results = client
            .search_mods(&query, Some(&config.minecraft_version))
            .await
            .context("Failed to search for mods")?;

        if search_results.is_empty() {
            return Err(MinepackError::NoModsFound(query).into());
        }

        // Display the results for selection
        let options: Vec<String> = search_results
            .iter()
            .map(|m| format!("{}: {}", m.id, m.name))
            .collect();

        let selection = Select::new()
            .with_prompt("Select a mod to add")
            .items(&options)
            .default(0)
            .interact()
            .context("Failed to select mod")?;

        search_results[selection].clone()
    };

    // Check if the mod is already in the pack
    if config.mods.iter().any(|m| m.project_id == mod_info.id) {
        return Err(MinepackError::ModAlreadyExists(mod_info.name).into());
    }

    println!("Selected mod: {} (ID: {})", mod_info.name, mod_info.id);
    println!("Description: {}", mod_info.summary);

    // Select mod file version that is compatible with the configured Minecraft version
    let compatible_files: Vec<_> = mod_info
        .latest_files
        .iter()
        .filter(|file| file.game_versions.contains(&config.minecraft_version))
        .collect();

    if compatible_files.is_empty() {
        return Err(MinepackError::NoCompatibleModFiles(config.minecraft_version.clone()).into());
    }

    // If multiple versions are available, let the user select one
    let file = if compatible_files.len() == 1 {
        compatible_files[0]
    } else {
        let file_options: Vec<String> = compatible_files
            .iter()
            .map(|f| format!("{}: {}", f.id, f.display_name))
            .collect();

        let file_selection = Select::new()
            .with_prompt("Select a file version")
            .items(&file_options)
            .default(0)
            .interact()
            .context("Failed to select file version")?;

        compatible_files[file_selection]
    };

    println!("Selected file: {} (ID: {})", file.display_name, file.id);

    // Confirm the addition
    let confirm = Confirm::new()
        .with_prompt("Add this mod to your modpack?")
        .default(true)
        .interact()
        .context("Failed to confirm mod addition")?;

    if !confirm {
        return Ok(());
    }

    // Ensure mods directory exists
    let mods_dir = utils::get_mods_dir();
    utils::ensure_dir_exists(&mods_dir)?;

    // Download the mod file
    println!("⬇️  Downloading mod...");
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner} {msg}")
            .context("Failed to create progress style")?,
    );
    pb.set_message(format!("Downloading {}", file.file_name));
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let mod_data = client
        .download_mod_file(mod_info.id, file.id)
        .await
        .context(format!("Failed to download mod: {}", mod_info.name))?;

    // Save the mod file to the mods directory
    let file_path = mods_dir.join(&file.file_name);
    let mut file_handle = File::create(&file_path)
        .context(format!("Failed to create file: {}", file_path.display()))?;
    file_handle
        .write_all(&mod_data)
        .context("Failed to write mod data to file")?;

    pb.finish_with_message(format!(
        "Downloaded {} to {}",
        file.file_name,
        file_path.display()
    ));

    // Add the mod to the config
    let mod_entry = ModEntry {
        name: mod_info.name,
        project_id: mod_info.id,
        file_id: file.id,
        version: file.display_name.clone(),
        download_url: file.download_url.clone(),
        required: true,
    };

    config.mods.push(mod_entry);
    utils::save_config(&config)?;

    println!("✅ Mod added successfully!");

    Ok(())
}
