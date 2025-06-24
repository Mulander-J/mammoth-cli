# Mammoth CLI

A powerful frontend project scaffolding CLI tool that helps you quickly create new projects from templates.

## Features

- 🚀 **Template Management**: Add, remove, and manage templates from Git repositories
- 📦 **Repository Support**: Support multiple remote repositories with sparse checkout
- 🎨 **Interactive Creation**: Interactive project creation with customizable options
- 🔧 **Auto Configuration**: Automatically update package.json and initialize Git repository
- 💾 **Smart Caching**: Efficient template caching to avoid repeated downloads

## Installation

```bash
cargo install --path .
```

## Usage

### Basic Commands

```bash
# Create a new project (interactive)
mammoth-cli

# Create a new project with specific template
mammoth-cli new --template admin@vue@arco --name my-project

# List all templates
mammoth-cli template list

# List all templates with detailed information
mammoth-cli template list --verbose
```

### Template Management

```bash
# Add a repository
mammoth-cli template repo-add aio-templates --url https://github.com/Mulander-J/aio-templates --branch main

# List repositories
mammoth-cli template repo-list

# Add a template
mammoth-cli template add admin@vue@arco \
  --name "Arco Vue Admin" \
  --repo aio-templates \
  --path "vue/arco-vue-admin" \
  --description "Arco Design Vue3 管理后台模板" \
  --language vue \
  --tags "admin,vue,arco,typescript"

# Download a specific template
mammoth-cli template download admin@vue@arco

# Download all templates
mammoth-cli template download-all

# Remove a template
mammoth-cli template remove admin@vue@arco

# Remove a repository
mammoth-cli template repo-remove aio-templates
```

## Configuration

The CLI stores configuration in:
- **Config**: `~/.config/mammoth-cli/templates.json` (Linux/macOS) or `%APPDATA%\mammoth-cli\templates.json` (Windows)
- **Cache**: `~/.cache/mammoth-cli/templates/` (Linux/macOS) or `%LOCALAPPDATA%\mammoth-cli\templates\` (Windows)

## Project Structure

```
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
