use anyhow::{Context, Result};
use colored::*;
use dialoguer::{Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::Path;

use crate::config::ProjectConfig;
use crate::manager::TemplateManager;
use crate::utils::{init_git_repository, update_package_json};

pub async fn new_project(
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

pub async fn get_project_config(
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

pub async fn generate_project(manager: &TemplateManager, config: &ProjectConfig) -> Result<()> {
    println!("{}", "🔨 Generating project...".bold().blue());
    
    let project_path = Path::new(&config.output_dir).join(&config.name);
    
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