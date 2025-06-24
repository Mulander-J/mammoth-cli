use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use dialoguer::{Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser)]
#[command(name = "mammoth-cli")]
#[command(about = "Mammoth - A powerful frontend project scaffolding CLI tool")]
#[command(version)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new project
    New {
        /// Template ID
        #[arg(short, long)]
        template: Option<String>,

        /// Project name
        #[arg(short, long)]
        name: Option<String>,

        /// Output directory
        #[arg(short, long, default_value = ".")]
        output: String,
    },
    /// Clean configuration and cache
    Clean {
        /// Also remove configuration file
        #[arg(short, long)]
        all: bool,

        /// Skip confirmation
        #[arg(short, long)]
        force: bool,
    },
    /// Show configuration information
    Info {
        /// Show as JSON format
        #[arg(short, long)]
        json: bool,
    },
    /// Template management
    Template {
        #[command(subcommand)]
        command: TemplateCommands,
    },
    /// Repository management
    Repo {
        #[command(subcommand)]
        command: RepoCommands,
    },
    /// Configuration management
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Subcommand)]
enum TemplateCommands {
    /// List all available templates
    List {
        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },
    /// Download/update a specific template
    Download {
        /// Template ID
        template_id: String,

        /// Force update
        #[arg(short, long)]
        force: bool,
    },
    /// Download/update all templates
    DownloadAll {
        /// Force update
        #[arg(short, long)]
        force: bool,
    },
    /// Add a new template
    Add {
        /// Template ID
        template_id: String,

        /// Template name
        #[arg(short, long)]
        name: String,

        /// Repository name
        #[arg(short, long)]
        repo: String,

        /// Template path in repository
        #[arg(short, long)]
        path: String,

        /// Template description
        #[arg(short, long)]
        description: String,

        /// Language
        #[arg(short, long, default_value = "vue")]
        language: String,

        /// Tags (comma-separated)
        #[arg(short, long)]
        tags: Option<String>,
    },
    /// Remove a template
    Remove {
        /// Template ID
        template_id: String,
    },
}

#[derive(Subcommand)]
enum RepoCommands {
    /// Add a new repository
    Add {
        /// Repository name
        repo_name: String,

        /// Repository URL
        #[arg(short, long)]
        url: String,

        /// Branch
        #[arg(short, long, default_value = "main")]
        branch: String,
    },
    /// Remove a repository
    Remove {
        /// Repository name
        repo_name: String,
    },
    /// List all repositories
    List,
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Export configuration to file
    Export {
        /// Output file path
        #[arg(short, long)]
        output: String,

        /// Include cache information
        #[arg(short, long)]
        include_cache: bool,
    },
    /// Import configuration from file
    Import {
        /// Input file path
        #[arg(short, long)]
        file: String,

        /// Import mode: merge (default) or overwrite
        #[arg(short, long, default_value = "merge")]
        mode: String,

        /// Skip validation
        #[arg(short, long)]
        skip_validation: bool,
    },
    /// Validate configuration file
    Validate {
        /// Configuration file path
        file: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Repo {
    name: String,
    url: String,
    branch: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Template {
    id: String,
    name: String,
    repo: String,
    path: String,
    description: String,
    language: String,
    tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Config {
    repos: Vec<Repo>,
    templates: Vec<Template>,
}

#[derive(Debug)]
struct ProjectConfig {
    name: String,
    author: String,
    description: String,
    output_dir: String,
    template: Template,
}

struct TemplateManager {
    config: Config,
    cache_dir: PathBuf,
}

impl TemplateManager {
    fn new() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        let config = if config_path.exists() {
            let content = fs::read_to_string(&config_path).context("Failed to read config file")?;
            serde_json::from_str(&content).context("Failed to parse config file")?
        } else {
            Config {
                repos: vec![],
                templates: vec![],
            }
        };

        let cache_dir = Self::get_cache_dir()?;
        fs::create_dir_all(&cache_dir).context("Failed to create cache directory")?;

        Ok(Self { config, cache_dir })
    }

    fn get_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from(".config"))
            .join("mammoth-cli");
        fs::create_dir_all(&config_dir).context("Failed to create config directory")?;
        Ok(config_dir.join("templates.json"))
    }

    fn get_cache_dir() -> Result<PathBuf> {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from(".cache"))
            .join("mammoth-cli")
            .join("templates");
        Ok(cache_dir)
    }

    fn save_config(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;
        let content =
            serde_json::to_string_pretty(&self.config).context("Failed to serialize config")?;
        fs::write(config_path, content).context("Failed to write config file")?;
        Ok(())
    }

    fn get_template_by_id(&self, id: &str) -> Option<&Template> {
        self.config.templates.iter().find(|t| t.id == id)
    }

    fn get_repo_by_name(&self, name: &str) -> Option<&Repo> {
        self.config.repos.iter().find(|r| r.name == name)
    }

    fn get_template_cache_path(&self, template: &Template) -> PathBuf {
        self.cache_dir.join(&template.repo).join(&template.id)
    }

    async fn download_template(&self, template: &Template, force: bool) -> Result<()> {
        let repo = self
            .get_repo_by_name(&template.repo)
            .ok_or_else(|| anyhow::anyhow!("Repository '{}' not found", template.repo))?;

        let cache_path = self.get_template_cache_path(template);
        let repo_cache_path = cache_path.parent().unwrap();

        if cache_path.exists() && !force {
            println!("✨ Template '{}' already cached", template.id);
            return Ok(());
        }

        println!("🚀 Downloading template '{}'...", template.id);

        // Create progress bar
        let pb = ProgressBar::new(100);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
                )
                .unwrap()
                .progress_chars("#>-"),
        );

        pb.set_message("Preparing sparse checkout...");
        pb.inc(20);

        // Create temporary directory for sparse clone
        let temp_dir = self.cache_dir.join(format!("temp_{}", repo.name));
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).context("Failed to remove temp dir")?;
        }
        fs::create_dir_all(&temp_dir).context("Failed to create temp dir")?;

        // Clone repository with sparse checkout
        pb.set_message("Cloning repository...");
        pb.inc(30);
        let status = Command::new("git")
            .args([
                "clone",
                "--no-checkout",
                "--filter=blob:none",
                "--sparse",
                &repo.url,
                &temp_dir.to_string_lossy(),
            ])
            .status()
            .context("Failed to start git clone")?;
        if !status.success() {
            anyhow::bail!("Failed to clone repository: {}", repo.url);
        }

        // Set sparse checkout directory
        pb.set_message("Configuring sparse checkout...");
        pb.inc(40);
        let status = Command::new("git")
            .args(["sparse-checkout", "set", &template.path])
            .current_dir(&temp_dir)
            .status()
            .context("Failed to start git sparse-checkout")?;
        if !status.success() {
            anyhow::bail!("Failed to set sparse checkout for path: {}", template.path);
        }

        // Checkout the specific branch
        pb.set_message("Checking out files...");
        pb.inc(50);
        let status = Command::new("git")
            .args(["checkout", &repo.branch])
            .current_dir(&temp_dir)
            .status()
            .context("Failed to start git checkout")?;
        if !status.success() {
            anyhow::bail!("Failed to checkout branch: {}", repo.branch);
        }

        // Create target directory
        fs::create_dir_all(repo_cache_path.parent().unwrap())
            .context("Failed to create repo cache parent dir")?;

        // Move template files to cache location
        pb.set_message("Copying template files...");
        pb.inc(60);
        let template_source = temp_dir.join(&template.path);
        if !template_source.exists() {
            anyhow::bail!("Template path '{}' not found in repository", template.path);
        }
        if cache_path.exists() {
            fs::remove_dir_all(&cache_path).context("Failed to remove old cache")?;
        }
        copy_directory(&template_source, &cache_path).context("Failed to copy template files")?;

        // Clean up temporary directory
        pb.set_message("Cleaning up...");
        pb.inc(80);

        if let Err(e) = fs::remove_dir_all(&temp_dir) {
            eprintln!("⚠️  Warning: Failed to remove temp dir after copy: {}", e);
        }

        pb.finish_with_message("Template downloaded successfully!");
        println!(
            "✅ Template '{}' downloaded to: {}",
            template.id,
            cache_path.display()
        );

        Ok(())
    }

    async fn download_all_templates(&self, force: bool) -> Result<()> {
        println!("🚀 Downloading all templates...");

        for template in &self.config.templates {
            match self.download_template(template, force).await {
                Ok(_) => {}
                Err(e) => {
                    println!("❌ Failed to download template '{}': {}", template.id, e);
                }
            }
        }

        println!("🎉 All templates downloaded!");
        Ok(())
    }

    fn list_templates(&self, verbose: bool) {
        if verbose {
            println!("{}", "📋 Available Templates".bold().blue());
        } else {
            println!("{}", "📋 Template List".bold().blue());
        }
        println!();

        if self.config.templates.is_empty() {
            println!("No templates available. Add templates first.");
            return;
        }

        for template in &self.config.templates {
            let cache_path = self.get_template_cache_path(template);
            let status = if cache_path.exists() {
                "✅".green()
            } else {
                "❌".red()
            };

            if verbose {
                // 全信息显示模式
                println!("{} {} - {}", status, template.id.bold(), template.name);
                println!("   Description: {}", template.description);
                println!("   Language: {}", template.language);
                println!("   Repository: {}", template.repo);
                println!("   Path: {}", template.path);
                println!("   Tags: {}", template.tags.join(", "));
                println!();
            } else {
                // 简要信息显示模式
                println!(
                    "{} {} - {} ({})",
                    status,
                    template.id.bold(),
                    template.name,
                    template.language
                );
            }
        }

        if !verbose {
            println!();
            println!("💡 Use --verbose to see detailed information");
        }
    }

    fn add_template(
        &mut self,
        id: String,
        name: String,
        repo: String,
        path: String,
        description: String,
        language: String,
        tags: Option<String>,
    ) -> Result<()> {
        // Verify repository exists
        if !self.config.repos.iter().any(|r| r.name == repo) {
            anyhow::bail!(
                "Repository '{}' not found. Add it first with 'template repo-add'",
                repo
            );
        }

        // Check if template ID already exists
        if self.config.templates.iter().any(|t| t.id == id) {
            anyhow::bail!("Template with ID '{}' already exists", id);
        }

        // Parse tags
        let tags_vec = if let Some(tags_str) = tags {
            tags_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            vec![]
        };

        let template = Template {
            id,
            name,
            repo,
            path,
            description,
            language,
            tags: tags_vec,
        };

        self.config.templates.push(template);
        self.save_config()?;

        println!("🎉 Template added successfully!");
        Ok(())
    }

    fn remove_template(&mut self, id: &str) -> Result<()> {
        let index = self.config.templates.iter().position(|t| t.id == id);

        if let Some(index) = index {
            self.config.templates.remove(index);
            self.save_config()?;
            println!("🗑️  Template '{}' removed successfully!", id);
        } else {
            anyhow::bail!("Template '{}' not found", id);
        }

        Ok(())
    }

    fn add_repo(&mut self, name: String, url: String, branch: String) -> Result<()> {
        // Check if repository already exists
        if self.config.repos.iter().any(|r| r.name == name) {
            anyhow::bail!("Repository '{}' already exists", name);
        }

        let repo = Repo { name, url, branch };
        self.config.repos.push(repo);
        self.save_config()?;

        println!("🎉 Repository added successfully!");
        Ok(())
    }

    fn remove_repo(&mut self, name: &str) -> Result<()> {
        // Check if any templates use this repository
        if self.config.templates.iter().any(|t| t.repo == name) {
            anyhow::bail!(
                "Cannot remove repository '{}' - it is used by templates",
                name
            );
        }

        let index = self.config.repos.iter().position(|r| r.name == name);

        if let Some(index) = index {
            self.config.repos.remove(index);
            self.save_config()?;
            println!("🗑️  Repository '{}' removed successfully!", name);
        } else {
            anyhow::bail!("Repository '{}' not found", name);
        }

        Ok(())
    }

    fn copy_template_files(&self, template: &Template, project_path: &Path) -> Result<()> {
        let cache_path = self.get_template_cache_path(template);

        if !cache_path.exists() {
            anyhow::bail!(
                "Template '{}' not cached. Run 'template download {}' first",
                template.id,
                template.id
            );
        }

        copy_directory(&cache_path, project_path)?;
        Ok(())
    }

    fn list_repos(&self) {
        println!("{}", "📦 Configured Template Repositories".bold().blue());
        println!();
        if self.config.repos.is_empty() {
            println!("No repositories configured. Add repositories first.");
            return;
        }
        for repo in &self.config.repos {
            println!("{} - {}", repo.name.bold(), repo.url);
            println!("   🪐Branch: {}", repo.branch);
            println!();
        }
    }

    fn export_config(&self, output: &str, include_cache: bool) -> Result<()> {
        println!("📤 Exporting configuration to: {}", output);

        let export_config = Config {
            repos: self.config.repos.clone(),
            templates: self.config.templates.clone(),
        };

        // 如果包含缓存信息，添加缓存状态
        if include_cache {
            println!("📦 Including cache information...");
            // 这里可以添加缓存相关的元数据
        }

        let content = serde_json::to_string_pretty(&export_config)
            .context("Failed to serialize configuration")?;

        fs::write(output, content)
            .with_context(|| format!("Failed to write configuration to: {}", output))?;

        println!("✅ Configuration exported successfully!");
        println!(
            "📊 Exported {} repositories and {} templates",
            export_config.repos.len(),
            export_config.templates.len()
        );

        Ok(())
    }

    fn import_config(&mut self, file: &str, mode: &str, skip_validation: bool) -> Result<()> {
        println!("📥 Importing configuration from: {}", file);

        let config_content = fs::read_to_string(file)
            .with_context(|| format!("Failed to read configuration file: {}", file))?;

        let import_config: Config =
            serde_json::from_str(&config_content).context("Failed to parse configuration file")?;

        if !skip_validation {
            self.validate_import_config(&import_config)?;
        }

        match mode.to_lowercase().as_str() {
            "merge" => {
                println!("🔄 Merging configuration...");
                self.merge_config(import_config)?;
            }
            "overwrite" => {
                println!("⚠️  Overwriting configuration...");
                self.config = import_config;
            }
            _ => {
                anyhow::bail!("Invalid import mode: {}. Use 'merge' or 'overwrite'", mode);
            }
        }

        self.save_config()?;

        println!("✅ Configuration imported successfully!");
        println!(
            "📊 Current configuration: {} repositories and {} templates",
            self.config.repos.len(),
            self.config.templates.len()
        );

        Ok(())
    }

    fn validate_config_file(&self, file: &str) -> Result<()> {
        println!("🔍 Validating configuration file: {}", file);

        let config_content = fs::read_to_string(file)
            .with_context(|| format!("Failed to read configuration file: {}", file))?;

        let config: Config =
            serde_json::from_str(&config_content).context("Failed to parse configuration file")?;

        self.validate_import_config(&config)?;

        println!("✅ Configuration file is valid!");
        println!(
            "📊 Contains {} repositories and {} templates",
            config.repos.len(),
            config.templates.len()
        );

        Ok(())
    }

    fn validate_import_config(&self, import_config: &Config) -> Result<()> {
        let mut validation_errors = Vec::new();
        let mut validation_warnings = Vec::new();

        // 验证仓库配置
        for repo in &import_config.repos {
            if repo.name.is_empty() {
                validation_errors.push("Repository name cannot be empty".to_string());
            }
            if repo.url.is_empty() {
                validation_errors.push(format!("Repository '{}' URL cannot be empty", repo.name));
            }
            if repo.branch.is_empty() {
                validation_errors
                    .push(format!("Repository '{}' branch cannot be empty", repo.name));
            }
        }

        // 验证模板配置
        for template in &import_config.templates {
            if template.id.is_empty() {
                validation_errors.push("Template ID cannot be empty".to_string());
            }
            if template.name.is_empty() {
                validation_errors.push(format!("Template '{}' name cannot be empty", template.id));
            }
            if template.repo.is_empty() {
                validation_errors.push(format!(
                    "Template '{}' repository cannot be empty",
                    template.id
                ));
            }
            if template.path.is_empty() {
                validation_errors.push(format!("Template '{}' path cannot be empty", template.id));
            }

            // 检查模板引用的仓库是否存在
            if !import_config.repos.iter().any(|r| r.name == template.repo) {
                validation_warnings.push(format!(
                    "Template '{}' references non-existent repository '{}'",
                    template.id, template.repo
                ));
            }
        }

        // 报告错误和警告
        if !validation_errors.is_empty() {
            println!("❌ Validation errors:");
            for error in validation_errors {
                println!("  {}", error);
            }
            anyhow::bail!("Configuration validation failed");
        }

        if !validation_warnings.is_empty() {
            println!("⚠️  Validation warnings:");
            for warning in validation_warnings {
                println!("  {}", warning);
            }
        }

        Ok(())
    }

    fn merge_config(&mut self, import_config: Config) -> Result<()> {
        let mut merged_repos = 0;
        let mut merged_templates = 0;

        // 合并仓库
        for import_repo in import_config.repos {
            if let Some(existing_repo) = self
                .config
                .repos
                .iter_mut()
                .find(|r| r.name == import_repo.name)
            {
                // 更新现有仓库
                existing_repo.url = import_repo.url;
                existing_repo.branch = import_repo.branch;
                merged_repos += 1;
            } else {
                // 添加新仓库
                self.config.repos.push(import_repo);
                merged_repos += 1;
            }
        }

        // 合并模板
        for import_template in import_config.templates {
            if let Some(existing_template) = self
                .config
                .templates
                .iter_mut()
                .find(|t| t.id == import_template.id)
            {
                // 更新现有模板
                *existing_template = import_template;
                merged_templates += 1;
            } else {
                // 添加新模板
                self.config.templates.push(import_template);
                merged_templates += 1;
            }
        }

        println!(
            "📊 Merged {} repositories and {} templates",
            merged_repos, merged_templates
        );

        Ok(())
    }

    fn clean_templates(&mut self, all: bool, force: bool) -> Result<()> {
        if !force {
            let message = if all {
                "⚠️  This will remove ALL templates, cache, and configuration. Are you sure?"
            } else {
                "⚠️  This will remove ALL cached template files. Are you sure?"
            };

            let confirm = dialoguer::Confirm::new()
                .with_prompt(message)
                .default(false)
                .interact()?;

            if !confirm {
                println!("❌ Clean operation cancelled");
                return Ok(());
            }
        }

        println!("🧹 Cleaning templates...");

        // 清理缓存目录
        if self.cache_dir.exists() {
            match fs::remove_dir_all(&self.cache_dir) {
                Ok(_) => println!("✅ Cache directory cleaned"),
                Err(e) => println!("⚠️  Failed to clean cache directory: {}", e),
            }
        }

        // 重新创建缓存目录
        fs::create_dir_all(&self.cache_dir).context("Failed to recreate cache directory")?;

        if all {
            // 清理配置文件
            let config_path = Self::get_config_path()?;
            if config_path.exists() {
                match fs::remove_file(&config_path) {
                    Ok(_) => println!("✅ Configuration file removed"),
                    Err(e) => println!("⚠️  Failed to remove configuration file: {}", e),
                }
            }

            // 重置配置
            self.config = Config {
                repos: vec![],
                templates: vec![],
            };
        }

        println!("🎉 Clean operation completed!");
        if all {
            println!("📝 Configuration has been reset to empty state");
        } else {
            println!("💾 Configuration preserved, only cache was cleaned");
        }

        Ok(())
    }

    fn show_info(&self, json: bool) -> Result<()> {
        if json {
            // 以JSON格式显示配置
            let config_json = serde_json::to_string_pretty(&self.config)
                .context("Failed to serialize configuration")?;
            println!("{}", config_json);
        } else {
            // 以友好格式显示配置信息
            println!("{}", "📋 Current Configuration".bold().blue());
            println!();

            // 显示仓库信息
            println!("{}", "📦 Repositories".bold().yellow());
            if self.config.repos.is_empty() {
                println!("  No repositories configured");
            } else {
                for repo in &self.config.repos {
                    println!("  {} - {}", repo.name.bold(), repo.url);
                    println!("    Branch: {}", repo.branch);
                }
            }
            println!();

            // 显示模板信息
            println!("{}", "🎨 Templates".bold().yellow());
            if self.config.templates.is_empty() {
                println!("  No templates configured");
            } else {
                for template in &self.config.templates {
                    let cache_path = self.get_template_cache_path(template);
                    let status = if cache_path.exists() {
                        "✅".green()
                    } else {
                        "❌".red()
                    };

                    println!("  {} {} - {}", status, template.id.bold(), template.name);
                    println!("    Description: {}", template.description);
                    println!("    Language: {}", template.language);
                    println!("    Repository: {}", template.repo);
                    println!("    Path: {}", template.path);
                    println!("    Tags: {}", template.tags.join(", "));
                    println!();
                }
            }

            // 显示统计信息
            println!("{}", "📊 Statistics".bold().yellow());
            println!("  Repositories: {}", self.config.repos.len());
            println!("  Templates: {}", self.config.templates.len());

            // 显示缓存状态
            let cached_count = self
                .config
                .templates
                .iter()
                .filter(|t| self.get_template_cache_path(t).exists())
                .count();
            println!(
                "  Cached templates: {}/{}",
                cached_count,
                self.config.templates.len()
            );

            // 显示配置路径
            println!();
            println!("{}", "📁 Paths".bold().yellow());
            match Self::get_config_path() {
                Ok(path) => println!("  Config: {}", path.display()),
                Err(_) => println!("  Config: Unable to determine path"),
            }
            println!("  Cache: {}", self.cache_dir.display());
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut manager = TemplateManager::new()?;

    match &cli.command {
        Some(Commands::New {
            template,
            name,
            output,
        }) => {
            new_project(&mut manager, template.as_deref(), name.as_deref(), output).await?;
        }
        Some(Commands::Template { command }) => match command {
            TemplateCommands::List { verbose } => {
                manager.list_templates(*verbose);
            }
            TemplateCommands::Download { template_id, force } => {
                let template = manager
                    .get_template_by_id(template_id)
                    .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", template_id))?;
                manager.download_template(template, *force).await?;
            }
            TemplateCommands::DownloadAll { force } => {
                manager.download_all_templates(*force).await?;
            }
            TemplateCommands::Add {
                template_id,
                name,
                repo,
                path,
                description,
                language,
                tags,
            } => {
                manager.add_template(
                    template_id.clone(),
                    name.clone(),
                    repo.clone(),
                    path.clone(),
                    description.clone(),
                    language.clone(),
                    tags.clone(),
                )?;
            }
            TemplateCommands::Remove { template_id } => {
                manager.remove_template(template_id)?;
            }
        },
        Some(Commands::Clean { all, force }) => {
            manager.clean_templates(*all, *force)?;
        }
        Some(Commands::Info { json }) => {
            manager.show_info(*json)?;
        }
        Some(Commands::Repo { command }) => match command {
            RepoCommands::Add {
                repo_name,
                url,
                branch,
            } => {
                manager.add_repo(repo_name.clone(), url.clone(), branch.clone())?;
            }
            RepoCommands::Remove { repo_name } => {
                manager.remove_repo(repo_name)?;
            }
            RepoCommands::List => {
                manager.list_repos();
            }
        },
        Some(Commands::Config { command }) => match command {
            ConfigCommands::Export {
                output,
                include_cache,
            } => {
                manager.export_config(output, *include_cache)?;
            }
            ConfigCommands::Import {
                file,
                mode,
                skip_validation,
            } => {
                manager.import_config(file, mode, *skip_validation)?;
            }
            ConfigCommands::Validate { file } => {
                manager.validate_config_file(file)?;
            }
        },
        None => {
            // Default to new project creation
            new_project(&mut manager, None, None, ".").await?;
        }
    }

    Ok(())
}

async fn new_project(
    manager: &mut TemplateManager,
    template_id: Option<&str>,
    name: Option<&str>,
    output: &str,
) -> Result<()> {
    println!(
        "{}",
        "🚀 Welcome to mammoth-cli Frontend Scaffolding Tool!"
            .bold()
            .green()
    );
    println!();

    // Get project configuration through interactive prompts
    let config = get_project_config(manager, template_id, name, output).await?;

    // Generate the project
    generate_project(manager, &config).await?;

    println!();
    println!("{}", "🎉 Project generated successfully!".bold().green());
    println!(
        "📁 Project location: {}",
        Path::new(&config.output_dir).join(&config.name).display()
    );
    println!();
    println!("Next steps:");
    println!("  cd {}", config.name);
    println!("  npm install  # or pnpm install");
    println!("  npm run dev  # or pnpm dev");

    Ok(())
}

async fn get_project_config(
    manager: &TemplateManager,
    template_id: Option<&str>,
    name: Option<&str>,
    output: &str,
) -> Result<ProjectConfig> {
    // Template selection
    let template = if let Some(id) = template_id {
        manager
            .get_template_by_id(id)
            .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", id))?
    } else {
        println!("{}", "🎨 Step 1: Select Template".bold().blue());

        if manager.config.templates.is_empty() {
            anyhow::bail!("No templates available. Add templates first with 'template add'");
        }

        let template_names: Vec<String> = manager
            .config
            .templates
            .iter()
            .map(|t| format!("{} - {}", t.id, t.description))
            .collect();

        let template_selection = Select::new()
            .with_prompt("Choose a template")
            .items(&template_names)
            .default(0)
            .interact()?;

        &manager.config.templates[template_selection]
    };

    println!("✨ Selected template: {}", template.id.green());
    println!();

    // Project information
    println!("{}", "📋 Step 2: Project Information".bold().blue());

    let project_name: String = if let Some(n) = name {
        n.to_string()
    } else {
        Input::new()
            .with_prompt("Project name")
            .with_initial_text("my-awesome-project")
            .interact_text()?
    };

    let author: String = Input::new()
        .with_prompt("Author name")
        .with_initial_text("Your Name")
        .interact_text()?;

    let description: String = Input::new()
        .with_prompt("Project description")
        .with_initial_text("A wonderful project")
        .interact_text()?;

    let output_dir: String = if output != "." {
        output.to_string()
    } else {
        Input::new()
            .with_prompt("Output directory")
            .with_initial_text(".")
            .interact_text()?
    };

    println!();
    println!("{}", "📊 Project Summary".bold().yellow());
    println!("Name: {}", project_name);
    println!("Author: {}", author);
    println!("Description: {}", description);
    println!("Template: {}", template.id);
    println!("Language: {}", template.language);
    println!("Output Directory: {}", output_dir);
    println!();

    // Confirmation
    let confirm = dialoguer::Confirm::new()
        .with_prompt("Do you want to proceed with project generation?")
        .default(true)
        .interact()?;

    if !confirm {
        println!("{}", "❌ Project generation cancelled".red());
        std::process::exit(0);
    }

    Ok(ProjectConfig {
        name: project_name,
        author,
        description,
        output_dir,
        template: template.clone(),
    })
}

async fn generate_project(manager: &TemplateManager, config: &ProjectConfig) -> Result<()> {
    println!("{}", "🔨 Generating project...".bold().blue());

    let project_path = Path::new(&config.output_dir).join(&config.name);

    // Create progress bar
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    pb.set_message("Creating project directory...");
    pb.inc(10);

    // Create project directory
    fs::create_dir_all(&project_path).with_context(|| {
        format!(
            "Failed to create project directory: {}",
            project_path.display()
        )
    })?;

    pb.set_message("Getting template files...");
    pb.inc(20);

    // Get template files (will download if not cached)
    manager.download_template(&config.template, false).await?;
    manager.copy_template_files(&config.template, &project_path)?;

    pb.set_message("Updating project configuration...");
    pb.inc(30);

    // Update package.json with project information
    update_package_json(&project_path, config)?;

    pb.set_message("Finalizing project...");
    pb.inc(40);

    // Initialize git repository
    init_git_repository(&project_path)?;

    pb.finish_with_message("Project generation completed!");

    Ok(())
}

fn copy_directory(src: &Path, dst: &Path) -> Result<()> {
    if src.is_file() {
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(src, dst)?;
    } else if src.is_dir() {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if src_path.is_dir() {
                copy_directory(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
        }
    }

    Ok(())
}

fn update_package_json(project_path: &Path, config: &ProjectConfig) -> Result<()> {
    let package_json_path = project_path.join("package.json");

    if !package_json_path.exists() {
        return Ok(()); // No package.json to update
    }

    let package_json_content = fs::read_to_string(&package_json_path)?;
    let mut package_json: serde_json::Value = serde_json::from_str(&package_json_content)?;

    // Update package.json fields
    if let Some(obj) = package_json.as_object_mut() {
        obj.insert(
            "name".to_string(),
            serde_json::Value::String(config.name.clone()),
        );
        obj.insert(
            "author".to_string(),
            serde_json::Value::String(config.author.clone()),
        );
        obj.insert(
            "description".to_string(),
            serde_json::Value::String(config.description.clone()),
        );
    }

    let updated_content = serde_json::to_string_pretty(&package_json)?;
    fs::write(&package_json_path, updated_content)?;

    Ok(())
}

fn init_git_repository(project_path: &Path) -> Result<()> {
    // Change to project directory
    let current_dir = std::env::current_dir()?;
    std::env::set_current_dir(project_path)?;

    // Initialize git repository
    let status = Command::new("git").args(["init"]).status();

    // Restore original directory
    std::env::set_current_dir(current_dir)?;

    match status {
        Ok(_) => {
            println!("🔧 Git repository initialized");
        }
        Err(_) => {
            println!("⚠️  Git not available, skipping repository initialization");
        }
    }

    Ok(())
}
