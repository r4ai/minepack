use anyhow::{anyhow, Context, Result};
use dialoguer::{Confirm, Select};
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::json;
use std::fs::{self, File};
use std::io::Write;

use crate::api::curseforge::CurseforgeClient;
use crate::utils;
use crate::utils::errors::MinepackError;

pub async fn run(mod_query: Option<String>) -> Result<()> {
    // Check if we're in a modpack directory
    if !utils::modpack_exists() {
        return Err(anyhow!(MinepackError::NoModpackFound));
    }

    let config = utils::load_config()?;
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
            Err(_e) => {
                // If failed to find by ID, fall back to search
                println!("Mod ID not found, searching by name instead...");
                let search_results = client
                    .search_mods(&query, Some(&config.minecraft_version))
                    .await
                    .context("Failed to search for mods")?;

                if search_results.is_empty() {
                    return Err(anyhow!(MinepackError::NoModsFound(query)));
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
            return Err(anyhow!(MinepackError::NoModsFound(query)));
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

    println!("Selected mod: {} (ID: {})", mod_info.name, mod_info.id);
    println!("Description: {}", mod_info.summary);

    // Select mod file version that is compatible with the configured Minecraft version
    let compatible_files: Vec<_> = mod_info
        .latest_files
        .iter()
        .filter(|file| file.game_versions.contains(&config.minecraft_version))
        .collect();

    if compatible_files.is_empty() {
        return Err(anyhow!(MinepackError::NoCompatibleModFiles(
            config.minecraft_version.clone()
        )));
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

    // Determine the mod side (client/server/both)
    let side = determine_mod_side(&mod_info.name, &file.file_name)?;

    // Confirm the addition
    let confirm = Confirm::new()
        .with_prompt("Add this mod to your modpack?")
        .default(true)
        .interact()
        .context("Failed to confirm mod addition")?;

    if !confirm {
        return Ok(());
    }

    // Ensure the mods directory exists
    let mods_dir = utils::get_mods_dir();
    utils::ensure_dir_exists(&mods_dir)?;

    // Ensure .minepack/cache/mods directory exists
    let cache_dir = utils::get_cache_mods_dir();
    utils::ensure_dir_exists(&cache_dir)?;

    // Download the mod file to the cache directory
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

    // Download mod file to cache directory
    let mod_data = client
        .download_mod_file(mod_info.id, file.id)
        .await
        .with_context(|| format!("Failed to download mod: {}", mod_info.name))?;

    // Save the mod file to the cache directory
    let cache_file_path = cache_dir.join(&file.file_name);
    let mut file_handle = File::create(&cache_file_path)
        .with_context(|| format!("Failed to create file: {}", cache_file_path.display()))?;
    file_handle
        .write_all(&mod_data)
        .context("Failed to write mod data to file")?;

    // Get the slug for the mod and use it in the JSON filename
    let slug = if mod_info.slug.is_empty() {
        // If slug is empty, create a slug from the name
        mod_info.name.to_lowercase().replace(' ', "-")
    } else {
        mod_info.slug
    };

    // Create the JSON reference file in the mods directory with updated format
    let json_file_path = mods_dir.join(format!("{}.ex.json", slug));
    let json_data = json!({
        "name": mod_info.name,
        "filename": file.file_name,
        "side": side,
        "link": {
            "site": "curseforge",
            "project_id": mod_info.id,
            "file_id": file.id,
        }
    });

    let json_content =
        serde_json::to_string_pretty(&json_data).context("Failed to serialize mod JSON data")?;
    fs::write(&json_file_path, json_content).with_context(|| {
        format!(
            "Failed to write JSON reference file: {}",
            json_file_path.display()
        )
    })?;

    pb.finish_with_message(format!(
        "Downloaded {} to {} and created reference in {}.ex.json",
        file.file_name,
        cache_file_path.display(),
        slug
    ));

    println!("✅ Mod added successfully!");

    Ok(())
}

/// Determines which side (client/server/both) the mod is meant for
fn determine_mod_side(mod_name: &str, file_name: &str) -> Result<&'static str> {
    // This is a very simple heuristic and can be improved
    // For better accuracy, this could be enhanced to read the mod's metadata
    // or use a more sophisticated approach

    let name_lower = mod_name.to_lowercase();
    let file_lower = file_name.to_lowercase();

    // Check for client-side mods
    if name_lower.contains("shader")
        || name_lower.contains("optifine")
        || name_lower.contains("texture")
        || name_lower.contains("resource")
        || name_lower.contains("client")
        || file_lower.contains("client")
    {
        return Ok("client");
    }

    // Check for server-side mods
    if name_lower.contains("server") || file_lower.contains("server") {
        return Ok("server");
    }

    // Default to both sides
    Ok("both")
}
