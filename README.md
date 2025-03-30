# Minepack

A CLI tool for creating and managing Minecraft Modpacks, similar to packwiz but with seamless integration with the Curseforge API.

## Features

- Easy modpack initialization with guided prompts
- Search for mods on Curseforge
- Add mods to your modpack from Curseforge
- Build modpacks in different formats:
  - MultiMC
  - CurseForge
  - Modrinth

## Installation

### From Source

```bash
git clone https://github.com/r4ai/minepack.git
cd minepack
cargo install --path .
```

## Usage

### Setting up the API Key

Minepack requires a Curseforge API key to function. You can obtain one by:

1. Creating an account on [Curseforge](https://www.curseforge.com/)
2. Accessing the [API portal](https://console.curseforge.com/)
3. Creating a new API key

Once you have your API key, set it as an environment variable:

```bash
export CURSEFORGE_API_KEY="your_key_here"
```

For persistent usage, add this to your shell profile file.

### Commands

#### Initialize a new modpack

```bash
minepack init
```

This will guide you through creating a new modpack with:
- Name
- Version
- Author
- Description
- Mod Loader (Forge/Fabric/Quilt)
- Minecraft version

#### Search for mods

```bash
minepack search <query>
```

Example:
```bash
minepack search jei
```

This will display information about mods matching your query, including ID, name, download count, and summary.

#### Add a mod to your modpack

```bash
minepack add <mod_id_or_name>
```

Example:
```bash
minepack add jei
# or
minepack add 238222  # JEI's project ID
```

If you run `minepack add` without arguments, it will prompt you to enter a search term or mod ID.

#### Build the modpack

```bash
minepack build
```

This will guide you through the process of building your modpack into one of the following formats:

- MultiMC (.zip) - For direct import into MultiMC launcher
- CurseForge (.zip) - For upload to Curseforge or use with CurseForge/Overwolf launchers
- Modrinth (.mrpack) - For use with Modrinth compatible launchers

## Directory Structure

A typical minepack project will have the following structure:

```
my-modpack/
├── minepack.toml  # Modpack configuration file
├── mods/          # Where mod files are stored
│   └── *.jar      # Downloaded mod files
└── config/        # Optional configuration files for mods
```

## License

MIT