use anyhow::{anyhow, Context, Result};
use console::style;

use crate::api::curseforge::CurseforgeClient;
use crate::utils::errors::MinepackError;
use crate::{api, utils};

pub async fn run<E: utils::Env>(env: &E, query: &str) -> Result<()> {
    // Check if we're in a modpack directory
    if !utils::modpack_exists(env) {
        return Err(anyhow!(MinepackError::NoModpackFound));
    }

    let config = utils::load_config(env)?;
    let client = CurseforgeClient::new().context("Failed to initialize Curseforge API client")?;

    println!("🔍 Searching for mods matching '{}'...", query);

    // Search for mods with the given query, filtered by the configured Minecraft version
    let mods = client
        .search_mods(&api::curseforge::schema::SearchModsRequestQuery {
            search_filter: Some(query.to_string()),
            game_version: Some(config.minecraft.version.clone()),
            ..Default::default()
        })
        .await
        .context("Failed to search for mods")?;

    if mods.is_empty() {
        return Err(anyhow!(MinepackError::NoModsFound(query.to_string())));
    }

    // Display mod information in a formatted table
    println!("\n{} results found:", mods.len());
    println!(
        "{:<8} {:<40} {:<15} {}",
        style("ID").bold(),
        style("Name").bold(),
        style("Downloads").bold(),
        style("Summary").bold()
    );

    // Show top 10 results
    for mod_info in mods.iter().take(10) {
        println!(
            "{:<8} {:<40} {:<15} {}",
            mod_info.id,
            if mod_info.name.len() > 38 {
                format!("{}...", &mod_info.name[0..35])
            } else {
                mod_info.name.clone()
            },
            format!("{}K", mod_info.download_count / 1000),
            if mod_info.summary.len() > 50 {
                format!("{}...", &mod_info.summary[0..47])
            } else {
                mod_info.summary.clone()
            }
        );
    }

    println!("\nTo add a mod, use 'minepack add <mod_id>' or 'minepack add <search_term>'");

    Ok(())
}
