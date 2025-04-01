use anyhow::{anyhow, bail, Context, Result};
use dialoguer::{Confirm, Select};
use serde_json::json;
use std::fs;
use url::Url;

use crate::api::curseforge::schema::Mod as CurseForgeModInfo;
use crate::api::curseforge::CurseforgeClient;
use crate::utils::{determine_mod_side, errors::MinepackError};
use crate::{api, models, utils};

fn parse_curseforge_mod_url(url: &Url) -> Result<(&str, Option<u32>)> {
    // Validate that it's a curseforge.com URL
    if url.host_str() != Some("www.curseforge.com") {
        bail!(MinepackError::InvalidCurseforgeModUrl);
    }

    // Split the path to extract mod name and potentially file ID
    let path_segments: Vec<&str> = url.path().split('/').filter(|s| !s.is_empty()).collect();

    // Validate URL structure
    if path_segments.len() < 3 || path_segments[0] != "minecraft" || path_segments[1] != "mc-mods" {
        bail!(MinepackError::InvalidCurseforgeModUrl);
    }

    // If we have a file ID in the URL (/files/[file-id])
    let file_id = if path_segments.len() >= 5 && path_segments[3] == "files" {
        let id = path_segments[4]
            .parse::<u32>()
            .context("Invalid file ID in URL")?;
        Some(id)
    } else {
        None
    };

    // Extract the slug from the URL
    let slug = path_segments[2];

    Ok((slug, file_id))
}

/// Extract mod information from CurseForge URL and fetch the mod details
async fn extract_mod_info_from_url(
    url_str: &str,
    client: &CurseforgeClient,
    minecraft_version: &str,
) -> Result<(CurseForgeModInfo, Option<u32>)> {
    // Parse the URL
    let url = Url::parse(url_str).context("Invalid URL format")?;

    let (slug, file_id) = parse_curseforge_mod_url(&url)?;

    // Search for mod by slug
    println!("🔍 Looking up mod from URL: {}", slug);
    let search_results = client
        .search_mods(&api::curseforge::schema::SearchModsRequestQuery {
            slug: Some(slug.to_string()),
            game_version: Some(minecraft_version.to_string()),
            ..Default::default()
        })
        .await
        .with_context(|| format!("Failed to search for mod with slug: {}", slug))?;

    if search_results.is_empty() {
        bail!(MinepackError::NoModsFound(slug.to_string()));
    }

    // Find the mod that matches the slug
    let matching_mod = search_results.iter().find(|m| m.slug == slug);

    match matching_mod {
        Some(mod_info) => {
            // Once we find a match, fetch the complete mod info
            let complete_mod_info = client
                .get_mod_info(mod_info.id)
                .await
                .context("Failed to fetch detailed mod information")?;
            Ok((complete_mod_info, file_id))
        }
        None => bail!(MinepackError::NoModsFound(slug.to_string())),
    }
}

pub async fn run<E: utils::Env>(env: &E, mod_query: Option<String>, yes: bool) -> Result<()> {
    // Check if we're in a modpack directory
    if !utils::modpack_exists(env) {
        return Err(anyhow!(MinepackError::NoModpackFound));
    }

    let config = utils::load_config(env)?;
    let client = CurseforgeClient::new().context("Failed to initialize Curseforge API client")?;

    // If no mod query is provided, prompt the user for one
    let query = match mod_query {
        Some(q) => q,
        None => {
            let query: String = dialoguer::Input::new()
                .with_prompt("Enter mod name or CurseForge URL")
                .interact_text()
                .context("Failed to get mod query")?;
            query
        }
    };

    // Check if the query is a URL and extract mod info if so
    let (mod_info, file_id_from_url) = if query.starts_with("https://www.curseforge.com/") {
        // Extract mod info from URL
        extract_mod_info_from_url(&query, &client, &config.minecraft.version).await?
    } else {
        // Search for mods by name
        println!("🔍 Searching for mods matching '{}'...", query);
        let search_results = client
            .search_mods(&api::curseforge::schema::SearchModsRequestQuery {
                search_filter: Some(query.clone()),
                game_version: Some(config.minecraft.version.clone()),
                ..Default::default()
            })
            .await
            .context("Failed to search for mods")?;

        if search_results.is_empty() {
            bail!(MinepackError::NoModsFound(query));
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

        // Fetch complete mod info for the selected mod
        let complete_mod_info = client
            .get_mod_info(search_results[selection].id)
            .await
            .context("Failed to fetch detailed mod information")?;

        (complete_mod_info, None)
    };

    println!("Selected mod: {} (ID: {})", mod_info.name, mod_info.id);
    println!("Description: {}", mod_info.summary);

    // Select mod file version that is compatible with the configured Minecraft version
    let compatible_files: Vec<_> = mod_info
        .latest_files
        .iter()
        .filter(|file| file.game_versions.contains(&config.minecraft.version))
        .collect();

    if compatible_files.is_empty() {
        return Err(anyhow!(MinepackError::NoCompatibleModFiles(
            config.minecraft.version.clone()
        )));
    }

    // If a specific file ID was provided in the URL, find that file
    let file = if let Some(file_id) = file_id_from_url {
        // Try to find the specified file ID in compatible files
        let file_match = compatible_files.iter().find(|f| f.id == file_id);

        match file_match {
            Some(f) => *f,
            None => {
                // If the specified file ID isn't compatible or doesn't exist
                println!(
                    "Warning: The specified file ID {} is not compatible with Minecraft {}",
                    file_id, config.minecraft.version
                );

                // Ask user what to do
                let options = vec!["Select a compatible version instead", "Cancel installation"];
                let selection = Select::new()
                    .with_prompt("What would you like to do?")
                    .items(&options)
                    .default(0)
                    .interact()
                    .context("Failed to get user selection")?;

                if selection == 1 {
                    // Cancel
                    return Ok(());
                }

                // Fall through to normal selection
                if compatible_files.len() == 1 {
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
                }
            }
        }
    } else if compatible_files.len() == 1 {
        compatible_files[0]
    } else {
        // Multiple versions available, let the user select
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
    let confirm = yes
        || Confirm::new()
            .with_prompt("Add this mod to your modpack?")
            .default(true)
            .interact()
            .context("Failed to confirm mod addition")?;

    if !confirm {
        return Ok(());
    }

    // Ensure the mods directory exists
    let mods_dir = utils::get_mods_dir(env)?;
    utils::ensure_dir_exists(&mods_dir)?;

    // Get the slug for the mod and use it in the JSON filename
    let slug = if mod_info.slug.is_empty() {
        // If slug is empty, create a slug from the name
        mod_info.name.to_lowercase().replace(' ', "-")
    } else {
        mod_info.slug
    };

    // Create the JSON reference file in the mods directory with updated format
    let json_file_path = mods_dir.join(format!("{}.ex.json", slug));
    let json_data = models::config::Reference {
        name: mod_info.name.clone(),
        filename: file.file_name.clone(),
        side,
        link: models::config::Link::CurseForge {
            project_id: mod_info.id,
            file_id: file.id,
        },
    };

    let json_content =
        serde_json::to_string_pretty(&json_data).context("Failed to serialize mod JSON data")?;
    fs::write(&json_file_path, json_content).with_context(|| {
        format!(
            "Failed to write JSON reference file: {}",
            json_file_path.display()
        )
    })?;

    println!("✅ Mod reference added successfully!");
    println!("Note: The actual mod file will be downloaded when you build the modpack.");

    Ok(())
}
