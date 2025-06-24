use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mammoth-cli")]
#[command(about = "Mammoth - A powerful frontend project scaffolding CLI tool")]
#[command(version)]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,
    
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
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
pub enum TemplateCommands {
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
pub enum RepoCommands {
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
pub enum ConfigCommands {
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