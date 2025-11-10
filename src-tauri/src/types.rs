use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModManifest {
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub mod_type: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub homepage: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub compatibility: Compatibility,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub conflicts: Vec<String>,
    #[serde(default)]
    pub load_after: Vec<String>,
    #[serde(default)]
    pub files: Vec<FileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Compatibility {
    #[serde(default)]
    pub fm_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub source: String,
    pub target_subpath: String,
    #[serde(default)]
    pub platform: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub target_path: Option<String>,
    #[serde(default)]
    pub user_dir_path: Option<String>,
    #[serde(default)]
    pub enabled_mods: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModInfo {
    pub name: String,
    pub version: String,
    pub mod_type: String,
    pub author: String,
    pub enabled: bool,
    pub has_conflicts: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestorePoint {
    pub timestamp: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictInfo {
    pub file_path: String,
    pub conflicting_mods: Vec<String>,
}
