use anyhow::{anyhow, bail, Context, Result};
use dialoguer::{Confirm, Select};
use serde_json;
use std::fs;
use url::Url;

use crate::api::curseforge::schema::{Mod as CurseForgeModInfo, SearchModsRequestQuery};
use crate::api::curseforge::CurseforgeClient;
use crate::models::config::{Link, Reference};
use crate::utils::{determine_mod_side_cf, errors::MinepackError};
use crate::{api, models, utils};

/// Parse a CurseForge mod URL to extract the slug and optional file ID
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
        .search_mods(&SearchModsRequestQuery {
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
            // Get the complete mod info with a single API call
            let complete_mod_info = client
                .get_mod_info(mod_info.id)
                .await
                .context("Failed to fetch detailed mod information")?;
            Ok((complete_mod_info, file_id))
        }
        None => bail!(MinepackError::NoModsFound(slug.to_string())),
    }
}

/// Search for mods by name or keyword
async fn search_mods_by_name(
    query: &str,
    client: &CurseforgeClient,
    minecraft_version: &str,
) -> Result<CurseForgeModInfo> {
    println!("🔍 Searching for mods matching '{}'...", query);
    let search_results = client
        .search_mods(&SearchModsRequestQuery {
            search_filter: Some(query.to_string()),
            game_version: Some(minecraft_version.to_string()),
            ..Default::default()
        })
        .await
        .context("Failed to search for mods")?;

    if search_results.is_empty() {
        bail!(MinepackError::NoModsFound(query.to_string()));
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
    let selected_mod_id = search_results[selection].id;
    let complete_mod_info = client
        .get_mod_info(selected_mod_id)
        .await
        .context("Failed to fetch detailed mod information")?;

    Ok(complete_mod_info)
}

/// Select a compatible file version from the mod's available files
fn select_file_version<'a>(
    mod_info: &'a CurseForgeModInfo,
    minecraft_version: &str,
    file_id_from_url: Option<u32>,
) -> Result<&'a api::curseforge::schema::File> {
    // Filter for compatible files
    let compatible_files: Vec<_> = mod_info
        .latest_files
        .iter()
        .filter(|file| file.game_versions.contains(&minecraft_version.to_string()))
        .collect();

    if compatible_files.is_empty() {
        bail!(MinepackError::NoCompatibleModFiles(
            minecraft_version.to_string()
        ));
    }

    // If a specific file ID was provided in the URL, find that file
    if let Some(file_id) = file_id_from_url {
        // Try to find the specified file ID in compatible files
        if let Some(file_match) = compatible_files.iter().find(|f| f.id == file_id) {
            return Ok(file_match);
        }

        // If the specified file ID isn't compatible or doesn't exist
        println!(
            "Warning: The specified file ID {} is not compatible with Minecraft {}",
            file_id, minecraft_version
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
            bail!("Installation cancelled by user");
        }
    }

    // Default file selection logic
    if compatible_files.len() == 1 {
        return Ok(compatible_files[0]);
    }

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

    Ok(compatible_files[file_selection])
}

/// Create and save the mod reference file
fn save_mod_reference(
    env: &impl utils::Env,
    mod_info: &CurseForgeModInfo,
    file: &api::curseforge::schema::File,
    side: models::config::Side,
) -> Result<()> {
    // Ensure the mods directory exists
    let mods_dir = utils::get_mods_dir(env)?;
    utils::ensure_dir_exists(&mods_dir)?;

    // Get the slug for the mod and use it in the JSON filename
    let slug = if mod_info.slug.is_empty() {
        // If slug is empty, create a slug from the name
        mod_info.name.to_lowercase().replace(' ', "-")
    } else {
        mod_info.slug.clone()
    };

    // Create the JSON reference file in the mods directory
    let json_file_path = mods_dir.join(format!("{}.ex.json", slug));
    let json_data = Reference {
        name: mod_info.name.clone(),
        filename: file.file_name.clone(),
        side,
        link: Link::CurseForge {
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

    Ok(())
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

    // Process the query - it's either a URL or a search term
    let (mod_info, file_id_from_url) = if query.starts_with("https://www.curseforge.com/") {
        // Extract mod info from URL
        extract_mod_info_from_url(&query, &client, &config.minecraft.version).await?
    } else {
        // Search for mods by name
        let mod_info = search_mods_by_name(&query, &client, &config.minecraft.version).await?;
        (mod_info, None)
    };

    // Display the selected mod info
    println!("Selected mod: {} (ID: {})", mod_info.name, mod_info.id);
    println!("Description: {}", mod_info.summary);

    // Select a compatible file version
    let file = select_file_version(&mod_info, &config.minecraft.version, file_id_from_url)?;
    println!("Selected file: {} (ID: {})", file.display_name, file.id);

    // Determine the mod side (client/server/both)
    let side = determine_mod_side_cf(&mod_info.name, file)?;

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

    // Save the mod reference file
    save_mod_reference(env, &mod_info, file, side)?;

    println!("✅ Mod reference added successfully!");
    println!("Note: The actual mod file will be downloaded when you build the modpack.");

    Ok(())
}
