use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Repo {
    pub name: String,
    pub url: String,
    pub branch: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub repo: String,
    pub path: String,
    pub description: String,
    pub language: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub repos: Vec<Repo>,
    pub templates: Vec<Template>,
}

#[derive(Debug)]
pub struct ProjectConfig {
    pub name: String,
    pub author: String,
    pub description: String,
    pub output_dir: String,
    pub template: Template,
} 