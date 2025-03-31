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
minepack init [--name NAME] [--version VERSION] [--author AUTHOR] [--description DESCRIPTION] [--loader LOADER] [--minecraft-version VERSION] [--loader-version VERSION]
```

This will guide you through creating a new modpack. You can provide the following options directly:

- `--name`: Name of the modpack
- `--version`: Version of the modpack
- `--author`: Author of the modpack
- `--description`: Description of the modpack
- `--loader`: Mod loader to use (forge, fabric, quilt, neoforge)
- `--minecraft-version`: Minecraft version
- `--loader-version`: Mod loader version

If options are not provided, you will be prompted to enter them interactively.

#### Search for mods

```bash
minepack search <QUERY>
```

Example:

```bash
minepack search jei
```

This will display information about mods matching your query, including ID, name, download count, and summary.

#### Add a mod to your modpack

```bash
minepack add [MOD] [--yes, -y]
```

Example:

```bash
minepack add https://www.curseforge.com/minecraft/mc-mods/oritech/files/6332315
minepack add https://www.curseforge.com/minecraft/mc-mods/oritech

# with automatic confirmation
minepack add https://www.curseforge.com/minecraft/mc-mods/oritech/files/6332315 --yes
```

The `--yes` flag will skip confirmation prompts.

#### Build the modpack

```bash
minepack build [--format FORMAT]
```

This will build your modpack into the specified format:

- `--format multimc`: For direct import into MultiMC launcher (.zip)
- `--format curseforge`: For upload to Curseforge or use with CurseForge/Overwolf launchers (.zip)
- `--format modrinth`: For use with Modrinth compatible launchers (.mrpack)

If no format is specified, you will be prompted to choose one.

## Directory Structure

A typical minepack project will have the following structure:

```
my-modpack/
├── minepack.json  # Modpack configuration file
├── mods/          # Where mod files are stored
│   └── *.ex.json  # Information about the mod to be installed
└── config/        # Optional configuration files for mods
```

## Development

### Quality Assurance

Before submitting changes, run the following commands to ensure quality:

```bash
mise tasks run format-write  # Format code
mise tasks run lint          # Run linters
mise tasks run test          # Run tests
mise tasks run build         # Build project
```

## License

MIT
