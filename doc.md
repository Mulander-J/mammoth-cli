# Develop Doc

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