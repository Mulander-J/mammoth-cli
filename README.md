# 🦣 Mammoth CLI ❄️

A powerful frontend project scaffolding CLI tool that helps you quickly create new projects from templates.

## Features

- 🚀 **Template Management**: Add, remove, and manage templates from Git repositories
- 📦 **Repository Support**: Support multiple remote repositories with sparse checkout
- 🎨 **Interactive Creation**: Interactive project creation with customizable options
- 🔧 **Auto Configuration**: Automatically update package.json and initialize Git repository
- 💾 **Smart Caching**: Efficient template caching to avoid repeated downloads
- 📤 **Configuration Export/Import**: Export and import configuration for backup and sharing

## Installation

```bash
cargo install --path .
```

## Usage

Commands Struct

```text
mammoth-cli
├── new                    # 创建项目
├── clean                  # 清理配置和缓存
├── info                   # 显示配置信息
├── template               # 模板管理
│   ├── list              # 列出模板
│   ├── add               # 添加模板
│   ├── remove            # 删除模板
│   ├── download          # 下载模板
│   └── download-all      # 下载所有模板
├── repo                   # 仓库管理
│   ├── list              # 列出仓库
│   ├── add               # 添加仓库
│   └── remove            # 删除仓库
└── config                 # 配置管理
    ├── export            # 导出配置
    ├── import            # 导入配置
    └── validate          # 验证配置
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

> Get [example.config.json](./example.config.json)

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
│   └── main.rs          # Main CLI application
├── Cargo.toml           # Project dependencies
└── README.md           # This file
```

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

## License

[MIT License](./LICENSE)

## 贡献

欢迎提交 Issue 和 Pull Request！

## 更新日志

### v0.1.0

- 初始版本
- 远程模板支持
- 模板缓存机制
- 完整的模板管理功能
- 交互式项目创建向导
- 配置导入/导出功能
