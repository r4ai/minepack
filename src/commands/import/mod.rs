use anyhow::{anyhow, Context, Result};
use dialoguer::Confirm;
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::json;
use std::collections::HashMap;
use std::fs::{self, File};
use std::path::Path;
use tempfile::tempdir;
use zip::ZipArchive;

use crate::api::curseforge::{schema::Manifest, CurseforgeClient};
use crate::models::config::{Minecraft, ModLoader, ModpackConfig};
use crate::utils;
use crate::utils::errors::MinepackError;

/// Extracted mod information from modlist.html or manifest.json
struct ModInfo {
    project_id: u32,
    file_id: u32,
    name: Option<String>,
    file_name: Option<String>,
}

/// Import a modpack from a CurseForge zip file
pub async fn run<E: utils::Env>(env: &E, modpack_path: String, yes: bool) -> Result<()> {
    // Check if we're in a modpack directory and confirm overwrite if exists
    if utils::modpack_exists(env) && !yes {
        let confirm = Confirm::new()
            .with_prompt("A modpack already exists in this directory. Do you want to overwrite it?")
            .default(false)
            .interact()
            .context("Failed to confirm overwrite")?;

        if !confirm {
            return Ok(());
        }
    }

    println!("📦 Importing modpack from {}", modpack_path);

    // Validate the file exists
    let modpack_file_path = Path::new(&modpack_path);
    if !modpack_file_path.exists() {
        return Err(anyhow!(MinepackError::FileNotFound(modpack_path)));
    }

    // Check if it's a zip file
    if modpack_file_path.extension().and_then(|ext| ext.to_str()) != Some("zip") {
        return Err(anyhow!(MinepackError::InvalidFileFormat(
            "Only CurseForge ZIP files are supported".to_string()
        )));
    }

    // Initialize CurseForge client for API operations (only used when absolutely necessary)
    let client = CurseforgeClient::new().context("Failed to initialize CurseForge API client")?;

    // Create a temporary directory to extract the modpack
    let temp_dir = tempdir().context("Failed to create temporary directory")?;
    println!("Extracting modpack to temporary directory...");

    // Open the zip file
    let file = File::open(modpack_file_path).context("Failed to open modpack file")?;
    let mut archive = ZipArchive::new(file).context("Failed to read modpack zip file")?;

    // Extract the zip file to the temporary directory
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).context("Failed to access zip entry")?;
        let outpath = temp_dir.path().join(file.name());

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath).with_context(|| {
                format!(
                    "Failed to create directory for extraction: {}",
                    outpath.display()
                )
            })?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).with_context(|| {
                        format!(
                            "Failed to create parent directory for extraction: {}",
                            p.display()
                        )
                    })?;
                }
            }
            let mut outfile = File::create(&outpath).with_context(|| {
                format!(
                    "Failed to create file for extraction: {}",
                    outpath.display()
                )
            })?;
            std::io::copy(&mut file, &mut outfile).context("Failed to extract file content")?;
        }
    }

    // Check if manifest.json exists in the extracted files
    let manifest_path = temp_dir.path().join("manifest.json");
    if !manifest_path.exists() {
        return Err(anyhow!(MinepackError::InvalidFileFormat(
            "No manifest.json found in the modpack file. Is this a valid CurseForge modpack?"
                .to_string()
        )));
    }

    // Parse the manifest
    let manifest_content =
        fs::read_to_string(&manifest_path).context("Failed to read manifest.json")?;
    let manifest: Manifest =
        serde_json::from_str(&manifest_content).context("Failed to parse manifest.json")?;

    // Create the ModpackConfig from the manifest
    let config = ModpackConfig::new(
        manifest.name,
        manifest.version,
        manifest.author,
        None, // CurseForge manifest doesn't include a description
        Minecraft::new(
            manifest.minecraft.version,
            manifest
                .minecraft
                .mod_loaders
                .iter()
                .map(|loader| {
                    let parts: Vec<&str> = loader.id.split('-').collect();
                    let loader_id = if parts.len() > 1 {
                        parts[0].to_string()
                    } else {
                        loader.id.clone()
                    };
                    let version = if parts.len() > 1 {
                        parts[1].to_string()
                    } else {
                        "".to_string()
                    };

                    ModLoader {
                        id: loader_id,
                        version,
                        primary: loader.primary,
                    }
                })
                .collect(),
        ),
    );

    // Create modpack structure
    utils::create_modpack_structure(env)?;

    // Save the configuration file
    utils::save_config(env, &config)?;

    // Check if modlist.html exists and parse it
    let modlist_path = temp_dir.path().join("modlist.html");
    let mod_details = if modlist_path.exists() {
        println!("Found modlist.html, extracting mod details...");
        parse_modlist_html(&modlist_path)?
    } else {
        println!("No modlist.html found, will use only manifest.json information.");
        HashMap::new()
    };

    // Process mods from manifest
    println!("Processing mods...");
    let pb = ProgressBar::new(manifest.files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .context("Failed to create progress bar style")?
            .progress_chars("#>-"),
    );

    // Ensure the mods directory exists
    let mods_dir = utils::get_mods_dir(env)?;
    utils::ensure_dir_exists(&mods_dir)?;

    // Process each mod in the manifest
    for file_entry in &manifest.files {
        pb.set_message(format!(
            "Creating reference for mod: {}",
            file_entry.project_id
        ));

        // Get mod info from modlist if available, or try a minimal API call
        let mod_info = create_mod_info(file_entry, &mod_details, &client).await;

        // Create JSON reference in mods directory
        create_mod_reference(&mod_info, &mods_dir, &pb)?;

        pb.inc(1);
    }
    pb.finish_with_message("All mod references created successfully");

    // Copy overrides content if it exists
    let overrides_dir = temp_dir.path().join("overrides");
    if overrides_dir.exists() && overrides_dir.is_dir() {
        println!("Copying overrides...");
        copy_overrides(&overrides_dir, env)?;
    }

    println!("✅ Modpack imported successfully!");
    println!("Name: {}", config.name);
    println!("Version: {}", config.version);
    println!("Minecraft Version: {}", config.minecraft.version);
    println!(
        "Mod Loader: {}-{}",
        config.minecraft.mod_loaders[0].id, config.minecraft.mod_loaders[0].version
    );

    // Note to user about mod references
    println!("\n⚠️  Note: Mod files have not been downloaded yet.");
    println!("   References to the mods have been created, but the actual jar files");
    println!("   will be downloaded on demand when you build or use the modpack.");

    Ok(())
}

/// Parses the modlist.html file to extract mod information
fn parse_modlist_html(modlist_path: &Path) -> Result<HashMap<u32, ModInfo>> {
    let content = fs::read_to_string(modlist_path).context("Failed to read modlist.html")?;
    let mut mod_details = HashMap::new();

    // Simple but effective parsing of modlist.html
    // Looking for patterns like: <li><a href="https://www.curseforge.com/minecraft/mc-mods/jei">Just Enough Items (JEI) (by mezz)</a></li>
    for line in content.lines() {
        // Skip lines that don't contain mod links
        if !line.contains("curseforge.com/minecraft/mc-mods/") {
            continue;
        }

        // Extract project ID and name from the line
        if let Some(href_start) = line.find("href=\"https://www.curseforge.com/minecraft/mc-mods/")
        {
            let href_end = match line[href_start..].find("\">") {
                Some(pos) => href_start + pos,
                None => continue,
            };

            let url = &line[href_start + 6..href_end];

            // Extract the mod slug from the URL
            let slug = match url.split('/').collect::<Vec<&str>>().get(5) {
                Some(slug) => *slug,
                None => continue,
            };

            // Extract the mod name from the link text
            let name_start = href_end + 2;
            let name_end = match line[name_start..].find("</a>") {
                Some(pos) => name_start + pos,
                None => continue,
            };

            let full_name = &line[name_start..name_end];
            // Handle cases where the name includes author like "Mod Name (by AuthorName)"
            let name = match full_name.find(" (by ") {
                Some(pos) => &full_name[0..pos],
                None => full_name,
            };

            // We don't have project_id directly from modlist.html, but we'll use it as a key later
            // For now, store with a dummy project_id of 0, we'll match by name/slug later
            mod_details.insert(
                slug.to_string(),
                ModInfo {
                    project_id: 0, // Will be filled in later when matching with manifest.json
                    file_id: 0,    // Will be filled in later
                    name: Some(name.to_string()),
                    file_name: None,
                },
            );
        }
    }

    // より効率的な方法でマップ値の処理を行う
    Ok(mod_details
        .into_values()
        .map(|info| (info.project_id, info))
        .collect())
}

/// Creates mod information from manifest and modlist, making API calls only when necessary
async fn create_mod_info(
    file_entry: &crate::api::curseforge::schema::ManifestFile,
    mod_details: &HashMap<u32, ModInfo>,
    client: &CurseforgeClient,
) -> ModInfo {
    // If we already have details from modlist.html, use those
    if let Some(details) = mod_details.get(&file_entry.project_id) {
        return ModInfo {
            project_id: file_entry.project_id,
            file_id: file_entry.file_id,
            name: details.name.clone(),
            file_name: details.file_name.clone(),
        };
    }

    // Otherwise, try a minimal API call to get mod name
    // If it fails, we'll just use placeholders
    match client.get_mod_info(file_entry.project_id).await {
        Ok(mod_info) => ModInfo {
            project_id: file_entry.project_id,
            file_id: file_entry.file_id,
            name: Some(mod_info.name),
            file_name: mod_info
                .latest_files
                .iter()
                .find(|f| f.id == file_entry.file_id)
                .map(|f| f.file_name.clone()),
        },
        Err(_) => {
            // If API call fails, use a generic name based on project ID
            ModInfo {
                project_id: file_entry.project_id,
                file_id: file_entry.file_id,
                name: Some(format!("Unknown Mod (ID: {})", file_entry.project_id)),
                file_name: None,
            }
        }
    }
}

/// Creates a mod reference JSON file without downloading the actual mod
fn create_mod_reference(mod_info: &ModInfo, mods_dir: &Path, pb: &ProgressBar) -> Result<()> {
    // Generate a slug from the name or use project ID if name is not available
    let slug = match &mod_info.name {
        Some(name) => name.to_lowercase().replace(' ', "-"),
        None => format!("mod-{}", mod_info.project_id),
    };

    // Generate a filename if not available
    let filename = match &mod_info.file_name {
        Some(name) => name.clone(),
        None => format!("{}-{}.jar", slug, mod_info.file_id),
    };

    // Create the JSON reference file in the mods directory
    let json_file_path = mods_dir.join(format!("{}.ex.json", slug));

    // Determine mod side (client/server/both) if possible
    let side = determine_mod_side(mod_info.name.as_deref().unwrap_or(""), &filename)?;

    let json_data = json!({
        "name": mod_info.name.clone().unwrap_or_else(|| format!("Mod {}", mod_info.project_id)),
        "filename": filename,
        "side": side,
        "link": {
            "site": "curseforge",
            "project_id": mod_info.project_id,
            "file_id": mod_info.file_id,
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

    pb.println(format!(
        "✓ Created reference for mod: {} ({})",
        mod_info.name.as_deref().unwrap_or("Unknown"),
        mod_info.project_id
    ));

    Ok(())
}

/// Determines which side (client/server/both) the mod is meant for
fn determine_mod_side(mod_name: &str, filename: &str) -> Result<String> {
    let name_lower = mod_name.to_lowercase();
    let filename_lower = filename.to_lowercase();

    // Check for obvious server-side or client-side keywords
    if name_lower.contains("server") || filename_lower.contains("server") {
        return Ok("server".to_string());
    } else if name_lower.contains("client") || filename_lower.contains("client") {
        return Ok("client".to_string());
    }

    // Default to both sides if no specific indication
    Ok("both".to_string())
}

/// Copy the overrides content to the appropriate locations in the modpack
fn copy_overrides(overrides_dir: &Path, env: &impl utils::Env) -> Result<()> {
    let current_dir = env.current_dir()?;

    // Walk through the overrides directory
    for entry in walkdir::WalkDir::new(overrides_dir) {
        let entry = entry.context("Failed to read override entry")?;
        let path = entry.path();

        // Skip the root directory itself
        if path == overrides_dir {
            continue;
        }

        // Get the relative path from the overrides directory
        let relative_path = path
            .strip_prefix(overrides_dir)
            .context("Failed to strip prefix from override path")?;
        let target_path = current_dir.join(relative_path);

        // Create the directory or copy the file
        if path.is_dir() {
            fs::create_dir_all(&target_path).with_context(|| {
                format!("Failed to create directory: {}", target_path.display())
            })?;
        } else {
            // Ensure parent directory exists
            if let Some(parent) = target_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent).with_context(|| {
                        format!("Failed to create parent directory: {}", parent.display())
                    })?;
                }
            }

            fs::copy(path, &target_path)
                .with_context(|| format!("Failed to copy file to: {}", target_path.display()))?;
        }
    }

    Ok(())
}
