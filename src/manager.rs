use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use crate::config::{Config, Repo, Template};
use crate::utils::copy_directory;
use colored::*;
use dialoguer::Confirm;
use serde_json;

pub struct TemplateManager {
    pub config: Config,
    cache_dir: PathBuf,
}

impl TemplateManager {
    pub fn new() -> Result<Self> {
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
    
    pub fn save_config(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;
        let content =
            serde_json::to_string_pretty(&self.config).context("Failed to serialize config")?;
        fs::write(config_path, content).context("Failed to write config file")?;
        Ok(())
    }
    
    pub fn get_template_by_id(&self, id: &str) -> Option<&Template> {
        self.config.templates.iter().find(|t| t.id == id)
    }
    
    pub fn get_repo_by_name(&self, name: &str) -> Option<&Repo> {
        self.config.repos.iter().find(|r| r.name == name)
    }
    
    fn get_template_cache_path(&self, template: &Template) -> PathBuf {
        self.cache_dir.join(&template.repo).join(&template.id)
    }
    
    pub async fn download_template(&self, template: &Template, force: bool) -> Result<()> {
        let repo = self
            .get_repo_by_name(&template.repo)
            .ok_or_else(|| anyhow::anyhow!("Repository '{}' not found", template.repo))?;
        
        let cache_path = self.get_template_cache_path(template);
        
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
        
        // Create temporary directory for sparse clone
        let temp_dir = self.cache_dir.join(format!("temp_{}", repo.name));
        
        // 确保清理旧的临时目录
        self.cleanup_temp_dir(&temp_dir)?;
        fs::create_dir_all(&temp_dir).context("Failed to create temp dir")?;
        
        // 使用 Result 来确保清理操作
        let result = self.download_template_internal(template, repo, &temp_dir, &cache_path, &pb).await;
        
        // 无论成功还是失败，都尝试清理临时目录
        if let Err(ref e) = result {
            eprintln!("❌ Download failed: {}", e);
        }
        
        // 清理临时目录
        self.cleanup_temp_dir(&temp_dir)?;
        
        result
    }
    
    async fn download_template_internal(
        &self,
        template: &Template,
        repo: &Repo,
        temp_dir: &Path,
        cache_path: &Path,
        pb: &ProgressBar,
    ) -> Result<()> {
        pb.set_message("Preparing sparse checkout...");
        pb.inc(20);
        
        // Clone repository with sparse checkout and timeout
        pb.set_message("Cloning repository...");
        pb.inc(30);
        
        let clone_result = tokio::time::timeout(
            std::time::Duration::from_secs(300), // 5分钟超时
            tokio::process::Command::new("git")
                .args([
                    "clone",
                    "--no-checkout",
                    "--filter=blob:none",
                    "--sparse",
                    &repo.url,
                    &temp_dir.to_string_lossy(),
                ])
                .status(),
        )
        .await;
        
        let status = match clone_result {
            Ok(Ok(status)) => status,
            Ok(Err(e)) => anyhow::bail!("Failed to clone repository: {}", e),
            Err(_) => anyhow::bail!("Git clone timed out after 5 minutes"),
        };
        
        if !status.success() {
            anyhow::bail!("Failed to clone repository: {}", repo.url);
        }
        
        // Set sparse checkout directory
        pb.set_message("Configuring sparse checkout...");
        pb.inc(40);
        
        let sparse_result = tokio::time::timeout(
            std::time::Duration::from_secs(60), // 1分钟超时
            tokio::process::Command::new("git")
                .args(["sparse-checkout", "set", &template.path])
                .current_dir(temp_dir)
                .status(),
        )
        .await;
        
        let status = match sparse_result {
            Ok(Ok(status)) => status,
            Ok(Err(e)) => anyhow::bail!("Failed to set sparse checkout: {}", e),
            Err(_) => anyhow::bail!("Sparse checkout timed out"),
        };
        
        if !status.success() {
            anyhow::bail!("Failed to set sparse checkout for path: {}", template.path);
        }
        
        // Checkout the specific branch
        pb.set_message("Checking out files...");
        pb.inc(50);
        
        let checkout_result = tokio::time::timeout(
            std::time::Duration::from_secs(120), // 2分钟超时
            tokio::process::Command::new("git")
                .args(["checkout", &repo.branch])
                .current_dir(temp_dir)
                .status(),
        )
        .await;
        
        let status = match checkout_result {
            Ok(Ok(status)) => status,
            Ok(Err(e)) => anyhow::bail!("Failed to checkout branch: {}", e),
            Err(_) => anyhow::bail!("Git checkout timed out"),
        };
        
        if !status.success() {
            anyhow::bail!("Failed to checkout branch: {}", repo.branch);
        }
        
        // Create target directory
        fs::create_dir_all(cache_path.parent().unwrap())
            .context("Failed to create repo cache parent dir")?;
        
        // Move template files to cache location
        pb.set_message("Copying template files...");
        pb.inc(60);
        
        let template_source = temp_dir.join(&template.path);
        if !template_source.exists() {
            anyhow::bail!("Template path '{}' not found in repository", template.path);
        }
        
        // 安全地清理和复制文件
        self.safe_copy_template_files(&template_source, cache_path)?;
        
        pb.finish_with_message("Template downloaded successfully!");
        println!(
            "✅ Template '{}' downloaded to: {}",
            template.id,
            cache_path.display()
        );
        
        Ok(())
    }
    
    fn cleanup_temp_dir(&self, temp_dir: &Path) -> Result<()> {
        if temp_dir.exists() {
            // 在 Windows 上，可能需要多次尝试删除
            for attempt in 1..=3 {
                match fs::remove_dir_all(temp_dir) {
                    Ok(_) => {
                        if attempt > 1 {
                            println!("✅ Temp directory cleaned on attempt {}", attempt);
                        }
                        return Ok(());
                    }
                    Err(e) => {
                        if attempt == 3 {
                            eprintln!("⚠️  Warning: Failed to remove temp dir after 3 attempts: {}", e);
                            return Err(e.into());
                        }
                        // 等待一小段时间再重试
                        std::thread::sleep(std::time::Duration::from_millis(500));
                    }
                }
            }
        }
        Ok(())
    }
    
    fn safe_copy_template_files(&self, source: &Path, dest: &Path) -> Result<()> {
        // 如果目标目录存在，先尝试删除
        if dest.exists() {
            // 在 Windows 上，可能需要多次尝试
            for attempt in 1..=3 {
                match fs::remove_dir_all(dest) {
                    Ok(_) => break,
                    Err(e) => {
                        if attempt == 3 {
                            anyhow::bail!("Failed to remove old cache after 3 attempts: {}", e);
                        }
                        std::thread::sleep(std::time::Duration::from_millis(500));
                    }
                }
            }
        }
        
        // 复制文件
        copy_directory(source, dest).context("Failed to copy template files")?;
        
        Ok(())
    }
    
    pub async fn download_all_templates(&self, force: bool) -> Result<()> {
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
    
    pub fn list_templates(&self, verbose: bool) {
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
    
    pub fn add_template(
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
                "Repository '{}' not found. Add it first with 'repo add'",
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
    
    pub fn remove_template(&mut self, id: &str) -> Result<()> {
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
    
    pub fn add_repo(&mut self, name: String, url: String, branch: String) -> Result<()> {
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
    
    pub fn remove_repo(&mut self, name: &str) -> Result<()> {
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
    
    pub fn copy_template_files(&self, template: &Template, project_path: &Path) -> Result<()> {
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
    
    pub fn list_repos(&self) {
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
    
    pub fn export_config(&self, output: &str, include_cache: bool) -> Result<()> {
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
    
    pub fn import_config(&mut self, file: &str, mode: &str, skip_validation: bool) -> Result<()> {
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
    
    pub fn validate_config_file(&self, file: &str) -> Result<()> {
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
    
    pub fn clean_templates(&mut self, all: bool, force: bool) -> Result<()> {
        if !force {
            let message = if all {
                "⚠️  This will remove ALL templates, cache, and configuration. Are you sure?"
            } else {
                "⚠️  This will remove ALL cached template files. Are you sure?"
            };
            
            let confirm = Confirm::new()
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
    
    pub fn show_info(&self, json: bool) -> Result<()> {
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