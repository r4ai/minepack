#[cfg(feature = "mock")]
mod tests {
    use anyhow::{Context, Result};
    use minepack::utils::Env;
    use std::fs;

    // Import the necessary modules from the main application
    use minepack::commands;
    use minepack::models::config::ModpackConfig;

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
            read_config.mod_loader.to_string(),
            expected_loader,
            "Mod loader doesn't match"
        );
        assert_eq!(
            read_config.minecraft_version, expected_minecraft_version,
            "Minecraft version doesn't match"
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
        )
        .await;
        assert!(
            init_result.is_ok(),
            "Init command failed: {:?}",
            init_result
        );
        let init_config_path = env.current_dir()?.join("minepack.json");
        let init_config: ModpackConfig =
            serde_json::from_str(&fs::read_to_string(&init_config_path).with_context(|| {
                format!(
                    "Failed to read minepack.json: {}",
                    &init_config_path.display()
                )
            })?)
            .context("Failed to parse minepack.json")?;
        dbg!(&init_config);
        assert_eq!(init_config.name, "Test Modpack", "Name doesn't match");
        assert_eq!(init_config.version, "1.0.0", "Version doesn't match");
        assert_eq!(init_config.author, "Test Author", "Author doesn't match");
        assert_eq!(
            init_config.description,
            Some("A test modpack".to_string()),
            "Description doesn't match"
        );
        assert_eq!(
            init_config.mod_loader.to_string(),
            "fabric",
            "Mod loader doesn't match"
        );
        assert_eq!(
            init_config.minecraft_version, "1.21.1",
            "Minecraft version doesn't match"
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
