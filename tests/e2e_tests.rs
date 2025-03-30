use anyhow::{Context, Result};
use assert_fs::TempDir;
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::{Mutex, Once};

// Import the necessary modules from the main application
use minepack::commands;
use minepack::models::config::{ModEntry, ModpackConfig};

// Used to ensure we initialize logging only once
static INIT: Once = Once::new();

// This mutex will ensure our tests run sequentially even if the test harness tries to run them in parallel
static TEST_MUTEX: Mutex<()> = Mutex::new(());

// Helper function to set up an isolated test environment
fn setup_test_environment(test_name: &str) -> Result<(TempDir, std::path::PathBuf)> {
    // Initialize logging only once
    INIT.call_once(|| {
        // We could set up logging here if needed
    });

    // Create a uniquely named temporary directory for this test
    let temp = TempDir::new()?;
    println!(
        "{} - Temporary directory created at: {}",
        test_name,
        temp.path().display()
    );

    // Save the original directory
    let original_dir = env::current_dir()?;

    // Change to the temporary directory
    env::set_current_dir(temp.path())?;
    println!(
        "{} - Current directory changed to: {}",
        test_name,
        env::current_dir()?.display()
    );

    Ok((temp, original_dir))
}

/// Basic test to verify modpack configuration creation and validation using non-interactive CLI mode
#[tokio::test]
async fn test_modpack_config_creation() -> Result<()> {
    // Acquire mutex lock to ensure test isolation
    let _lock = TEST_MUTEX.lock().unwrap();

    // Set up isolated test environment
    let (_temp_dir, original_dir) = setup_test_environment("CONFIG_TEST")?;

    // Expected values for testing
    let expected_name = "Test Modpack";
    let expected_version = "1.0.0";
    let expected_author = "Test Author";
    let expected_description = "A test modpack";
    let expected_loader = "forge";
    let expected_minecraft_version = "1.19.2";

    println!("CONFIG_TEST - Running init command with non-interactive CLI");

    // Run the init command programmatically
    let result = commands::init::run(
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
    println!("CONFIG_TEST - Directory structure after initialization:");
    print_dir_structure(".", 0)?;

    // Verify that files and directories were created
    assert!(
        Path::new("minepack.toml").exists(),
        "minepack.toml doesn't exist"
    );
    assert!(Path::new("mods").exists(), "mods directory doesn't exist");
    assert!(
        Path::new("config").exists(),
        "config directory doesn't exist"
    );

    // Read and verify the content of the config file
    let read_content =
        fs::read_to_string("minepack.toml").context("Failed to read minepack.toml")?;
    println!("CONFIG_TEST - Raw TOML content:\n{}", read_content);

    // Parse the TOML content
    let read_config: ModpackConfig =
        toml::from_str(&read_content).context("Failed to parse minepack.toml")?;

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
    assert_eq!(read_config.mods.len(), 0, "Mods array should be empty");

    // Change back to the original directory before the temp dir is dropped
    env::set_current_dir(original_dir)?;

    // temp_dir will be automatically cleaned up when it goes out of scope
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

/// Test for the full workflow: init, add, build
#[tokio::test]
async fn test_full_simulated_workflow() -> Result<()> {
    // Acquire mutex lock to ensure test isolation
    let _lock = TEST_MUTEX.lock().unwrap();

    // Set up isolated test environment
    let (_temp_dir, original_dir) = setup_test_environment("WORKFLOW_TEST")?;

    // Expected values for testing - using different values from the first test
    let expected_name = "Full Workflow Test Modpack";
    let expected_version = "2.0.0";
    let expected_author = "Workflow Author";
    let expected_description = "A modpack for testing the full workflow";
    let expected_loader = "fabric";
    let expected_minecraft_version = "1.20.1";

    // Step 1: Initialize the modpack using non-interactive CLI
    println!("WORKFLOW_TEST - Step 1: Initializing modpack with non-interactive CLI");
    let init_result = commands::init::run(
        Some(expected_name.to_string()),
        Some(expected_version.to_string()),
        Some(expected_author.to_string()),
        Some(expected_description.to_string()),
        Some(expected_loader.to_string()),
        Some(expected_minecraft_version.to_string()),
    )
    .await;

    assert!(
        init_result.is_ok(),
        "Init command failed: {:?}",
        init_result
    );
    println!("WORKFLOW_TEST - Modpack initialized successfully");

    // Verify the directory structure after initialization
    assert!(
        Path::new("minepack.toml").exists(),
        "minepack.toml doesn't exist"
    );
    assert!(Path::new("mods").exists(), "mods directory doesn't exist");
    assert!(
        Path::new("config").exists(),
        "config directory doesn't exist"
    );

    // Read and print the raw TOML content for debugging
    let initial_content =
        fs::read_to_string("minepack.toml").context("Failed to read minepack.toml")?;
    println!(
        "WORKFLOW_TEST - Raw TOML content after initialization:\n{}",
        initial_content
    );

    // Parse the TOML content
    let initial_config: ModpackConfig =
        toml::from_str(&initial_content).context("Failed to parse minepack.toml")?;

    // Print the parsed config values for debugging
    println!("WORKFLOW_TEST - Parsed config values:");
    println!("name: {}", initial_config.name);
    println!("version: {}", initial_config.version);
    println!("author: {}", initial_config.author);
    println!("description: {:?}", initial_config.description);
    println!("mod_loader: {}", initial_config.mod_loader);
    println!("minecraft_version: {}", initial_config.minecraft_version);

    // Verify initial config values
    assert_eq!(initial_config.name, expected_name, "Name doesn't match");
    assert_eq!(
        initial_config.version, expected_version,
        "Version doesn't match"
    );
    assert_eq!(
        initial_config.author, expected_author,
        "Author doesn't match"
    );
    assert_eq!(
        initial_config.description,
        Some(expected_description.to_string()),
        "Description doesn't match"
    );
    assert_eq!(
        initial_config.mod_loader.to_string(),
        expected_loader,
        "Mod loader doesn't match"
    );
    assert_eq!(
        initial_config.minecraft_version, expected_minecraft_version,
        "Minecraft version doesn't match"
    );
    assert_eq!(initial_config.mods.len(), 0, "Mods array should be empty");

    // Step 2: Simulate adding a mod to the modpack
    println!("WORKFLOW_TEST - Step 2: Simulating adding a mod to the modpack");

    // Ensure mods directory exists
    fs::create_dir_all("mods").context("Failed to create mods directory")?;

    // Create a test mod file
    let mod_name = "test-mod";
    let mod_version = "1.0.0";
    let mod_filename = format!("{}-{}.jar", mod_name, mod_version);
    let mod_path = format!("mods/{}", mod_filename);

    // Create an empty JAR file as a placeholder
    let mut file = fs::File::create(&mod_path).context("Failed to create test mod file")?;
    file.write_all(b"test mod content")
        .context("Failed to write test mod content")?;
    println!("WORKFLOW_TEST - Created test mod file at {}", mod_path);

    // Update the config file to include the mod
    let updated_config = ModpackConfig {
        name: initial_config.name.clone(),
        version: initial_config.version.clone(),
        author: initial_config.author.clone(),
        description: initial_config.description.clone(),
        mod_loader: initial_config.mod_loader,
        minecraft_version: initial_config.minecraft_version.clone(),
        mods: vec![
            // Create a new mods vector with our test mod
            ModEntry {
                name: "Test Mod".to_string(),
                project_id: 12345,
                file_id: 67890,
                version: "1.0.0".to_string(),
                download_url: "https://example.com/mod.jar".to_string(),
                required: true,
            },
        ],
    };

    // Save the updated config
    let updated_content =
        toml::to_string(&updated_config).context("Failed to serialize updated config")?;
    fs::write("minepack.toml", updated_content).context("Failed to write updated config")?;
    println!("WORKFLOW_TEST - Updated config file with test mod");

    // Verify the mod was added to the config file
    let read_updated_content =
        fs::read_to_string("minepack.toml").context("Failed to read updated minepack.toml")?;
    let read_updated_config: ModpackConfig =
        toml::from_str(&read_updated_content).context("Failed to parse updated config")?;

    assert_eq!(
        read_updated_config.mods.len(),
        1,
        "Expected 1 mod in the config"
    );
    assert_eq!(
        read_updated_config.mods[0].name, "Test Mod",
        "Mod name doesn't match"
    );
    assert_eq!(
        read_updated_config.mods[0].version, "1.0.0",
        "Mod version doesn't match"
    );

    // Step 3: Verify the mod file exists in the mods directory
    assert!(
        Path::new(&mod_path).exists(),
        "Mod file doesn't exist at {}",
        mod_path
    );

    // Print the final directory structure
    println!("WORKFLOW_TEST - Directory structure after completion:");
    print_dir_structure(".", 0)?;

    // Change back to the original directory before the temp dir is dropped
    env::set_current_dir(original_dir)?;

    // temp_dir will be automatically cleaned up when it goes out of scope
    Ok(())
}
