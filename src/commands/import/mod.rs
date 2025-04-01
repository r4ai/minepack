use anyhow::{anyhow, Context, Result};
use dialoguer::Confirm;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::{self, File};
use std::path::Path;
use tempfile::tempdir;
use zip::ZipArchive;

use crate::api::curseforge::{schema::Manifest, CurseforgeClient};
use crate::models::config::{self, Minecraft, ModLoader, ModpackConfig, Side};
use crate::utils::errors::MinepackError;
use crate::utils::{self, determine_mod_side_cf};

/// Extracted mod information from modlist.html or manifest.json
struct ModData {
    project_id: u32,
    file_id: u32,
    name: String,
    slug: String,
    file_name: Option<String>,
    download_url: Option<String>,
    side: Side,
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
    extract_zip(modpack_file_path, temp_dir.path())
        .context("Failed to extract modpack zip file")?;

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
    let mod_ids: Vec<u32> = manifest.files.iter().map(|file| file.project_id).collect();
    let file_ids: Vec<u32> = manifest.files.iter().map(|file| file.file_id).collect();
    let mod_infos = client.get_mod_infos(mod_ids).await?;
    let file_infos = client.get_mod_file_infos(file_ids).await?;
    for (index, file_entry) in manifest.files.iter().enumerate() {
        pb.set_message(format!(
            "Creating reference for mod: {}",
            file_entry.project_id
        ));

        // Get mod info from modlist if available, or try a minimal API call
        let mod_info = mod_infos.get(index).with_context(|| {
            format!(
                "Failed to get mod info for project ID: {}",
                file_entry.project_id
            )
        })?;

        // Create mod data with available information
        let file_info = &file_infos[index];
        let mod_data = ModData {
            project_id: file_entry.project_id,
            file_id: file_entry.file_id,
            name: mod_info.name.clone(),
            slug: mod_info.slug.clone(),
            side: determine_mod_side_cf(&mod_info.name, file_info)?,
            file_name: Some(file_info.file_name.clone()),
            download_url: file_info.download_url.clone(),
        };

        // Create JSON reference in mods directory
        create_mod_reference(&mod_data, &mods_dir, &pb)?;

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

/// Extracts the zip file to the destination directory
fn extract_zip(source: &Path, destination: &Path) -> Result<()> {
    // Open the zip file
    let file = File::open(source).context("Failed to open modpack file")?;
    let mut archive = ZipArchive::new(file).context("Failed to read modpack zip file")?;

    // Extract the zip file to the temporary directory
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).context("Failed to access zip entry")?;
        let outpath = destination.join(file.name());

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

    Ok(())
}

/// Creates a mod reference JSON file without downloading the actual mod
fn create_mod_reference(mod_data: &ModData, mods_dir: &Path, pb: &ProgressBar) -> Result<()> {
    // Generate a filename if not available
    let filename = match &mod_data.file_name {
        Some(name) => name.clone(),
        None => format!("{}-{}.jar", mod_data.slug, mod_data.file_id),
    };

    // Create the JSON reference file in the mods directory
    let json_file_path = mods_dir.join(format!("{}.ex.json", mod_data.slug));

    let json_data = config::Reference {
        name: mod_data.name.clone(),
        filename,
        side: mod_data.side.clone(),
        link: config::Link::CurseForge {
            project_id: mod_data.project_id,
            file_id: mod_data.file_id,
            download_url: mod_data.download_url.clone(),
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

    pb.println(format!(
        "✓ Created reference for mod: {} ({})",
        mod_data.name, mod_data.project_id
    ));

    Ok(())
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
