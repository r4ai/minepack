use anyhow::{Context, Result};
use assert_fs::TempDir;
use std::env;
use std::fs;
use std::path::Path;
use std::sync::{Mutex, Once};

// Import the necessary modules from the main application
use minepack::commands;
use minepack::models::config::ModpackConfig;

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
        Path::new("minepack.json").exists(),
        "minepack.json doesn't exist"
    );
    assert!(Path::new("mods").exists(), "mods directory doesn't exist");
    assert!(
        Path::new("config").exists(),
        "config directory doesn't exist"
    );

    // Read and verify the content of the config file
    let read_content =
        fs::read_to_string("minepack.json").context("Failed to read minepack.json")?;
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
