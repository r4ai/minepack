use anyhow::{anyhow, Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use crate::models::config::ModLoader;
use crate::utils;
use crate::utils::errors::MinepackError;

// Supported export formats
enum ExportFormat {
    MultiMC,
    CurseForge,
    Modrinth,
}

pub async fn run() -> Result<()> {
    // Check if we're in a modpack directory
    if !utils::modpack_exists() {
        return Err(anyhow!(MinepackError::NoModpackFound));
    }

    let config = utils::load_config()?;

    println!("🔨 Building modpack: {}", config.name);

    // Create build directory
    let build_dir = PathBuf::from("build");
    utils::ensure_dir_exists(&build_dir)?;

    // Choose export format
    let format_options = ["MultiMC (.zip)", "CurseForge (.zip)", "Modrinth (mrpack)"];
    let format_index = dialoguer::Select::new()
        .with_prompt("Select export format")
        .items(&format_options)
        .default(0)
        .interact()
        .context("Failed to select export format")?;

    let format = match format_index {
        0 => ExportFormat::MultiMC,
        1 => ExportFormat::CurseForge,
        2 => ExportFormat::Modrinth,
        _ => return Err(anyhow!(MinepackError::InvalidExportFormat)),
    };

    // Set up progress bar
    let pb = ProgressBar::new(config.mods.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .context("Failed to create progress bar style")?
            .progress_chars("#>-"),
    );

    // Create modpack based on selected format
    match format {
        ExportFormat::MultiMC => build_multimc_pack(&config, &build_dir, pb).await?,
        ExportFormat::CurseForge => build_curseforge_pack(&config, &build_dir, pb).await?,
        ExportFormat::Modrinth => build_modrinth_pack(&config, &build_dir, pb).await?,
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

async fn build_multimc_pack(
    config: &crate::models::config::ModpackConfig,
    build_dir: &Path,
    pb: ProgressBar,
) -> Result<()> {
    // Create instance directory structure inside a temp directory
    let temp_dir = build_dir.join("temp_multimc");
    utils::ensure_dir_exists(&temp_dir)?;

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
        config.name, config.minecraft_version
    );
    fs::write(instance_dir.join("instance.cfg"), instance_cfg)
        .context("Failed to write instance.cfg")?;

    // Create mmc-pack.json
    let loader_name = match config.mod_loader {
        ModLoader::Forge => "net.minecraftforge",
        ModLoader::Fabric => "net.fabricmc.fabric-loader",
        ModLoader::Quilt => "org.quiltmc.quilt-loader",
    };

    let components = format!(
        r#"{{
        "components": [
            {{
                "uid": "net.minecraft",
                "version": "{}"
            }},
            {{
                "uid": "{}",
                "version": "0.0.0" 
            }}
        ]
    }}"#,
        config.minecraft_version, loader_name
    );

    fs::write(instance_dir.join("mmc-pack.json"), components)
        .context("Failed to write mmc-pack.json")?;

    // Copy all mods
    pb.set_message("Copying mod files");
    for mod_entry in &config.mods {
        let source_path =
            Path::new("mods").join(format!("{}-{}.jar", mod_entry.name, mod_entry.version));
        let target_path = mods_dir.join(format!("{}-{}.jar", mod_entry.name, mod_entry.version));

        // If the mod file exists locally, copy it, otherwise try to download it
        if source_path.exists() {
            fs::copy(&source_path, &target_path)
                .with_context(|| format!("Failed to copy mod file: {}", source_path.display()))?;
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

async fn build_curseforge_pack(
    config: &crate::models::config::ModpackConfig,
    build_dir: &Path,
    pb: ProgressBar,
) -> Result<()> {
    // Create directory structure inside a temp directory
    let temp_dir = build_dir.join("temp_curseforge");
    utils::ensure_dir_exists(&temp_dir)?;

    // Create manifest.json
    let loader_type = match config.mod_loader {
        ModLoader::Forge => "forge",
        ModLoader::Fabric => "fabric",
        ModLoader::Quilt => "quilt",
    };

    let mut manifest = String::from("{\n");
    manifest.push_str("  \"minecraft\": {\n");
    manifest.push_str(&format!(
        "    \"version\": \"{}\",\n",
        config.minecraft_version
    ));
    manifest.push_str("    \"modLoaders\": [\n");
    manifest.push_str("      {\n");
    manifest.push_str(&format!("        \"id\": \"{}-latest\",\n", loader_type));
    manifest.push_str("        \"primary\": true\n");
    manifest.push_str("      }\n");
    manifest.push_str("    ]\n");
    manifest.push_str("  },\n");
    manifest.push_str("  \"manifestType\": \"minecraftModpack\",\n");
    manifest.push_str("  \"manifestVersion\": 1,\n");
    manifest.push_str(&format!("  \"name\": \"{}\",\n", config.name));
    manifest.push_str(&format!("  \"version\": \"{}\",\n", config.version));
    manifest.push_str(&format!("  \"author\": \"{}\",\n", config.author));

    if let Some(desc) = &config.description {
        manifest.push_str(&format!("  \"description\": \"{}\",\n", desc));
    }

    // Add files list
    manifest.push_str("  \"files\": [\n");

    pb.set_message("Building manifest");
    for (i, mod_entry) in config.mods.iter().enumerate() {
        manifest.push_str("    {\n");
        manifest.push_str(&format!("      \"projectID\": {},\n", mod_entry.project_id));
        manifest.push_str(&format!("      \"fileID\": {},\n", mod_entry.file_id));
        manifest.push_str(&format!("      \"required\": {}\n", mod_entry.required));
        manifest.push_str(&format!(
            "    }}{}\n",
            if i < config.mods.len() - 1 { "," } else { "" }
        ));
        pb.inc(1);
    }

    manifest.push_str("  ]\n");
    manifest.push_str("}\n");

    fs::write(temp_dir.join("manifest.json"), manifest).context("Failed to write manifest.json")?;

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

async fn build_modrinth_pack(
    config: &crate::models::config::ModpackConfig,
    build_dir: &Path,
    pb: ProgressBar,
) -> Result<()> {
    // Create directory structure inside a temp directory
    let temp_dir = build_dir.join("temp_modrinth");
    utils::ensure_dir_exists(&temp_dir)?;

    // Create modrinth.index.json
    let loader_type = match config.mod_loader {
        ModLoader::Forge => "forge",
        ModLoader::Fabric => "fabric",
        ModLoader::Quilt => "quilt",
    };

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
    for (i, mod_entry) in config.mods.iter().enumerate() {
        index.push_str("    {\n");
        index.push_str(&format!(
            "      \"path\": \"mods/{}-{}.jar\",\n",
            mod_entry.name.replace(" ", "-"),
            mod_entry.version
        ));
        index.push_str("      \"hashes\": {},\n");
        index.push_str(&format!(
            "      \"downloads\": [\"{}\"],\n",
            mod_entry.download_url
        ));
        index.push_str("      \"fileSize\": 0\n");
        index.push_str(&format!(
            "    }}{}\n",
            if i < config.mods.len() - 1 { "," } else { "" }
        ));
        pb.inc(1);
    }

    index.push_str("  ],\n");
    index.push_str("  \"dependencies\": {\n");
    index.push_str(&format!(
        "    \"minecraft\": \"{}\",\n",
        config.minecraft_version
    ));
    index.push_str(&format!("    \"{}\": \"*\"\n", loader_type));
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

// Helper function to copy a directory recursively
fn copy_directory(src: &Path, dst: &Path) -> Result<()> {
    for entry in
        fs::read_dir(src).with_context(|| format!("Failed to read directory: {}", src.display()))?
    {
        let entry = entry
            .with_context(|| format!("Failed to read directory entry in: {}", src.display()))?;
        let ty = entry
            .file_type()
            .with_context(|| format!("Failed to get file type for: {}", entry.path().display()))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            utils::ensure_dir_exists(&dst_path)?;
            copy_directory(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path).with_context(|| {
                format!(
                    "Failed to copy file from {} to {}",
                    src_path.display(),
                    dst_path.display()
                )
            })?;
        }
    }
    Ok(())
}

// Helper function to create a zip archive from a directory
fn zip_directory(src_dir: &Path, dst_file: &Path) -> Result<()> {
    let file = File::create(dst_file)
        .with_context(|| format!("Failed to create zip file: {}", dst_file.display()))?;
    let writer = std::io::BufWriter::new(file);
    let options = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);
    let mut zip = zip::ZipWriter::new(writer);

    // Walk the directory and add files to the zip
    fn add_dir_to_zip(
        zip: &mut zip::ZipWriter<std::io::BufWriter<File>>,
        src_dir: &Path,
        base_path: &Path,
        options: &zip::write::FileOptions,
    ) -> Result<()> {
        for entry in fs::read_dir(src_dir)
            .with_context(|| format!("Failed to read directory: {}", src_dir.display()))?
        {
            let entry = entry.with_context(|| {
                format!("Failed to read directory entry in: {}", src_dir.display())
            })?;
            let path = entry.path();
            let name = path
                .strip_prefix(base_path)
                .with_context(|| format!("Failed to strip path prefix for: {}", path.display()))?;

            if path.is_file() {
                zip.start_file(name.to_string_lossy(), *options)
                    .context("Failed to start file in zip archive")?;
                let mut f = File::open(&path).with_context(|| {
                    format!("Failed to open file for zipping: {}", path.display())
                })?;
                std::io::copy(&mut f, zip).context("Failed to copy file data to zip archive")?;
            } else if path.is_dir() {
                zip.add_directory(name.to_string_lossy(), *options)
                    .context("Failed to add directory to zip archive")?;
                add_dir_to_zip(zip, &path, base_path, options)?;
            }
        }
        Ok(())
    }

    add_dir_to_zip(&mut zip, src_dir, src_dir, &options)?;
    zip.finish()
        .context("Failed to finish writing zip archive")?;

    Ok(())
}
