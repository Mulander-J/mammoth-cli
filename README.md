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
cargo install --path .
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
      "name": "aio-templates",
      "url": "https://github.com/Mulander-J/aio-templates",
      "branch": "main"
    }
  ],
  "templates": [
    {
      "id": "nuxt-shadcn",
      "name": "Nuxt Shadcn Starter",
      "repo": "aio-templates",
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

## Project Structure

```text
mammoth-cli/
├── src/
│   ├── main.rs      # Entry, command dispatch
│   ├── cli.rs       # CLI argument and subcommand definitions
│   ├── config.rs    # Config structure and management
│   ├── manager.rs   # Template & repository management core logic
│   ├── project.rs   # Project creation and initialization
│   ├── utils.rs     # Utility functions
│   └── lib.rs       # Common library (if any)
├── Cargo.toml
└── README.md
```

## Dependencies

| 依赖 | 版本 | 功能简述 |
|------|------|----------|
| `clap` | 4.0 | CLI参数解析和命令行界面框架，支持derive宏 |
| `anyhow` | 1.0 | 错误处理库，提供简洁的错误类型和传播 |
| `serde` | 1.0 | 序列化和反序列化框架，支持derive宏 |
| `serde_json` | 1.0 | JSON序列化和反序列化实现 |
| `tokio` | 1.0 | 异步运行时，提供完整的异步功能支持 |
| `colored` | 2.0 | 终端文本颜色和样式控制 |
| `dialoguer` | 0.11 | 交互式命令行提示和输入库 |
| `indicatif` | 0.17 | 进度条和加载动画显示 |
| `dirs` | 5.0 | 跨平台系统目录路径获取 |

## Development

```bash
# Build the project
cargo build

# Run in development mode
cargo run

# Run with verbose output
cargo run -- --verbose

# Check for compilation errors
cargo check

# Run tests
cargo test
```

## FAQ

### Q: On Windows, sometimes cache/temp directories cannot be deleted?

A: This may be caused by other processes (such as Explorer or antivirus software) occupying related files. Please close those programs and try again, or reboot and run the clean command.

## License

[MIT License](./LICENSE)

## Contribution

Welcome to submit Issue and Pull Request!

## Update Log

### v0.1.0

- Initial version
- Remote template support
- Template cache mechanism
- Complete template management functionality
- Interactive project creation wizard
- Configuration import/export functionality
