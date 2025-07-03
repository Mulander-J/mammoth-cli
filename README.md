# 🦣 Mammoth CLI ❄️

A powerful frontend project scaffolding CLI tool that helps you quickly create new projects from templates.

## Features

- 🚀 **Template Management**: Add, remove, and manage templates from Git repositories
- 📦 **Repository Support**: Support multiple remote repositories with sparse checkout
- 🎨 **Interactive Creation**: Interactive project creation with customizable options
- 🔧 **Auto Configuration**: Automatically update package.json and initialize Git repository
- 💾 **Smart Caching**: Efficient template caching to avoid repeated downloads
- 📤 **Configuration Export/Import**: Export and import configuration for backup and sharing
- 🧩 **Modular Design**: Clean code structure, easy to maintain and extend
- 🛡️ **Robust Cache & Cleanup**: Multiple retries and process timeouts, cross-platform compatibility
- ⚡ **Minimal Dependencies**: No redundant dependencies, fast startup, small binary size

## Installation

```bash
cargo install mammoth-cli
```

## Usage

Command structure:

```text
mammoth-cli
├── new                    # Create project (top-level command)
├── clean                  # Clean config and cache (top-level command)
├── info                   # Show config info (top-level command)
├── template               # Template management (subcommand)
│   ├── list              # List templates
│   ├── add               # Add template
│   ├── remove            # Remove template
│   ├── download          # Download template
│   └── download-all      # Download all templates
├── repo                   # Repository management (subcommand)
│   ├── list              # List repositories
│   ├── add               # Add repository
│   └── remove            # Remove repository
└── config                 # Config management (subcommand)
    ├── export            # Export config
    ├── import            # Import config
    └── validate          # Validate config
```

### Basic Commands

```bash
# Create a new project (interactive)
mammoth-cli

# Create a new project with specific template
mammoth-cli new --template nuxt-shadcn --name my-project

# Show configuration information
mammoth-cli info

# Show configuration as JSON
mammoth-cli info --json

# Clean cache and configuration
mammoth-cli clean

# Clean everything including config file
mammoth-cli clean --all

# Clean without confirmation
mammoth-cli clean --force
```

### Template Management

```bash
# List all templates
mammoth-cli template list

# List all templates with detailed information
mammoth-cli template list --verbose

# Add a template
mammoth-cli template add nuxt-shadcn \
  --name "Nuxt Shadcn Starter" \
  --repo aio-templates \
  --path "vue/nuxt-shadcn" \
  --description "Nuxt Shadcn with Tailwind" \
  --language vue \
  --tags "nuxt,shadcn,tailwind"

# Download a specific template
mammoth-cli template download nuxt-shadcn

# Download all templates
mammoth-cli template download-all

# Remove a template
mammoth-cli template remove nuxt-shadcn
```

### Repository Management

```bash
# Add a repository
mammoth-cli repo add aio-templates --url https://github.com/Mulander-J/aio-templates --branch main

# List repositories
mammoth-cli repo list

# Remove a repository
mammoth-cli repo remove aio-templates
```

### Configuration Management

```bash
# Export configuration to file
mammoth-cli config export --output config-backup.json

# Export configuration with cache information
mammoth-cli config export --output config-backup.json --include-cache

# Import configuration (merge mode - default)
mammoth-cli config import --file config-backup.json

# Import configuration (overwrite mode)
mammoth-cli config import --file config-backup.json --mode overwrite

# Import configuration without validation
mammoth-cli config import --file config-backup.json --skip-validation

# Validate configuration file
mammoth-cli config validate config-backup.json
```

## Configuration

The CLI stores configuration in:

- **Config**: `~/.config/mammoth-cli/templates.json` (Linux/macOS) or `%APPDATA%\mammoth-cli\templates.json` (Windows)
- **Cache**: `~/.cache/mammoth-cli/templates/` (Linux/macOS) or `%LOCALAPPDATA%\mammoth-cli\templates\` (Windows)

### Configuration Format

> See [example.config.json](./example.config.json)

The configuration file uses JSON format:

```json
{
    "repos": [
        {
            "name": "public-templates",
            "url": "https://github.com/your-org/public-templates",
            "branch": "main"
        },
        {
            "name": "private-templates",
            "url": "https://github.com/your-org/private-templates",
            "branch": "main",
            "username": "your-username",
            "auth_token": "your-personal-access-token"
        }
    ],
    "templates": [
        {
            "id": "nuxt-shadcn",
            "name": "Nuxt Shadcn Starter",
            "repo": "public-templates",
            "path": "vue/nuxt-shadcn",
            "description": "Nuxt Shadcn with Tailwind",
            "language": "vue",
            "tags": [
                "nuxt",
                "shadcn",
                "tailwind"
            ]
        }
    ]
}
```

## 🔗[Develop Doc](https://github.com/Mulander-J/mammoth-cli/blob/main/doc.md) ←

## ❓FAQ

### Q: On Windows, sometimes cache/temp directories cannot be deleted?

A: This may be caused by other processes (such as Explorer or antivirus software) occupying related files. Please close those programs and try again, or reboot and run the clean command.

## License

MIT License

## Contribution

Welcome to submit Issue and Pull Request!
