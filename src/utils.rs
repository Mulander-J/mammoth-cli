use anyhow::Result;
use serde_json;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::config::ProjectConfig;

pub fn copy_directory(src: &Path, dst: &Path) -> Result<()> {
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

pub fn update_package_json(project_path: &Path, config: &ProjectConfig) -> Result<()> {
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

pub fn init_git_repository(project_path: &Path) -> Result<()> {
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