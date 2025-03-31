#[cfg(feature = "mock")]
mod tests {
    use anyhow::{Context, Result};
    use minepack::utils::Env;
    use std::fs;

    // Import the necessary modules from the main application
    use minepack::commands;
    use minepack::models::config::ModpackConfig;
    use minepack::utils;
    use minepack::utils::MockEnv;

    /// Basic test to verify modpack configuration creation and validation using non-interactive CLI mode
    #[tokio::test]
    async fn test_minepack_init() -> Result<()> {
        // Set up isolated test environment
        let env = MockEnv::new();

        // Expected values for testing
        let expected_name = "Test Modpack";
        let expected_version = "1.0.0";
        let expected_author = "Test Author";
        let expected_description = "A test modpack";
        let expected_loader = "forge";
        let expected_minecraft_version = "1.21.1";

        println!("CONFIG_TEST - Running init command with non-interactive CLI");

        // Run the init command programmatically
        let result = commands::init::run(
            &env,
            Some(expected_name.to_string()),
            Some(expected_version.to_string()),
            Some(expected_author.to_string()),
            Some(expected_description.to_string()),
            Some(expected_loader.to_string()),
            Some(expected_minecraft_version.to_string()),
            Some("1.21.1-71.0.14".to_string()), // Adding loader version
        )
        .await;

        // Assert that the init command succeeded
        assert!(result.is_ok(), "Init command failed: {:?}", result);

        println!("CONFIG_TEST - Init command executed successfully");

        // Print the directory structure for debugging
        println!("CONFIG_TEST - Current directory: {:?}", env.current_dir()?);
        println!("CONFIG_TEST - Directory structure after initialization:");
        print_dir_structure(&env.tempdir.to_string_lossy(), 0)?;

        // Verify that files and directories were created
        assert!(
            env.current_dir()?.join("minepack.json").exists(),
            "minepack.json doesn't exist"
        );
        assert!(
            env.current_dir()?.join("mods").exists(),
            "mods directory doesn't exist"
        );
        assert!(
            env.current_dir()?.join("config").exists(),
            "config directory doesn't exist"
        );

        // Read and verify the content of the config file
        let read_content = fs::read_to_string(env.current_dir()?.join("minepack.json"))
            .context("Failed to read minepack.json")?;
        println!("CONFIG_TEST - Raw JSON content:\n{}", read_content);

        // Parse the JSON content
        let read_config: ModpackConfig =
            serde_json::from_str(&read_content).context("Failed to parse minepack.json")?;

        // Verify all fields match what we expected
        assert_eq!(read_config.name, expected_name, "Name doesn't match");
        assert_eq!(
            read_config.version, expected_version,
            "Version doesn't match"
        );
        assert_eq!(read_config.author, expected_author, "Author doesn't match");
        assert_eq!(
            read_config.description,
            Some(expected_description.to_string()),
            "Description doesn't match"
        );

        assert_eq!(
            read_config.minecraft.version, expected_minecraft_version,
            "Minecraft version doesn't match"
        );
        assert_eq!(
            read_config.minecraft.mod_loaders[0].id, expected_loader,
            "Mod loader doesn't match"
        );
        assert_eq!(
            read_config.minecraft.mod_loaders[0].version, "1.21.1-71.0.14",
            "Mod loader version doesn't match"
        );
        assert_eq!(
            read_config.minecraft.mod_loaders[0].primary, true,
            "Mod loader is not primary"
        );

        // Change back to the original directory before the temp dir is dropped
        env.close()?;

        // temp_dir will be automatically cleaned up when it goes out of scope
        Ok(())
    }

    /// Test to verify adding a mod by CurseForge URL to the modpack
    #[tokio::test]
    async fn test_add_mod_by_url() -> Result<()> {
        // Set up isolated test environment
        let env = MockEnv::new();

        // First, initialize a modpack (required before adding mods)
        println!("ADD_MOD_URL_TEST - Initializing test modpack");
        let init_result = commands::init::run(
            &env,
            Some("Test Modpack".to_string()),
            Some("1.0.0".to_string()),
            Some("Test Author".to_string()),
            Some("A test modpack".to_string()),
            Some("fabric".to_string()),
            Some("1.21.1".to_string()),
            Some("0.15.1".to_string()), // Adding loader version
        )
        .await;
        assert!(
            init_result.is_ok(),
            "Init command failed: {:?}",
            init_result
        );
        let init_config_path = env.current_dir()?.join("minepack.json");
        assert!(
            init_config_path.exists(),
            "minepack.json doesn't exist after initialization"
        );

        println!(
            "ADD_MOD_URL_TEST - Current directory: {:?}",
            env.current_dir()?
        );
        println!("ADD_MOD_URL_TEST - Directory structure before adding mod:");
        print_dir_structure(&env.tempdir.to_string_lossy(), 0)?;

        // The URL to add - using the oritech mod as an example
        let mod_url =
            "https://www.curseforge.com/minecraft/mc-mods/oritech/files/6332315".to_string();
        println!(
            "ADD_MOD_URL_TEST - Running add command with URL {}",
            mod_url
        );

        // Run the add command programmatically
        let add_result = commands::add::run(&env, Some(mod_url), true).await;

        // Assert that the add command succeeded
        assert!(add_result.is_ok(), "Add command failed: {:?}", add_result);

        println!("ADD_MOD_URL_TEST - Add command executed successfully");

        // Print the directory structure for debugging
        println!("ADD_MOD_URL_TEST - Directory structure after adding mod:");
        print_dir_structure(&env.tempdir.to_string_lossy(), 0)?;

        // Verify that the mod was added by checking if a .ex.json file exists in the mods directory
        let mods_dir = env.current_dir()?.join("mods");
        assert!(mods_dir.exists(), "mods directory doesn't exist");

        // Check if at least one .ex.json file exists in the mods directory
        let has_mod_files = fs::read_dir(mods_dir)?
            .filter_map(|entry| entry.ok())
            .any(|entry| {
                entry
                    .path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map_or(false, |ext| ext == "json")
            });

        assert!(has_mod_files, "No mod JSON files found in mods directory");

        // Change back to the original directory before the temp dir is dropped
        env.close()?;

        Ok(())
    }

    /// Test to verify building a modpack with a specified format without prompts
    #[tokio::test]
    async fn test_build_with_curseforge_format() -> Result<()> {
        // Set up isolated test environment
        let env = MockEnv::new();

        // First, initialize a modpack (required before building)
        println!("BUILD_FORMAT_TEST - Initializing test modpack");
        let init_result = commands::init::run(
            &env,
            Some("Test Modpack".to_string()),
            Some("1.0.0".to_string()),
            Some("Test Author".to_string()),
            Some("A test modpack".to_string()),
            Some("fabric".to_string()),
            Some("1.20.1".to_string()),
            Some("0.14.21".to_string()), // Adding loader version
        )
        .await;
        assert!(
            init_result.is_ok(),
            "Init command failed: {:?}",
            init_result
        );

        // Add a simple mock mod file to simulate a real modpack
        let mods_dir = env.current_dir()?.join("mods");
        assert!(mods_dir.exists(), "mods directory doesn't exist");

        // Create a mock mod JSON file
        let mock_mod_json = r#"{
            "name": "Test Mod",
            "filename": "test-mod-1.0.0.jar",
            "link": {
                "project_id": 123456,
                "file_id": 7890123
            }
        }"#;

        fs::write(mods_dir.join("test-mod.ex.json"), mock_mod_json)
            .context("Failed to write mock mod JSON file")?;

        // Create the cache directory and fake mod file
        let cache_dir = utils::get_minepack_cache_mods_dir(&env)?;
        fs::create_dir_all(&cache_dir).context("Failed to create cache directory")?;

        // Create an empty JAR file in the cache directory
        let jar_path = cache_dir.join("test-mod-1.0.0.jar");
        fs::write(&jar_path, "mock jar file").context("Failed to create mock JAR file")?;

        // Create a mock config file
        let config_dir = env.current_dir()?.join("config");
        assert!(config_dir.exists(), "config directory doesn't exist");

        // Create a mock config file with some content
        let config_content = r#"
# Test config file for Minecraft
# This demonstrates that config files are properly packaged

# Game settings
render_distance=12
difficulty=normal
enable_structures=true

# Graphics settings
graphics_mode=fancy
use_vsync=true
max_fps=120

# Advanced settings
allocated_memory=4G
java_arguments=-XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200
"#;

        // Create a typical Minecraft config file structure
        let minecraft_config_path = config_dir.join("minecraft").join("options.txt");
        fs::create_dir_all(minecraft_config_path.parent().unwrap())
            .context("Failed to create minecraft config directory")?;
        fs::write(&minecraft_config_path, config_content)
            .context("Failed to write mock config file")?;

        println!(
            "BUILD_FORMAT_TEST - Created mock config file at {:?}",
            minecraft_config_path
        );

        // Create build directory
        let build_dir = env.current_dir()?.join("build");
        fs::create_dir_all(&build_dir).context("Failed to create build directory")?;

        println!("BUILD_FORMAT_TEST - Directory structure before building:");
        print_dir_structure(&env.tempdir.to_string_lossy(), 0)?;

        // Run the build command with curseforge format option
        println!("BUILD_FORMAT_TEST - Running build command with format option");
        println!(
            "BUILD_FORMAT_TEST - Current directory: {:?}",
            env.current_dir()?
        );
        println!("BUILD_FORMAT_TEST - Build directory: {:?}", build_dir);

        let build_result = commands::build::run(&env, Some("curseforge".to_string())).await;

        // Assert that the build command succeeded
        assert!(
            build_result.is_ok(),
            "Build command failed: {:?}",
            build_result
        );

        println!("BUILD_FORMAT_TEST - Build command executed successfully");
        println!("BUILD_FORMAT_TEST - Directory structure after building:");
        print_dir_structure(&env.tempdir.to_string_lossy(), 0)?;

        // Explicitly check all files in the build directory
        println!("BUILD_FORMAT_TEST - Files in build directory:");
        if build_dir.exists() {
            for entry in fs::read_dir(&build_dir)? {
                let entry = entry?;
                println!("  - {}", entry.path().display());
            }
        } else {
            println!("  - Build directory does not exist!");
        }

        // Verify that the output file was created with more flexible check
        // Just check if any ZIP file exists in the build directory
        let zipfile_dir = env
            .current_dir()?
            .join("build")
            .join("Test Modpack-1.0.0-CurseForge.zip");
        assert!(
            zipfile_dir.exists(),
            "Output ZIP file doesn't exist in the build directory"
        );

        // Unzip the file to verify its contents
        let zipfile = fs::File::open(&zipfile_dir).context("Failed to open the output ZIP file")?;
        let mut archive =
            zip::ZipArchive::new(zipfile).context("Failed to read the ZIP archive")?;
        let output_dir = env.current_dir()?.join("unzipped");
        fs::create_dir_all(&output_dir).context("Failed to create output directory")?;
        archive
            .extract(&output_dir)
            .context("Failed to extract the ZIP archive")?;

        println!("BUILD_FORMAT_TEST - Extracted files in output directory:");
        for file in fs::read_dir(&output_dir)? {
            let file = file?;
            println!("  - {}", file.path().display());
        }

        let manifest_path = output_dir.join("manifest.json");
        assert!(
            manifest_path.exists(),
            "Manifest file doesn't exist in the extracted directory"
        );
        let manifest_content =
            fs::read_to_string(&manifest_path).context("Failed to read the manifest file")?;
        let manifest: minepack::api::curseforge::schema::Manifest =
            serde_json::from_str(&manifest_content).context("Failed to parse manifest file")?;
        assert_eq!(manifest.name, "Test Modpack", "Manifest name doesn't match");
        assert_eq!(manifest.version, "1.0.0", "Manifest version doesn't match");
        assert_eq!(
            manifest.author, "Test Author",
            "Manifest author doesn't match"
        );
        assert_eq!(
            manifest.files.len(),
            1,
            "Manifest files count doesn't match"
        );
        assert_eq!(
            manifest.overrides, "overrides",
            "Manifest overrides directory doesn't match"
        );
        assert_eq!(
            manifest.minecraft.version, "1.20.1",
            "Manifest Minecraft version doesn't match"
        );
        assert_eq!(
            manifest.minecraft.mod_loaders[0].id, "fabric-0.14.21",
            "Manifest mod loader ID doesn't match"
        );
        assert_eq!(
            manifest.minecraft.mod_loaders[0].primary, true,
            "Manifest mod loader is not primary"
        );

        // Verify config file was included in the built modpack
        let extracted_config_path = output_dir
            .join("overrides")
            .join("config")
            .join("minecraft")
            .join("options.txt");
        println!(
            "BUILD_FORMAT_TEST - Looking for config file at: {:?}",
            extracted_config_path
        );
        assert!(
            extracted_config_path.exists(),
            "Config file wasn't included in the built modpack"
        );

        // Verify the config file content is correct
        let extracted_config_content = fs::read_to_string(&extracted_config_path)
            .context("Failed to read extracted config file")?;
        assert_eq!(
            extracted_config_content, config_content,
            "Extracted config file content doesn't match the original"
        );

        // Print the directory structure of the extracted files for debugging
        println!("BUILD_FORMAT_TEST - Detailed structure of extracted files:");
        print_dir_structure(&output_dir.to_string_lossy(), 0)?;

        // Clean up
        env.close()?;
        Ok(())
    }

    fn print_dir_structure(path: &str, depth: usize) -> Result<()> {
        let indent = "  ".repeat(depth);
        let entries = fs::read_dir(path)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            let file_name = path.file_name().unwrap().to_string_lossy();

            if path.is_dir() {
                println!("{}{}/", indent, file_name);
                print_dir_structure(&path.to_string_lossy(), depth + 1)?;
            } else {
                println!("{}{}", indent, file_name);
            }
        }

        Ok(())
    }
}
