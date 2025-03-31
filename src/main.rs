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
    Init {
        /// Name of the modpack
        #[arg(long)]
        name: Option<String>,

        /// Version of the modpack
        #[arg(long)]
        version: Option<String>,

        /// Author of the modpack
        #[arg(long)]
        author: Option<String>,

        /// Description of the modpack
        #[arg(long)]
        description: Option<String>,

        /// Mod loader to use (forge, fabric, quilt, neoforge)
        #[arg(long)]
        loader: Option<String>,

        /// Minecraft version
        #[arg(long)]
        minecraft_version: Option<String>,
    },
    /// Add a mod to the modpack
    Add {
        /// Mod ID or search query
        #[arg(value_name = "MOD")]
        mod_query: Option<String>,

        #[arg(long, short, default_value_t = false)]
        yes: bool,
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

    let env = utils::RealEnv;

    let result = match cli.command {
        Commands::Init {
            name,
            version,
            author,
            description,
            loader,
            minecraft_version,
        } => {
            commands::init::run(
                &env,
                name,
                version,
                author,
                description,
                loader,
                minecraft_version,
            )
            .await
        }
        Commands::Add { mod_query, yes } => commands::add::run(&env, mod_query, yes).await,
        Commands::Search { query } => commands::search::run(&env, &query).await,
        Commands::Build => commands::build::run(&env).await,
    };

    if let Err(err) = result {
        eprintln!("{}", style("[ERROR]").bold().red());
        eprintln!("{:?}", style(err).red());
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
