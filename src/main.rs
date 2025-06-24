use anyhow::Result;
use clap::Parser;

use mammoth_cli::{
    cli::{Cli, Commands, ConfigCommands, RepoCommands, TemplateCommands},
    manager::TemplateManager,
    project::new_project,
};

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