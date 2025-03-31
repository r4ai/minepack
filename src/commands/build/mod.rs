use anyhow::{anyhow, Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::Value;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use walkdir::WalkDir;

use crate::api::curseforge::{
    schema::{Manifest, ManifestFile, ManifestMinecraft, ManifestModLoader},
    CurseforgeClient,
};
use crate::models::config::ModEntry;
use crate::utils;
use crate::utils::errors::MinepackError;

// Supported export formats
enum ExportFormat {
    MultiMC,
    CurseForge,
    Modrinth,
}

pub async fn run<E: utils::Env>(env: &E, format: Option<String>) -> Result<()> {
    // Check if we're in a modpack directory
    if !utils::modpack_exists(env) {
        return Err(anyhow!(MinepackError::NoModpackFound));
    }

    let config = utils::load_config(env)?;

    println!("🔨 Building modpack: {}", config.name);

    // Create build directory
    let build_dir = utils::get_build_dir(env)?;
    utils::ensure_dir_exists(&build_dir)?;

    // Determine export format from command line argument or prompt user
    let export_format = match format {
        Some(format_str) => match format_str.to_lowercase().as_str() {
            "multimc" => ExportFormat::MultiMC,
            "curseforge" => ExportFormat::CurseForge,
            "modrinth" => ExportFormat::Modrinth,
            _ => return Err(anyhow!(MinepackError::InvalidExportFormat)),
        },
        None => {
            // Choose export format via prompt if not specified
            let format_options = ["MultiMC (.zip)", "CurseForge (.zip)", "Modrinth (mrpack)"];
            let format_index = dialoguer::Select::new()
                .with_prompt("Select export format")
                .items(&format_options)
                .default(0)
                .interact()
                .context("Failed to select export format")?;

            match format_index {
                0 => ExportFormat::MultiMC,
                1 => ExportFormat::CurseForge,
                2 => ExportFormat::Modrinth,
                _ => return Err(anyhow!(MinepackError::InvalidExportFormat)),
            }
        }
    };

    // Load all mod entries from JSON files
    let mod_entries = load_mod_entries(env).context("Failed to load mod entries")?;

    // Set up progress bar
    let pb = ProgressBar::new(mod_entries.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .context("Failed to create progress bar style")?
            .progress_chars("#>-"),
    );

    // Create modpack based on selected format
    match export_format {
        ExportFormat::MultiMC => {
            build_multimc_pack(env, &config, &build_dir, &mod_entries, pb).await?
        }
        ExportFormat::CurseForge => {
            build_curseforge_pack(env, &config, &build_dir, &mod_entries, pb).await?
        }
        ExportFormat::Modrinth => {
            build_modrinth_pack(env, &config, &build_dir, &mod_entries, pb).await?
        }
    }

    println!("✅ Modpack built successfully!");
    println!(
        "Output: {}/{}-{}.zip",
        build_dir.display(),
        config.name,
        config.version
    );

    Ok(())
}

/// Load mod entries from the JSON files in the mods directory
fn load_mod_entries<E: utils::Env>(env: &E) -> Result<Vec<ModEntry>> {
    let mods_dir = utils::get_mods_dir(env)?;
    let mut mod_entries = Vec::new();

    for entry in WalkDir::new(&mods_dir).min_depth(1).max_depth(1) {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        // Only process JSON files with .ex.json extension
        if path.extension().is_some_and(|ext| ext == "json")
            && path.to_string_lossy().ends_with(".ex.json")
        {
            // Read and parse the JSON file
            let mut file = File::open(path)
                .with_context(|| format!("Failed to open JSON file: {}", path.display()))?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .with_context(|| format!("Failed to read JSON file: {}", path.display()))?;

            let json: Value = serde_json::from_str(&contents)
                .with_context(|| format!("Failed to parse JSON file: {}", path.display()))?;

            // Create ModEntry from new JSON format
            let name = json["name"].as_str().unwrap_or("Unknown").to_string();
            let filename = json["filename"].as_str().unwrap_or("Unknown").to_string();

            // Get project_id and file_id from the link object
            let project_id = json["link"]["project_id"].as_u64().unwrap_or(0) as u32;
            let file_id = json["link"]["file_id"].as_u64().unwrap_or(0) as u32;

            // Construct download_url if not present (fallback for older JSON files)
            let download_url = format!(
                "https://edge.forgecdn.net/files/{}/{}/{}",
                file_id / 1000,
                file_id % 1000,
                filename
            );

            let mod_entry = ModEntry {
                name,
                project_id,
                file_id,
                version: filename.clone(), // Use filename as version if not specified
                download_url,
                required: true, // Default to required
            };

            mod_entries.push(mod_entry);
        }
    }

    Ok(mod_entries)
}

async fn build_multimc_pack<E: utils::Env>(
    env: &E,
    config: &crate::models::config::ModpackConfig,
    build_dir: &Path,
    mod_entries: &[ModEntry],
    pb: ProgressBar,
) -> Result<()> {
    // Create instance directory structure inside a temp directory
    let temp_dir = build_dir.join("temp_multimc");
    utils::ensure_dir_exists(&temp_dir)?;

    // Initialize CurseForge client for potential mod downloads
    let client = CurseforgeClient::new().context("Failed to initialize Curseforge API client")?;

    // Create MultiMC instance structure
    let instance_dir = temp_dir.join(&config.name);
    utils::ensure_dir_exists(&instance_dir)?;

    // Create .minecraft and mods directories
    let minecraft_dir = instance_dir.join(".minecraft");
    utils::ensure_dir_exists(&minecraft_dir)?;
    let mods_dir = minecraft_dir.join("mods");
    utils::ensure_dir_exists(&mods_dir)?;

    // Copy configuration files if they exist
    if Path::new("config").exists() {
        let config_dir = minecraft_dir.join("config");
        utils::ensure_dir_exists(&config_dir)?;
        copy_directory(Path::new("config"), &config_dir)
            .context("Failed to copy configuration files")?;
    }

    // Create instance.cfg
    let instance_cfg = format!(
        "InstanceType=OneSix\nname={}\nIntendedVersion={}\n",
        config.name, config.minecraft.version
    );
    fs::write(instance_dir.join("instance.cfg"), instance_cfg)
        .context("Failed to write instance.cfg")?;

    // Create mmc-pack.json
    let loader_name = match &config.minecraft.mod_loaders[0].id[..] {
        "forge" => "net.minecraftforge",
        "fabric" => "net.fabricmc.fabric-loader",
        "quilt" => "org.quiltmc.quilt-loader",
        "neoforge" => "net.neoforged",
        _ => "unknown",
    };

    // Get loader version (now required)
    let loader_version = &config.minecraft.mod_loaders[0].version;

    let components = format!(
        r#"{{
        "components": [
            {{
                "uid": "net.minecraft",
                "version": "{}"
            }},
            {{
                "uid": "{}",
                "version": "{}"
            }}
        ]
    }}"#,
        config.minecraft.version, loader_name, loader_version
    );

    fs::write(instance_dir.join("mmc-pack.json"), components)
        .context("Failed to write mmc-pack.json")?;

    // Copy all mods from cache
    pb.set_message("Copying mod files");
    let cache_mods_dir = utils::get_minepack_cache_mods_dir(env)?;

    for mod_entry in mod_entries {
        let filename = &mod_entry.version; // filename is stored in version field now
        let cache_path = cache_mods_dir.join(filename);
        let target_path = mods_dir.join(filename);

        // If the mod file exists in cache, copy it
        if cache_path.exists() {
            fs::copy(&cache_path, &target_path)
                .with_context(|| format!("Failed to copy mod file: {}", cache_path.display()))?;
        } else {
            // Try to download it if not found in cache
            pb.set_message(format!("Mod file not in cache, downloading: {}", filename));
            // Download URL is constructed from project_id and file_id
            let data = client
                .download_mod_file(mod_entry.project_id, mod_entry.file_id)
                .await
                .with_context(|| format!("Failed to download mod: {}", mod_entry.name))?;

            // Save to target directly
            let mut file = File::create(&target_path)
                .with_context(|| format!("Failed to create file: {}", target_path.display()))?;
            file.write_all(&data)
                .context("Failed to write mod data to file")?;
        }

        pb.inc(1);
    }

    // Create zip archive
    let output_path = build_dir.join(format!("{}-{}-MultiMC.zip", config.name, config.version));
    zip_directory(&temp_dir, &output_path).context("Failed to create zip archive")?;

    // Clean up temp directory
    fs::remove_dir_all(temp_dir).context("Failed to clean up temporary directory")?;

    pb.finish_with_message(format!("Built MultiMC pack: {}", output_path.display()));
    Ok(())
}

async fn build_curseforge_pack<E: utils::Env>(
    _env: &E,
    config: &crate::models::config::ModpackConfig,
    build_dir: &Path,
    mod_entries: &[ModEntry],
    pb: ProgressBar,
) -> Result<()> {
    // Initialize CurseForge client for potential mod downloads
    let _client = CurseforgeClient::new().context("Failed to initialize Curseforge API client")?;

    // Create directory structure inside a temp directory
    let temp_dir = build_dir.join("temp_curseforge");
    utils::ensure_dir_exists(&temp_dir)?;

    // Convert loader type to string
    let loader_type = &config.minecraft.mod_loaders[0].id;

    // Create manifest using proper types
    pb.set_message("Building manifest");

    // Create mod loader entry - use the required version
    let loader_id = format!(
        "{}-{}",
        loader_type, &config.minecraft.mod_loaders[0].version
    );

    // Create mod loader entry
    let mod_loader = ManifestModLoader {
        id: loader_id,
        primary: config.minecraft.mod_loaders[0].primary,
    };

    // Create list of manifest files from mod entries
    let manifest_files: Vec<ManifestFile> = mod_entries
        .iter()
        .map(|entry| {
            pb.inc(1);
            ManifestFile {
                project_id: entry.project_id,
                file_id: entry.file_id,
                required: entry.required,
            }
        })
        .collect();

    // Create manifest structure
    let manifest = Manifest {
        minecraft: ManifestMinecraft {
            version: config.minecraft.version.clone(),
            mod_loaders: vec![mod_loader],
        },
        manifest_type: "minecraftModpack".to_string(),
        manifest_version: 1,
        name: config.name.clone(),
        version: config.version.clone(),
        author: config.author.clone(),
        files: manifest_files,
        overrides: "overrides".to_string(),
    };

    // Serialize manifest to JSON and write to file
    let manifest_json =
        serde_json::to_string_pretty(&manifest).context("Failed to serialize manifest to JSON")?;
    fs::write(temp_dir.join("manifest.json"), manifest_json)
        .context("Failed to write manifest.json")?;

    // Create overrides directory for configs, etc.
    let overrides_dir = temp_dir.join("overrides");
    utils::ensure_dir_exists(&overrides_dir)?;

    // Copy config directory if it exists
    if Path::new("config").exists() {
        let config_dir = overrides_dir.join("config");
        utils::ensure_dir_exists(&config_dir)?;
        copy_directory(Path::new("config"), &config_dir)
            .context("Failed to copy configuration files")?;
    }

    // Create zip archive
    let output_path = build_dir.join(format!("{}-{}-CurseForge.zip", config.name, config.version));
    zip_directory(&temp_dir, &output_path).context("Failed to create zip archive")?;

    // Clean up temp directory
    fs::remove_dir_all(temp_dir).context("Failed to clean up temporary directory")?;

    pb.finish_with_message(format!("Built CurseForge pack: {}", output_path.display()));
    Ok(())
}

async fn build_modrinth_pack<E: utils::Env>(
    _env: &E,
    config: &crate::models::config::ModpackConfig,
    build_dir: &Path,
    mod_entries: &[ModEntry],
    pb: ProgressBar,
) -> Result<()> {
    // Initialize CurseForge client for potential mod downloads
    let _client = CurseforgeClient::new().context("Failed to initialize Curseforge API client")?;

    // Create directory structure inside a temp directory
    let temp_dir = build_dir.join("temp_modrinth");
    utils::ensure_dir_exists(&temp_dir)?;

    // Create modrinth.index.json
    let loader_type = &config.minecraft.mod_loaders[0].id;

    // Use mod loader version (now required)
    let loader_version = &config.minecraft.mod_loaders[0].version;

    let mut index = String::from("{\n");
    index.push_str("  \"formatVersion\": 1,\n");
    index.push_str("  \"game\": \"minecraft\",\n");
    index.push_str(&format!("  \"versionId\": \"{}\",\n", config.version));
    index.push_str(&format!("  \"name\": \"{}\",\n", config.name));

    if let Some(desc) = &config.description {
        index.push_str(&format!("  \"summary\": \"{}\",\n", desc));
    }

    index.push_str("  \"files\": [\n");

    pb.set_message("Building Modrinth index");
    for (i, mod_entry) in mod_entries.iter().enumerate() {
        // Use filename from the version field as it now contains filename
        let filename = &mod_entry.version;

        index.push_str("    {\n");
        index.push_str(&format!("      \"path\": \"mods/{}\",\n", filename));
        index.push_str("      \"hashes\": {},\n");
        index.push_str(&format!(
            "      \"downloads\": [\"{}\"],\n",
            mod_entry.download_url
        ));
        index.push_str("      \"fileSize\": 0\n");
        index.push_str(&format!(
            "    }}{}\n",
            if i < mod_entries.len() - 1 { "," } else { "" }
        ));
        pb.inc(1);
    }

    index.push_str("  ],\n");
    index.push_str("  \"dependencies\": {\n");
    index.push_str(&format!(
        "    \"minecraft\": \"{}\",\n",
        config.minecraft.version
    ));
    index.push_str(&format!(
        "    \"{}\": \"{}\"\n",
        loader_type, loader_version
    ));
    index.push_str("  }\n");
    index.push_str("}\n");

    fs::write(temp_dir.join("modrinth.index.json"), index)
        .context("Failed to write modrinth.index.json")?;

    // Create overrides directory for configs
    let overrides_dir = temp_dir.join("overrides");
    utils::ensure_dir_exists(&overrides_dir)?;

    // Copy config directory if it exists
    if Path::new("config").exists() {
        let config_dir = overrides_dir.join("config");
        utils::ensure_dir_exists(&config_dir)?;
        copy_directory(Path::new("config"), &config_dir)
            .context("Failed to copy configuration files")?;
    }

    // Create zip archive (with .mrpack extension)
    let output_path = build_dir.join(format!("{}-{}.mrpack", config.name, config.version));
    zip_directory(&temp_dir, &output_path).context("Failed to create mrpack archive")?;

    // Clean up temp directory
    fs::remove_dir_all(temp_dir).context("Failed to clean up temporary directory")?;

    pb.finish_with_message(format!("Built Modrinth pack: {}", output_path.display()));
    Ok(())
}

// Copy a directory recursively
fn copy_directory(src: &Path, dst: &Path) -> Result<()> {
    for entry in WalkDir::new(src) {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();
        let relative_path = path.strip_prefix(src).context("Failed to strip prefix")?;
        let target_path = dst.join(relative_path);

        if path.is_dir() {
            fs::create_dir_all(&target_path).context("Failed to create directory")?;
        } else {
            fs::copy(path, &target_path).context("Failed to copy file")?;
        }
    }
    Ok(())
}

// Create a zip archive from a directory
fn zip_directory(src_dir: &Path, dst_file: &Path) -> Result<()> {
    let file = File::create(dst_file)
        .with_context(|| format!("Failed to create zip file: {}", dst_file.display()))?;

    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    // Add files to the zip archive
    for entry in WalkDir::new(src_dir) {
        let entry = entry.context("Failed to read file for zipping")?;
        let path = entry.path();
        let relative_path = path
            .strip_prefix(src_dir)
            .context("Failed to strip prefix for zip entry")?;

        if path.is_dir() {
            let path_string = relative_path
                .to_str()
                .ok_or_else(|| anyhow!("Invalid path"))?;
            let dir_path = if path_string.is_empty() {
                continue;
            } else {
                format!("{}/", path_string)
            };
            zip.add_directory(&dir_path, options)
                .context("Failed to add directory to zip")?;
        } else {
            let mut file = File::open(path)
                .with_context(|| format!("Failed to open file: {}", path.display()))?;
            zip.start_file(
                relative_path
                    .to_str()
                    .ok_or_else(|| anyhow!("Invalid path"))?,
                options,
            )
            .context("Failed to add file to zip")?;
            std::io::copy(&mut file, &mut zip).context("Failed to copy file contents to zip")?;
        }
    }

    zip.finish().context("Failed to write zip file")?;
    Ok(())
}
