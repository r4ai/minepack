mod api;
mod commands;
mod models;
mod utils;

use clap::{Parser, Subcommand};
use console::style;

#[derive(Parser)]
#[command(name = "minepack")]
#[command(author, version, about = "A CLI tool for creating Minecraft Modpacks", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new modpack project
    Init,
    /// Add a mod to the modpack
    Add {
        /// Mod ID or search query
        #[arg(value_name = "MOD")]
        mod_query: Option<String>,
    },
    /// Search for mods on Curseforge
    Search {
        /// Search query
        #[arg(value_name = "QUERY")]
        query: String,
    },
    /// Build the modpack
    Build,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init => commands::init::run().await,
        Commands::Add { mod_query } => commands::add::run(mod_query).await,
        Commands::Search { query } => commands::search::run(&query).await,
        Commands::Build => commands::build::run().await,
    };

    if let Err(err) = result {
        eprintln!("{} {}", style("Error:").bold().red(), err);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }
}
