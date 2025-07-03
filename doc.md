# 🦣 Mammoth CLI ❄️

## 🎯 项目概述

Mammoth CLI 是一个强大的前端项目脚手架工具，用 Rust 语言开发。它帮助开发者快速从模板创建新的前端项目，支持多种模板管理和远程仓库集成。

- 🚀 **高性能**: 基于 Rust 开发，启动速度快，内存占用小
- 📦 **模板管理**: 支持从 Git 仓库添加、删除和管理模板
- 🎨 **交互式创建**: 提供友好的交互式项目创建向导
- 🔧 **自动配置**: 自动更新 package.json 并初始化 Git 仓库
- 💾 **智能缓存**: 高效的模板缓存机制，避免重复下载
- 📤 **配置管理**: 支持配置的导出和导入
- 🛡️ **跨平台**: 支持 Windows、macOS 和 Linux

## 📊 性能对比

| 特性 | Mammoth CLI | 其他脚手架工具 |
|------|-------------|----------------|
| 启动时间 | 快速启动（Rust 原生） | 相对较慢（需要 Node.js 运行时） |
| 内存占用 | 低内存占用 | 较高内存占用（Node.js 运行时开销） |
| 模板下载 | 稀疏检出（只下载需要的文件） | 完整克隆（下载整个仓库） |
| 跨平台 | 原生支持（单一二进制文件） | 依赖 Node.js 环境 |
| 依赖数量 | 9个核心依赖 | 通常 50+ 个依赖 |
| 二进制大小 | 较小（Rust 编译优化） | 较大（包含 Node.js 运行时） |
| 冷启动 | 无需预热，即时可用 | 可能需要预热 Node.js 运行时 |

> **注意**: 以上对比基于 Rust 和 Node.js 的技术特性，具体性能数据需要在实际环境中进行基准测试验证。

**最新基准测试结果（Criterion 实测）**：

- 启动时间: ≈678ms
- 配置解析: ≈0.0033ms
- 文件复制: ≈41ms
- 模板下载（稀疏检出模拟）: ≈4.4ms
- 集成测试（完整项目创建流程）: ≈7.9ms

## 🚀 快速开始

```bash
# 1. 安装
cargo install mammoth-cli

# 2. 添加模板仓库
mammoth-cli repo add aio-templates \
  --url https://github.com/Mulander-J/aio-templates \
  --branch main

# 3. 添加一个模板
mammoth-cli template add nuxt-shadcn \
  --name "Nuxt Shadcn Starter" \
  --repo aio-templates \
  --path "vue/nuxt-shadcn" \
  --description "Nuxt Shadcn with Tailwind" \
  --language vue \
  --tags "nuxt,shadcn,tailwind"

# 4. 下载模板
mammoth-cli template download nuxt-shadcn

# 或者直接指定模板创建
mammoth-cli new --template nuxt-shadcn --name my-project
```

> **注意**: 完整的使用示例和命令说明请参考 [README.md](./README.md)

## 🛠️ 开发指南

### 开发环境设置

确保您的系统已安装以下工具：

- **Rust**: 最新稳定版本 (推荐 1.70+)
- **Git**: 用于版本控制和模板下载
- **Cargo**: Rust 包管理器（随 Rust 安装）

### 构建和运行

```bash
# 构建项目
cargo build

# 运行开发模式
cargo run

# 运行并显示详细输出
cargo run -- --verbose

# 检查编译错误
cargo check

# 运行测试
cargo test
```

### 开发工作流

1. **代码检查**: 使用 `cargo check` 检查语法错误
2. **测试**: 运行 `cargo test` 确保功能正常
3. **构建**: 使用 `cargo build` 构建项目
4. **运行**: 使用 `cargo run` 测试功能

### 调试技巧

- 使用 `--verbose` 参数获取详细输出
- 在代码中添加 `println!` 或 `dbg!` 进行调试
- 使用 `cargo test -- --nocapture` 查看测试输出

## 🔬 研发设计

### 🏗️ 整体架构图

```text
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   CLI Layer     │    │  Manager Layer  │    │  Config Layer   │
│                 │    │                 │    │                 │
│  - cli.rs       │◄──►│  - manager.rs   │◄──►│  - config.rs    │
│  - main.rs      │    │  - project.rs   │    │  - utils.rs     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   External      │    │   File System   │    │   Git Repos     │
│   Dependencies  │    │                 │    │                 │
│                 │    │  - Templates    │    │  - Remote       │
│  - clap         │    │  - Cache        │    │  - Config       │
│  - tokio        │    │  - Config       │    │  - Checkout     │
│  - serde        │    └─────────────────┘    └─────────────────┘
└─────────────────┘
```

### 📁 项目结构

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
├── Cargo.lock
├── README.md
├── doc.md
├── example.config.json
└── LICENSE
```

### 📦 依赖关系

```toml
[dependencies]
# CLI参数解析和命令行界面框架，支持derive宏
clap = { version = "4.0", features = ["derive", "std", "help"], default-features = false }
# 错误处理库，提供简洁的错误类型和传播
anyhow = "1.0" 
# 序列化和反序列化框架，支持derive宏
serde = { version = "1.0", features = ["derive"], default-features = false }
# JSON序列化和反序列化实现
serde_json = "1.0"
# 异步运行时，提供完整的异步功能支持
tokio = { version = "1.0", features = ["rt-multi-thread", "macros", "time", "process"] }
# 终端文本颜色和样式控制
colored = "2.0"
# 交互式命令行提示和输入库
dialoguer = "0.11"
# 进度条和加载动画显示
indicatif = "0.17"
# 跨平台系统目录路径获取
dirs = "5.0" 
```

### ⚡ 核心功能

#### 1. 模板管理

- **添加模板**: 支持从远程仓库添加新模板
- **删除模板**: 移除不需要的模板
- **列表显示**: 查看所有可用模板
- **下载更新**: 支持强制更新模板

```rust
// 模板数据结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Template {
    pub id: String,           // 模板唯一标识
    pub name: String,         // 模板显示名称
    pub repo: String,         // 所属仓库
    pub path: String,         // 仓库中的路径
    pub description: String,  // 模板描述
    pub language: String,     // 编程语言
    pub tags: Vec<String>,    // 标签
}
```

#### 2. 仓库管理

- **添加仓库**: 支持添加多个远程模板仓库（包括私有仓库）
- **删除仓库**: 移除不需要的仓库
- **稀疏检出**: 使用 Git sparse-checkout 只下载需要的模板文件
- **私有仓库支持**: 支持通过用户名和认证令牌访问私有仓库

```rust
// 仓库数据结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Repo {
    pub name: String,         // 仓库名称
    pub url: String,          // Git 仓库 URL
    pub branch: String,       // 分支名称
    pub auth_token: Option<String>,  // 私有仓库认证令牌
    pub username: Option<String>,    // 私有仓库用户名
}
```

**私有仓库配置示例**:

```bash
# 添加私有仓库
mammoth-cli repo add private-templates \
  --url https://github.com/your-org/private-templates \
  --branch main \
  --username your-username \
  --auth-token your-personal-access-token
```

**支持的认证方式**:

- **HTTPS + 个人访问令牌**: 用于 GitHub/GitLab 私有仓库
- **SSH 密钥**: 自动使用系统配置的 SSH 密钥 【更推荐🔥】
- **HTTP 基本认证**: 支持用户名/密码组合

#### 3. 项目创建

- 交互式流程
  1. **模板选择**: 从可用模板中选择
  2. **项目信息**: 输入项目名称、作者、描述等
  3. **配置确认**: 显示项目摘要并确认
  4. **项目生成**: 复制模板文件并配置项目
- 自动配置
  1. 更新 `package.json` 中的项目信息
  2. 初始化 Git 仓库
  3. 设置项目目录结构

#### 4. 配置管理

- **导出配置**: 将当前配置保存到文件
- **导入配置**: 从文件恢复配置
- **配置验证**: 验证配置文件的完整性
- **合并模式**: 支持合并和覆盖两种导入模式

## 📚 参考资料

- [Rust 官方文档](https://doc.rust-lang.org/)
- [Tokio 异步运行时](https://tokio.rs/)
- [Clap CLI 框架](https://clap.rs/)
- [Git Sparse Checkout](https://git-scm.com/docs/git-sparse-checkout)
- [cargo-generate](https://github.com/cargo-generate/cargo-generate)
