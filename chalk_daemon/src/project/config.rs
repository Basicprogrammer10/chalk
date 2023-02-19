use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectConfig {
    // Misc
    pub name: String,
    pub api_token: String,

    pub run: ProjectRunConfig,
    pub git: ProjectGitConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectRunConfig {
    pub path: String,
    pub command: String,
    pub arguments: Vec<String>,
    pub environment_vars: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectGitConfig {
    pub repo: Option<String>,
    pub username: Option<String>,
    pub token: Option<String>,
    pub ssh_key_file: Option<String>,
}
