//! Type Definitions
//!
//! This module contains all shared type definitions used across the application.
//! Types are organized by domain: mod management, configuration, graphics packs, and progress tracking.

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
pub struct ResolvedFilePreview {
    pub target_subpath: String,
    pub resolved_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModInstallPreview {
    pub base_target: String,
    pub resolved_files: Vec<ResolvedFilePreview>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub target_path: Option<String>,
    #[serde(default)]
    pub user_dir_path: Option<String>,
    #[serde(default)]
    pub enabled_mods: Vec<String>,
    #[serde(default)]
    pub dark_mode: bool,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub active_name_fix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameFixSource {
    pub id: String,
    pub name: String,
    pub source_type: NameFixSourceType,
    pub install_type: NameFixInstallType,
    pub description: String,
    pub imported_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NameFixSourceType {
    GitHub,   // Built-in GitHub download
    Imported, // User-imported ZIP/RAR
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NameFixInstallType {
    Files,   // FMScout style: Add specific .lnc/.edt/.dbc files
    Folders, // Sortitoutsi style: Replace entire dbc/edt/lnc folders + editor data
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
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
    pub name: String,
    pub timestamp: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictInfo {
    pub file_path: String,
    pub conflicting_mods: Vec<String>,
}

/// Progress tracking for archive extraction operations.
/// Emitted via Tauri events during async extraction of graphics packs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionProgress {
    pub current: usize,
    pub total: usize,
    pub current_file: String,
    pub bytes_processed: u64,
    pub phase: String, // "extracting" or "installing"
}

/// Progress tracking for file installation operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct InstallProgress {
    pub current: usize,
    pub total: usize,
    pub current_file: String,
    pub operation: String, // "copying", "validating", etc.
}

/// Metadata for an installed graphics pack.
/// Persisted to config.json as part of GraphicsPacksRegistry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsPackMetadata {
    pub id: String,
    pub name: String,
    pub install_date: String,
    pub file_count: usize,
    pub source_filename: String,
    pub pack_type: String, // "faces", "logos", "kits", "mixed"
    pub installed_to: String,
}

/// Registry of all installed graphics packs.
/// Stored in config.json to persist across application sessions.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GraphicsPacksRegistry {
    pub graphics_packs: Vec<GraphicsPackMetadata>,
}

/// Information about graphics pack installation conflicts.
/// Used to warn users when installing to directories with existing content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsConflictInfo {
    pub target_directory: String,
    pub existing_file_count: usize,
    pub pack_name: String,
}
