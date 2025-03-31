use anyhow::{anyhow, Context, Result};
use dialoguer::Confirm;
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::json;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;
use zip::ZipArchive;

use crate::api::curseforge::{schema::Manifest, CurseforgeClient};
use crate::models::config::{Minecraft, ModLoader, ModpackConfig};
use crate::utils;
use crate::utils::errors::MinepackError;

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

    // Initialize CurseForge client for mod downloads
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

    // Download all the mods
    println!("Downloading mods...");
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

    // Ensure .minepack/cache/mods directory exists
    let cache_dir = utils::get_minepack_cache_mods_dir(env)?;
    utils::ensure_dir_exists(&cache_dir)?;

    // Process each mod in the manifest
    for file_entry in &manifest.files {
        pb.set_message(format!(
            "Processing mod project ID: {}",
            file_entry.project_id
        ));

        // Get mod info
        let mod_info = match client.get_mod_info(file_entry.project_id).await {
            Ok(info) => info,
            Err(e) => {
                pb.println(format!(
                    "Failed to get mod info for project ID {}: {}",
                    file_entry.project_id, e
                ));
                continue;
            }
        };

        // Get mod file
        let file = match mod_info
            .latest_files
            .iter()
            .find(|f| f.id == file_entry.file_id)
        {
            Some(f) => f.clone(),
            None => {
                // If file not found in latest_files, try to download it anyway using the ID
                pb.println(format!(
                    "File ID {} not found in latest_files for mod {}, trying direct download...",
                    file_entry.file_id, mod_info.name
                ));

                // Download the mod file data
                let mod_data = match client
                    .download_mod_file(file_entry.project_id, file_entry.file_id)
                    .await
                {
                    Ok(data) => data,
                    Err(e) => {
                        pb.println(format!("Failed to download mod {}: {}", mod_info.name, e));
                        continue;
                    }
                };

                // Generate a filename based on mod name and file ID
                let filename = format!(
                    "{}-{}.jar",
                    mod_info.slug.replace(" ", "-"),
                    file_entry.file_id
                );

                // Save the mod to cache
                let cache_file_path = cache_dir.join(&filename);
                let mut file_handle = File::create(&cache_file_path).with_context(|| {
                    format!("Failed to create cache file: {}", cache_file_path.display())
                })?;
                file_handle
                    .write_all(&mod_data)
                    .context("Failed to write mod data to cache file")?;

                // Create mod entry
                let slug = if mod_info.slug.is_empty() {
                    mod_info.name.to_lowercase().replace(' ', "-")
                } else {
                    mod_info.slug
                };

                // Create JSON reference in mods directory
                let json_file_path = mods_dir.join(format!("{}.ex.json", slug));
                let json_data = json!({
                    "name": mod_info.name,
                    "filename": filename,
                    "side": "both", // Default to "both" when we can't determine
                    "link": {
                        "site": "curseforge",
                        "project_id": mod_info.id,
                        "file_id": file_entry.file_id,
                    }
                });

                let json_content = serde_json::to_string_pretty(&json_data)
                    .context("Failed to serialize mod JSON data")?;
                fs::write(&json_file_path, json_content).with_context(|| {
                    format!(
                        "Failed to write JSON reference file: {}",
                        json_file_path.display()
                    )
                })?;

                pb.inc(1);
                continue;
            }
        };

        // Download the mod file data if needed
        let mod_data = client
            .download_mod_file(file_entry.project_id, file_entry.file_id)
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
            mod_info.name.to_lowercase().replace(' ', "-")
        } else {
            mod_info.slug
        };

        // Create the JSON reference file in the mods directory
        let json_file_path = mods_dir.join(format!("{}.ex.json", slug));
        let json_data = json!({
            "name": mod_info.name,
            "filename": file.file_name,
            "side": determine_mod_side(&mod_info.name, &file.file_name)?,
            "link": {
                "site": "curseforge",
                "project_id": mod_info.id,
                "file_id": file.id,
            }
        });

        let json_content = serde_json::to_string_pretty(&json_data)
            .context("Failed to serialize mod JSON data")?;
        fs::write(&json_file_path, json_content).with_context(|| {
            format!(
                "Failed to write JSON reference file: {}",
                json_file_path.display()
            )
        })?;

        pb.inc(1);
    }
    pb.finish_with_message("All mods processed successfully");

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
