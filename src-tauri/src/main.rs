// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod conflicts;
mod game_detection;
mod graphics_analyzer;
mod import;
mod logging;
mod mod_manager;
mod name_fix;
mod restore;
mod types;

use config::{
    get_mods_dir, init_storage, load_config, load_graphics_packs, save_config, save_graphics_packs,
};
use conflicts::find_conflicts;
use game_detection::get_default_candidates;
use import::{
    auto_detect_mod_type, extract_zip, extract_zip_async, find_mod_root, generate_manifest,
    has_manifest,
};
use mod_manager::{
    cleanup_old_backups, cleanup_old_restore_points, get_mod_info, install_mod, list_mods,
};
use restore::{create_restore_point, list_restore_points, rollback_to_restore_point};
use std::path::{Path, PathBuf};
use tauri::{Emitter, Manager};
use types::{
    Config, ConflictInfo, ExtractionProgress, GraphicsConflictInfo, GraphicsPackMetadata,
    ModManifest, RestorePoint,
};

#[tauri::command]
fn init_app() -> Result<(), String> {
    tracing::info!("Initializing application");
    init_storage()?;
    cleanup_old_backups(10)?;
    cleanup_old_restore_points(10)?;
    tracing::info!("Application initialized successfully");
    Ok(())
}

#[tauri::command]
fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
fn get_config() -> Result<Config, String> {
    load_config()
}

#[tauri::command]
fn update_config(config: Config) -> Result<(), String> {
    save_config(&config)
}

#[tauri::command]
fn detect_game_path() -> Result<Vec<String>, String> {
    let candidates = get_default_candidates();
    Ok(candidates
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect())
}

#[tauri::command]
fn set_game_target(path: String) -> Result<(), String> {
    let mut config = load_config()?;
    config.target_path = Some(path);
    save_config(&config)
}

#[tauri::command]
fn detect_user_dir() -> Result<String, String> {
    let config = load_config()?;
    let user_dir = game_detection::get_fm_user_dir(config.user_dir_path.as_deref());
    Ok(user_dir.to_string_lossy().to_string())
}

#[tauri::command]
fn get_mods_list() -> Result<Vec<String>, String> {
    list_mods()
}

#[tauri::command]
fn get_mod_details(mod_name: String) -> Result<ModManifest, String> {
    get_mod_info(&mod_name)
}

#[tauri::command]
fn enable_mod(mod_name: String) -> Result<(), String> {
    let mut config = load_config()?;

    if !config.enabled_mods.contains(&mod_name) {
        config.enabled_mods.push(mod_name);
        save_config(&config)?;
    }

    Ok(())
}

#[tauri::command]
fn disable_mod(mod_name: String) -> Result<(), String> {
    let mut config = load_config()?;

    config.enabled_mods.retain(|m| m != &mod_name);
    save_config(&config)?;

    Ok(())
}

#[tauri::command]
fn apply_mods() -> Result<String, String> {
    let config = load_config()?;

    let target_path = config.target_path.as_ref().ok_or("Game target not set")?;

    let target = std::path::PathBuf::from(target_path);

    if !target.exists() {
        return Err("Game target path does not exist".to_string());
    }

    let mut results = Vec::new();

    for mod_name in &config.enabled_mods {
        match install_mod(mod_name, &target, config.user_dir_path.as_deref()) {
            Ok(msg) => results.push(msg),
            Err(e) => results.push(format!("Failed to install {}: {}", mod_name, e)),
        }
    }

    Ok(results.join("\n"))
}

#[tauri::command]
fn remove_mod(mod_name: String) -> Result<(), String> {
    let mod_dir = get_mods_dir().join(&mod_name);

    if !mod_dir.exists() {
        return Err(format!("Mod not found: {}", mod_name));
    }

    // First disable it
    disable_mod(mod_name.clone())?;

    // Then remove the directory
    std::fs::remove_dir_all(&mod_dir)
        .map_err(|e| format!("Failed to remove mod directory: {}", e))?;

    Ok(())
}

#[tauri::command]
fn import_mod(
    source_path: String,
    mod_name: Option<String>,
    version: Option<String>,
    mod_type: Option<String>,
    author: Option<String>,
    description: Option<String>,
) -> Result<String, String> {
    use std::fs;

    tracing::info!("Starting mod import from: {}", source_path);
    tracing::debug!(
        "Import params - name: {:?}, version: {:?}, type: {:?}",
        mod_name,
        version,
        mod_type
    );

    let source = PathBuf::from(&source_path);
    let mods_dir = get_mods_dir();

    if !source.exists() {
        tracing::error!("Source path does not exist: {}", source_path);
        return Err("Source path does not exist".to_string());
    }

    tracing::info!(
        "Source exists: {:?}, is_file: {}, is_dir: {}",
        source,
        source.is_file(),
        source.is_dir()
    );

    // Handle different source types
    let mod_root = if source.is_file() {
        let ext = source.extension().and_then(|s| s.to_str());
        tracing::info!("File extension: {:?}", ext);

        if ext == Some("zip") {
            // Extract ZIP to temp directory
            let temp_dir =
                std::env::temp_dir().join(format!("fmmloader_import_{}", uuid::Uuid::new_v4()));
            tracing::info!("Extracting ZIP to: {:?}", temp_dir);
            extract_zip(&source, &temp_dir)?;
            let root = find_mod_root(&temp_dir)?;
            tracing::info!("Found mod root in ZIP: {:?}", root);
            root
        } else {
            // Single file (.bundle, .fmf, etc) - create temp dir with just this file
            let temp_dir =
                std::env::temp_dir().join(format!("fmmloader_import_{}", uuid::Uuid::new_v4()));
            tracing::info!("Creating temp directory for single file: {:?}", temp_dir);
            fs::create_dir_all(&temp_dir).map_err(|e| {
                tracing::error!("Failed to create temp directory: {}", e);
                format!("Failed to create temp directory: {}", e)
            })?;

            let file_name = source.file_name().ok_or("Invalid file name")?;
            let dest_file = temp_dir.join(file_name);

            tracing::info!("Copying file to: {:?}", dest_file);
            fs::copy(&source, &dest_file).map_err(|e| {
                tracing::error!("Failed to copy file: {}", e);
                format!("Failed to copy file: {}", e)
            })?;

            temp_dir
        }
    } else {
        // It's a directory
        tracing::info!("Source is a directory, finding mod root");
        let root = find_mod_root(&source)?;
        tracing::info!("Found mod root: {:?}", root);
        root
    };

    // Check if manifest exists
    let needs_manifest = !has_manifest(&mod_root);
    tracing::info!("Needs manifest: {}", needs_manifest);

    // If no manifest and no metadata provided, return error asking for metadata
    if needs_manifest {
        if mod_name.is_none() || version.is_none() || mod_type.is_none() {
            tracing::warn!("Manifest needed but metadata not provided");
            // Return special error code indicating we need metadata
            return Err("NEEDS_METADATA".to_string());
        }

        tracing::info!("Generating manifest with provided metadata");
        // Generate manifest with provided metadata
        generate_manifest(
            &mod_root,
            mod_name.clone().unwrap(),
            version.unwrap(),
            mod_type.unwrap(),
            author.unwrap_or_default(),
            description.unwrap_or_default(),
        )?;
    }

    // Read the manifest to get the mod name
    tracing::info!("Reading manifest from mod root");
    let manifest = mod_manager::read_manifest(&mod_root)?;
    let final_mod_name = mod_name.unwrap_or(manifest.name.clone());
    tracing::info!("Final mod name: {}", final_mod_name);

    // Copy to mods directory
    let dest_dir = mods_dir.join(&final_mod_name);
    tracing::info!("Destination directory: {:?}", dest_dir);

    if dest_dir.exists() {
        tracing::error!("Mod already exists: {}", final_mod_name);
        return Err(format!("Mod '{}' already exists", final_mod_name));
    }

    // Copy the mod files
    tracing::info!("Copying mod files from {:?} to {:?}", mod_root, dest_dir);
    copy_dir_recursive(&mod_root, &dest_dir)?;
    tracing::info!("Mod import completed successfully: {}", final_mod_name);

    Ok(final_mod_name)
}

#[tauri::command]
fn detect_mod_type(path: String) -> Result<String, String> {
    let mod_path = PathBuf::from(path);

    if !mod_path.exists() {
        return Err("Path does not exist".to_string());
    }

    Ok(auto_detect_mod_type(&mod_path))
}

#[tauri::command]
fn check_conflicts() -> Result<Vec<ConflictInfo>, String> {
    let config = load_config()?;

    let target_path = config.target_path.as_ref().ok_or("Game target not set")?;

    let target = PathBuf::from(target_path);

    find_conflicts(
        &config.enabled_mods,
        &target,
        config.user_dir_path.as_deref(),
    )
}

#[tauri::command]
fn get_restore_points() -> Result<Vec<RestorePoint>, String> {
    list_restore_points()
}

#[tauri::command]
fn restore_from_point(point_path: String) -> Result<String, String> {
    let path = PathBuf::from(point_path);
    rollback_to_restore_point(&path)
}

#[tauri::command]
fn create_backup_point(name: String) -> Result<String, String> {
    let config = load_config()?;

    let target_path = config.target_path.as_ref().ok_or("Game target not set")?;

    let target = PathBuf::from(target_path);
    let point_dir = create_restore_point(&name, &[target])?;

    Ok(point_dir.to_string_lossy().to_string())
}

#[tauri::command]
fn open_logs_folder() -> Result<(), String> {
    let logs_dir = logging::get_logs_dir();
    tracing::info!("Opening logs folder: {:?}", logs_dir);

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&logs_dir)
            .spawn()
            .map_err(|e| format!("Failed to open logs folder: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&logs_dir)
            .spawn()
            .map_err(|e| format!("Failed to open logs folder: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&logs_dir)
            .spawn()
            .map_err(|e| format!("Failed to open logs folder: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
fn get_logs_path() -> Result<String, String> {
    let logs_dir = logging::get_logs_dir();
    Ok(logs_dir.to_string_lossy().to_string())
}

#[tauri::command]
fn open_mods_folder() -> Result<(), String> {
    let mods_dir = get_mods_dir();
    tracing::info!("Opening mods folder: {:?}", mods_dir);

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&mods_dir)
            .spawn()
            .map_err(|e| format!("Failed to open mods folder: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&mods_dir)
            .spawn()
            .map_err(|e| format!("Failed to open mods folder: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&mods_dir)
            .spawn()
            .map_err(|e| format!("Failed to open mods folder: {}", e))?;
    }

    Ok(())
}

/// Log update-related events to backend log files with structured [UPDATE_*] prefixes.
/// This bridges frontend update checking with backend file logging infrastructure.
///
/// # Arguments
/// * `event_type` - Type of update event (CHECK, FOUND, DOWNLOAD, INSTALL, ERROR)
/// * `current_version` - Current application version
/// * `latest_version` - Latest available version (if applicable)
/// * `message` - Human-readable log message
/// * `details` - Optional additional details (error traces, release notes, etc.)
#[tauri::command]
fn log_update_event(
    event_type: String,
    current_version: String,
    latest_version: Option<String>,
    message: String,
    details: Option<String>,
) -> Result<(), String> {
    tracing::info!(
        "[UPDATE_{}] Current: {} | Latest: {} | {} | Details: {}",
        event_type,
        current_version,
        latest_version.unwrap_or_else(|| "N/A".to_string()),
        message,
        details.unwrap_or_else(|| "None".to_string())
    );
    Ok(())
}

#[tauri::command]
fn check_name_fix_installed() -> Result<bool, String> {
    let config = load_config()?;
    name_fix::check_installed(config.target_path.as_deref())
}

#[tauri::command]
fn install_name_fix() -> Result<String, String> {
    // Install the GitHub name fix (backwards compatibility)
    name_fix::install_name_fix(name_fix::GITHUB_NAME_FIX_ID.to_string())
}

#[tauri::command]
fn uninstall_name_fix() -> Result<String, String> {
    name_fix::uninstall()
}

#[tauri::command]
fn list_name_fixes() -> Result<Vec<crate::types::NameFixSource>, String> {
    name_fix::list_name_fixes()
}

#[tauri::command]
fn import_name_fix(file_path: String, name: String) -> Result<String, String> {
    name_fix::import_name_fix(file_path, name)
}

#[tauri::command]
fn install_name_fix_by_id(name_fix_id: String) -> Result<String, String> {
    name_fix::install_name_fix(name_fix_id)
}

#[tauri::command]
fn delete_name_fix(name_fix_id: String) -> Result<String, String> {
    name_fix::delete_name_fix(name_fix_id)
}

#[tauri::command]
fn get_active_name_fix() -> Result<Option<String>, String> {
    name_fix::get_active_name_fix()
}

#[tauri::command]
fn list_graphics_packs() -> Result<Vec<GraphicsPackMetadata>, String> {
    let registry = load_graphics_packs()?;
    Ok(registry.graphics_packs)
}

/// Analyzes a graphics pack (file or directory) to determine its type
#[tauri::command]
async fn analyze_graphics_pack(
    source_path: String,
) -> Result<graphics_analyzer::GraphicsPackAnalysis, String> {
    use graphics_analyzer::analyze_graphics_pack;

    tracing::info!("Analyzing graphics pack: {}", source_path);

    let source = PathBuf::from(&source_path);

    // If it's an archive, extract it to a temp directory first
    let (analysis_path, temp_dir_to_cleanup) = if source.is_file() {
        let temp_dir =
            std::env::temp_dir().join(format!("fmmloader_analysis_{}", uuid::Uuid::new_v4()));

        tracing::info!("Extracting to temp for analysis: {:?}", temp_dir);

        // Extract without progress tracking (just for analysis)
        extract_zip(&source, &temp_dir)?;

        // Find the content root
        let content_root = find_graphics_content_root(&temp_dir)?;
        (content_root, Some(temp_dir))
    } else {
        (source, None)
    };

    // Analyze the pack
    let analysis = analyze_graphics_pack(&analysis_path);

    // Clean up temp directory if it was created
    if let Some(temp_dir) = temp_dir_to_cleanup {
        if let Err(e) = std::fs::remove_dir_all(&temp_dir) {
            tracing::warn!("Failed to cleanup analysis temp directory: {}", e);
        } else {
            tracing::info!("Cleaned up analysis temp directory: {:?}", temp_dir);
        }
    }

    // Return analysis result (propagate error if analysis failed)
    let analysis = analysis?;
    tracing::info!("Analysis complete: {:?}", analysis);

    Ok(analysis)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphicsPackIssue {
    pub pack_name: String,
    pub current_path: String,
    pub suggested_path: String,
    pub reason: String,
    pub pack_type: String,
}

/// Validates existing graphics packs and identifies misplaced ones
#[tauri::command]
fn validate_graphics() -> Result<Vec<GraphicsPackIssue>, String> {
    use graphics_analyzer::analyze_graphics_pack;

    tracing::info!("Validating installed graphics packs");

    let config = load_config()?;
    let user_dir = game_detection::get_fm_user_dir(config.user_dir_path.as_deref());
    let graphics_dir = user_dir.join("graphics");

    if !graphics_dir.exists() {
        return Ok(Vec::new());
    }

    let mut issues = Vec::new();

    // Scan all subdirectories in graphics/
    for entry in std::fs::read_dir(&graphics_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let pack_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        // Skip known subdirectories
        if ["faces", "logos", "kits", "badges"].contains(&pack_name.to_lowercase().as_str()) {
            continue;
        }

        // Analyze this pack
        match analyze_graphics_pack(&path) {
            Ok(analysis) => {
                let current_location = path
                    .strip_prefix(&graphics_dir)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .to_string();

                // Check if it's in the right place
                let expected_subdir = match &analysis.pack_type {
                    graphics_analyzer::GraphicsPackType::Faces => Some("faces"),
                    graphics_analyzer::GraphicsPackType::Logos => Some("logos"),
                    graphics_analyzer::GraphicsPackType::Kits => Some("kits"),
                    _ => None,
                };

                if let Some(expected) = expected_subdir {
                    if !current_location.starts_with(expected) {
                        let suggested_path = format!("{}/{}", expected, pack_name);

                        issues.push(GraphicsPackIssue {
                            pack_name: pack_name.clone(),
                            current_path: current_location,
                            suggested_path: suggested_path.clone(),
                            reason: format!(
                                "This pack appears to be a {} pack but is not in the {} directory",
                                expected, expected
                            ),
                            pack_type: expected.to_string(),
                        });
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to analyze pack {}: {}", pack_name, e);
            }
        }
    }

    tracing::info!("Validation complete. Found {} issues", issues.len());
    Ok(issues)
}

/// Migrates a graphics pack to the correct subdirectory
#[tauri::command]
async fn migrate_graphics_pack(
    app: tauri::AppHandle,
    pack_name: String,
    target_subdir: String,
) -> Result<String, String> {
    use graphics_analyzer::analyze_graphics_pack;
    use std::fs;

    tracing::info!("Migrating pack '{}' to '{}'", pack_name, target_subdir);

    let config = load_config()?;
    let user_dir = game_detection::get_fm_user_dir(config.user_dir_path.as_deref());
    let graphics_dir = user_dir.join("graphics");

    // Find the current pack location
    let current_path = graphics_dir.join(&pack_name);

    if !current_path.exists() {
        return Err(format!(
            "Pack '{}' not found in graphics directory",
            pack_name
        ));
    }

    // Analyze the pack to detect if it's flat
    let analysis = analyze_graphics_pack(&current_path).unwrap_or_else(|_| {
        // If analysis fails, assume it's not flat (safer default)
        tracing::warn!("Failed to analyze pack, assuming structured pack");
        graphics_analyzer::GraphicsPackAnalysis {
            pack_type: graphics_analyzer::GraphicsPackType::Unknown,
            confidence: 0.0,
            suggested_paths: Vec::new(),
            file_count: 0,
            total_size_bytes: 0,
            has_config_xml: false,
            subdirectory_breakdown: std::collections::HashMap::new(),
            is_flat_pack: false,
        }
    });

    let is_flat = analysis.is_flat_pack;
    tracing::info!("Pack is flat: {}", is_flat);

    // Create target directory structure
    let target_dir = graphics_dir.join(&target_subdir);
    fs::create_dir_all(&target_dir)
        .map_err(|e| format!("Failed to create target directory: {}", e))?;

    // Create backup in app data before moving
    let app_data_dir = config::get_app_data_dir();
    let backup_dir = app_data_dir.join("graphics_migration_backup");
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let backup_path = backup_dir.join(format!("{}_{}", pack_name, timestamp));

    tracing::info!("Creating backup at: {:?}", backup_path);

    fs::create_dir_all(&backup_dir)
        .map_err(|e| format!("Failed to create backup directory: {}", e))?;

    // Copy to backup first
    copy_dir_all(&current_path, &backup_path)
        .map_err(|e| format!("Failed to create backup: {}", e))?;

    tracing::info!("Backup created, now moving to new location");

    if is_flat {
        // For flat packs, copy contents directly to target directory (e.g., graphics/faces/)
        tracing::info!(
            "Migrating flat pack - copying contents to: {:?}",
            target_dir
        );

        // Check if target directory has existing files (conflict detection)
        if target_dir.exists() {
            let existing_file_count = count_files_in_dir(&target_dir)?;
            if existing_file_count > 0 {
                tracing::warn!(
                    "Target directory {} already contains {} files. Migration may overwrite files.",
                    target_dir.display(),
                    existing_file_count
                );
                // Note: For migration from validation dialog, user has already confirmed they want to move it
                // So we proceed, but log the warning
            }
        }

        // Count total files for progress tracking
        let total_files = count_files_in_dir(&current_path)?;
        let mut current_file_count = 0;

        // Emit initial progress
        if let Some(window) = app.get_webview_window("main") {
            let progress = ExtractionProgress {
                current: 0,
                total: total_files,
                current_file: "Starting migration...".to_string(),
                bytes_processed: 0,
                phase: "migrating".to_string(),
            };
            let _ = window.emit("migration-progress", &progress);
        }

        for entry in fs::read_dir(&current_path).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let src_path = entry.path();
            let file_name = entry.file_name();
            let dst_path = target_dir.join(&file_name);

            if src_path.is_dir() {
                // Copy directory and update progress
                let dir_file_count = count_files_in_dir(&src_path)?;
                copy_dir_all(&src_path, &dst_path)
                    .map_err(|e| format!("Failed to copy directory: {}", e))?;
                current_file_count += dir_file_count;
            } else {
                fs::copy(&src_path, &dst_path)
                    .map_err(|e| format!("Failed to copy file: {}", e))?;
                current_file_count += 1;
            }

            // Emit progress every 100 files or on last file
            if current_file_count % 100 == 0 || current_file_count == total_files {
                if let Some(window) = app.get_webview_window("main") {
                    let progress = ExtractionProgress {
                        current: current_file_count,
                        total: total_files,
                        current_file: format!("Migrating {}", file_name.to_string_lossy()),
                        bytes_processed: 0,
                        phase: "migrating".to_string(),
                    };
                    let _ = window.emit("migration-progress", &progress);
                }
            }
        }

        // Remove the original pack directory
        fs::remove_dir_all(&current_path)
            .map_err(|e| format!("Failed to remove original directory: {}", e))?;

        // Emit completion
        if let Some(window) = app.get_webview_window("main") {
            let progress = ExtractionProgress {
                current: total_files,
                total: total_files,
                current_file: "Migration complete".to_string(),
                bytes_processed: 0,
                phase: "complete".to_string(),
            };
            let _ = window.emit("migration-progress", &progress);
        }

        tracing::info!("Flat pack migrated successfully");

        // Clean up backup after successful migration
        if let Err(e) = fs::remove_dir_all(&backup_path) {
            tracing::warn!("Failed to cleanup backup directory: {}", e);
        } else {
            tracing::info!("Backup cleaned up successfully");
        }

        Ok(format!(
            "Pack '{}' contents moved to {}",
            pack_name,
            target_dir.display()
        ))
    } else {
        // For structured packs, move the whole directory
        let target_path = target_dir.join(&pack_name);

        // Check if target already exists
        if target_path.exists() {
            return Err(format!(
                "Target location already exists: {}",
                target_path.display()
            ));
        }

        // Emit progress for structured pack (quick rename operation)
        if let Some(window) = app.get_webview_window("main") {
            let progress = ExtractionProgress {
                current: 0,
                total: 1,
                current_file: format!("Moving {}", pack_name),
                bytes_processed: 0,
                phase: "migrating".to_string(),
            };
            let _ = window.emit("migration-progress", &progress);
        }

        fs::rename(&current_path, &target_path)
            .map_err(|e| format!("Failed to move pack: {}", e))?;

        // Emit completion
        if let Some(window) = app.get_webview_window("main") {
            let progress = ExtractionProgress {
                current: 1,
                total: 1,
                current_file: "Migration complete".to_string(),
                bytes_processed: 0,
                phase: "complete".to_string(),
            };
            let _ = window.emit("migration-progress", &progress);
        }

        tracing::info!("Structured pack migrated successfully");

        // Clean up backup after successful migration
        if let Err(e) = fs::remove_dir_all(&backup_path) {
            tracing::warn!("Failed to cleanup backup directory: {}", e);
        } else {
            tracing::info!("Backup cleaned up successfully");
        }

        Ok(format!(
            "Pack '{}' moved to {}",
            pack_name,
            target_path.display()
        ))
    }
}

// Helper function to recursively copy directories
fn copy_dir_all(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
    use std::fs;

    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

/// Check for conflicts before installing graphics pack
#[tauri::command]
fn check_graphics_conflicts(
    target_path: String,
    pack_name: String,
    is_flat_pack: bool,
) -> Result<Option<GraphicsConflictInfo>, String> {
    let config = load_config()?;
    let user_dir = game_detection::get_fm_user_dir(config.user_dir_path.as_deref());
    let graphics_dir = user_dir.join("graphics");

    // Determine the actual install directory based on pack type
    let install_dir = if is_flat_pack {
        // For flat packs, extract just the type directory (e.g., "logos" from "logos/PackName")
        let target_parts: Vec<&str> = target_path.split('/').filter(|s| !s.is_empty()).collect();
        if target_parts.is_empty() {
            graphics_dir.join("faces") // Default
        } else {
            graphics_dir.join(target_parts[0])
        }
    } else {
        // For structured packs, use the full target path (e.g., "logos/PackName")
        graphics_dir.join(&target_path)
    };

    // Check if directory exists and has files
    if install_dir.exists() {
        let file_count = count_files_in_dir(&install_dir)?;
        if file_count > 0 {
            return Ok(Some(GraphicsConflictInfo {
                target_directory: install_dir.to_string_lossy().to_string(),
                existing_file_count: file_count,
                pack_name,
            }));
        }
    }

    Ok(None)
}

/// Async command to import graphics packs with type detection and smart routing
#[tauri::command]
async fn import_graphics_pack_with_type(
    app: tauri::AppHandle,
    source_path: String,
    target_path: String,
    should_split: bool,
    _force: bool,
) -> Result<String, String> {
    use graphics_analyzer::{analyze_graphics_pack, split_mixed_pack};

    tracing::info!(
        "Starting graphics pack import with type detection from: {}",
        source_path
    );

    let source = PathBuf::from(&source_path);

    // Validate source exists
    if !source.exists() {
        return Err("Source path does not exist".to_string());
    }

    // Check if it's an archive file
    let is_archive = source.is_file()
        && source
            .extension()
            .and_then(|s| s.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("zip"))
            .unwrap_or(false);

    if !is_archive {
        return Err("Only ZIP archives are currently supported for graphics packs".to_string());
    }

    // Check source file size and estimate extraction size
    let source_size = std::fs::metadata(&source)
        .map_err(|e| format!("Failed to read source file: {}", e))?
        .len();

    // Graphics packs typically decompress to 3-5x compressed size
    let estimated_extracted_size = source_size * 5;
    let estimated_gb = estimated_extracted_size as f64 / 1024.0 / 1024.0 / 1024.0;

    // Warn if extraction will be very large (>20GB)
    if estimated_extracted_size > 20 * 1024 * 1024 * 1024 {
        tracing::warn!(
            "Large graphics pack detected: {}MB compressed, estimated ~{:.1}GB extracted. Ensure sufficient disk space.",
            source_size / 1024 / 1024,
            estimated_gb
        );
    }

    // Create temporary extraction directory
    let temp_dir =
        std::env::temp_dir().join(format!("fmmloader_graphics_{}", uuid::Uuid::new_v4()));

    tracing::info!("Extracting to temporary directory: {:?}", temp_dir);

    // Extract with progress tracking
    let app_clone = app.clone();
    let source_clone = source.clone();
    let temp_dir_clone = temp_dir.clone();

    let extracted_dir = extract_zip_async(source_clone, temp_dir_clone, move |progress| {
        if let Some(window) = app_clone.get_webview_window("main") {
            if let Err(e) = window.emit("graphics-extraction-progress", &progress) {
                tracing::error!("Failed to emit progress event: {}", e);
            }
        }
    })
    .await?;

    tracing::info!("Extraction complete to: {:?}", extracted_dir);

    // Load config to get user directory
    let config = load_config()?;
    let user_dir = game_detection::get_fm_user_dir(config.user_dir_path.as_deref());
    let graphics_dir = user_dir.join("graphics");

    // Create graphics directory if it doesn't exist
    std::fs::create_dir_all(&graphics_dir)
        .map_err(|e| format!("Failed to create graphics directory: {}", e))?;

    // Find the actual graphics content root
    let content_root = find_graphics_content_root(&extracted_dir)?;
    tracing::info!("Found graphics content root: {:?}", content_root);

    // Analyze the pack
    let analysis = analyze_graphics_pack(&content_root)?;
    tracing::info!("Pack analysis: {:?}", analysis);

    let pack_name = source
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown Graphics Pack")
        .to_string();

    // Determine installation targets
    let final_target = PathBuf::from(&target_path);

    // Track total installed files for metadata
    let mut total_installed_files = 0;

    // Handle mixed packs if splitting is requested
    if should_split
        && matches!(
            analysis.pack_type,
            graphics_analyzer::GraphicsPackType::Mixed(_)
        )
    {
        tracing::info!("Splitting mixed pack into separate directories");

        let split_map = split_mixed_pack(&content_root, &analysis)?;

        for (pack_type, source_dir) in split_map {
            let target_dir = graphics_dir
                .join(&pack_type)
                .join(format!("{}-{}", pack_name, pack_type));

            std::fs::create_dir_all(&target_dir)
                .map_err(|e| format!("Failed to create target directory: {}", e))?;

            // Copy this portion
            let file_count = count_files_in_dir(&source_dir)?;
            copy_graphics_content(&source_dir, &target_dir, file_count, |_, _| {})?;

            total_installed_files += file_count;
            tracing::info!("Installed {} pack to: {:?}", pack_type, target_dir);
        }
    } else {
        // Single install location
        // Determine if this is a flat pack - if so, install contents directly to target directory
        let install_path = if analysis.is_flat_pack {
            // For flat packs, install directly to graphics/faces/ (or logos/kits)
            // Extract the type directory from target_path (e.g., "faces" from "faces/PackName")
            let target_parts: Vec<&str> = final_target
                .to_str()
                .unwrap_or("")
                .split('/')
                .filter(|s| !s.is_empty())
                .collect();

            // Use the first part (faces/logos/kits) which is the type directory
            if target_parts.is_empty() {
                graphics_dir.join("faces") // Default to faces if unclear
            } else {
                graphics_dir.join(target_parts[0])
            }
        } else {
            // For structured packs, preserve the pack name as a subdirectory
            graphics_dir.join(&final_target)
        };

        tracing::info!(
            "Installing pack (flat={}) to: {:?}",
            analysis.is_flat_pack,
            install_path
        );

        std::fs::create_dir_all(&install_path)
            .map_err(|e| format!("Failed to create install directory: {}", e))?;

        // Emit indexing phase progress
        if let Some(window) = app.get_webview_window("main") {
            let progress = ExtractionProgress {
                current: 0,
                total: 100,
                current_file: "Indexing files...".to_string(),
                bytes_processed: 0,
                phase: "indexing".to_string(),
            };
            let _ = window.emit("graphics-extraction-progress", &progress);
        }

        // Count files
        let file_count = count_files_in_dir(&content_root)?;
        total_installed_files = file_count;

        // Copy with progress tracking based on pack type
        let app_clone_copy = app.clone();

        // Copy files based on pack type
        // Both flat and structured packs now use the same approach:
        // Copy all contents from content_root to install_path recursively
        copy_flat_pack_content(
            &content_root,
            &install_path,
            file_count,
            move |current, current_file| {
                if let Some(window) = app_clone_copy.get_webview_window("main") {
                    let progress = ExtractionProgress {
                        current,
                        total: file_count,
                        current_file,
                        bytes_processed: 0,
                        phase: "copying".to_string(),
                    };
                    if let Err(e) = window.emit("graphics-extraction-progress", &progress) {
                        tracing::error!("Failed to emit copy progress event: {}", e);
                    }
                }
            },
        )?;

        tracing::info!("Installed pack to: {:?}", install_path);
    }

    // Register the pack in the metadata registry
    let pack_type_str = match &analysis.pack_type {
        graphics_analyzer::GraphicsPackType::Faces => "Faces",
        graphics_analyzer::GraphicsPackType::Logos => "Logos",
        graphics_analyzer::GraphicsPackType::Kits => "Kits",
        graphics_analyzer::GraphicsPackType::Mixed(_) => "Mixed",
        graphics_analyzer::GraphicsPackType::Unknown => "Unknown",
    };

    let pack_metadata = GraphicsPackMetadata {
        id: uuid::Uuid::new_v4().to_string(),
        name: pack_name.clone(),
        install_date: chrono::Utc::now().to_rfc3339(),
        file_count: total_installed_files,
        source_filename: source
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown.zip")
            .to_string(),
        pack_type: pack_type_str.to_string(),
        installed_to: final_target.to_str().unwrap_or("").to_string(),
    };

    // Load registry, add pack, and save
    let mut registry = load_graphics_packs().unwrap_or_default();
    registry.graphics_packs.push(pack_metadata);
    save_graphics_packs(&registry)?;

    tracing::info!("Registered graphics pack in metadata registry");

    // Emit completion event
    if let Some(window) = app.get_webview_window("main") {
        let completion = ExtractionProgress {
            current: 100,
            total: 100,
            current_file: "Installation complete".to_string(),
            bytes_processed: 0,
            phase: "complete".to_string(),
        };
        let _ = window.emit("graphics-extraction-progress", &completion);
    }

    // Cleanup temp directory
    if let Err(e) = std::fs::remove_dir_all(&temp_dir) {
        tracing::warn!("Failed to cleanup temp directory: {}", e);
    }

    tracing::info!("Graphics pack imported successfully");
    Ok("Graphics pack installed successfully".to_string())
}

fn count_files_in_dir(dir: &PathBuf) -> Result<usize, String> {
    Ok(walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .count())
}

/// Legacy import function (kept for backwards compatibility)
#[tauri::command]
async fn import_graphics_pack(
    app: tauri::AppHandle,
    source_path: String,
) -> Result<String, String> {
    // Delegate to new function with default behavior (no splitting, auto-detect path, no force)
    import_graphics_pack_with_type(app, source_path, "graphics".to_string(), false, false).await
}

// Helper function to find the actual graphics content root
// Skips wrapper folders and finds where faces/, logos/, kits/ actually live
fn find_graphics_content_root(extracted_dir: &PathBuf) -> Result<PathBuf, String> {
    use std::fs;

    // Graphics subdirectory names to look for
    let _graphics_dirs = ["faces", "logos", "kits", "badges", "graphics"];

    // Check if current directory contains any graphics subdirectories
    fn has_graphics_subdirs(path: &PathBuf) -> bool {
        let graphics_dirs = ["faces", "logos", "kits", "badges"];
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        let name_lower = name.to_lowercase();
                        if graphics_dirs.iter().any(|&gd| name_lower.contains(gd)) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    // If extracted_dir itself has graphics subdirs, it's the content root
    if has_graphics_subdirs(extracted_dir) {
        return Ok(extracted_dir.clone());
    }

    // Otherwise, search one or two levels deep for the content root
    if let Ok(entries) = fs::read_dir(extracted_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Check if this subdirectory has graphics subdirs
                if has_graphics_subdirs(&path) {
                    return Ok(path);
                }

                // Check one more level deep (for deeply nested structures)
                if let Ok(sub_entries) = fs::read_dir(&path) {
                    for sub_entry in sub_entries.flatten() {
                        let sub_path = sub_entry.path();
                        if sub_path.is_dir() && has_graphics_subdirs(&sub_path) {
                            return Ok(sub_path);
                        }
                    }
                }
            }
        }
    }

    // If we didn't find graphics subdirs, just return the extracted dir
    Ok(extracted_dir.clone())
}

// Helper function to detect graphics pack type
#[allow(dead_code)]
fn detect_graphics_pack_type(path: &PathBuf) -> String {
    use std::fs;

    let mut has_faces = false;
    let mut has_logos = false;
    let mut has_kits = false;

    // Check subdirectories
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    let name_lower = name.to_lowercase();
                    if name_lower.contains("face") {
                        has_faces = true;
                    }
                    if name_lower.contains("logo") || name_lower.contains("badge") {
                        has_logos = true;
                    }
                    if name_lower.contains("kit") {
                        has_kits = true;
                    }
                }
            }
        }
    }

    // Determine pack type
    let types_found = [has_faces, has_logos, has_kits]
        .iter()
        .filter(|&&x| x)
        .count();

    if types_found > 1 {
        "mixed".to_string()
    } else if has_faces {
        "faces".to_string()
    } else if has_logos {
        "logos".to_string()
    } else if has_kits {
        "kits".to_string()
    } else {
        "graphics".to_string()
    }
}

// Helper function to copy graphics content, preserving subdirectories
// Copies each graphics subdirectory (faces/, logos/, kits/) to the FM graphics directory
fn copy_graphics_content<F>(
    content_root: &Path,
    graphics_dir: &Path,
    total_files: usize,
    mut progress_callback: F,
) -> Result<(), String>
where
    F: FnMut(usize, String),
{
    use std::fs;
    use walkdir::WalkDir;

    // Graphics subdirectory names to look for
    let graphics_subdirs = ["faces", "logos", "kits", "badges", "graphics"];
    let mut files_copied = 0;

    // Find and copy each graphics subdirectory
    if let Ok(entries) = fs::read_dir(content_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    let name_lower = name.to_lowercase();

                    // Check if this is a graphics subdirectory
                    if graphics_subdirs.iter().any(|&gd| name_lower.contains(gd)) {
                        // This is a graphics subdirectory, copy it preserving structure
                        let dest_subdir = graphics_dir.join(name);
                        tracing::info!(
                            "Copying graphics subdirectory: {} -> {:?}",
                            name,
                            dest_subdir
                        );

                        // Create destination subdirectory if it doesn't exist
                        fs::create_dir_all(&dest_subdir).map_err(|e| {
                            format!("Failed to create destination subdirectory: {}", e)
                        })?;

                        // Copy all contents recursively
                        for walk_entry in WalkDir::new(&path) {
                            let walk_entry = walk_entry
                                .map_err(|e| format!("Failed to walk directory: {}", e))?;
                            let entry_path = walk_entry.path();

                            if let Ok(rel_path) = entry_path.strip_prefix(&path) {
                                let target_path = dest_subdir.join(rel_path);

                                if entry_path.is_dir() {
                                    // Create all subdirectories (including nested ones like logos/Clubs, logos/Nations, etc.)
                                    fs::create_dir_all(&target_path).map_err(|e| {
                                        format!("Failed to create directory: {}", e)
                                    })?;
                                    tracing::debug!("Created directory: {:?}", target_path);
                                } else {
                                    if let Some(parent) = target_path.parent() {
                                        fs::create_dir_all(parent).map_err(|e| {
                                            format!("Failed to create parent directory: {}", e)
                                        })?;
                                    }

                                    // Log config.xml files specifically for verification
                                    if entry_path.file_name().and_then(|n| n.to_str())
                                        == Some("config.xml")
                                    {
                                        tracing::info!(
                                            "Copying config.xml: {:?} -> {:?}",
                                            entry_path,
                                            target_path
                                        );
                                    }

                                    fs::copy(entry_path, &target_path)
                                        .map_err(|e| format!("Failed to copy file: {}", e))?;

                                    files_copied += 1;

                                    // Emit progress every 50 files or on completion
                                    if files_copied % 50 == 0 || files_copied == total_files {
                                        let current_file_name = entry_path
                                            .file_name()
                                            .and_then(|n| n.to_str())
                                            .unwrap_or("")
                                            .to_string();
                                        progress_callback(files_copied, current_file_name);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Emit final progress update
    if files_copied > 0 {
        progress_callback(files_copied, "Complete".to_string());
    }

    Ok(())
}

/// Copy flat pack contents directly to destination (for packs with PNGs/config.xml at root)
fn copy_flat_pack_content<F>(
    content_root: &Path,
    install_dir: &Path,
    total_files: usize,
    mut progress_callback: F,
) -> Result<(), String>
where
    F: FnMut(usize, String),
{
    use std::fs;
    use walkdir::WalkDir;

    let mut files_copied = 0;

    // Copy all files from content_root directly to install_dir
    for entry in WalkDir::new(content_root) {
        let entry = entry.map_err(|e| format!("Failed to walk directory: {}", e))?;
        let entry_path = entry.path();

        if let Ok(rel_path) = entry_path.strip_prefix(content_root) {
            let target_path = install_dir.join(rel_path);

            if entry_path.is_dir() {
                // Create directories
                fs::create_dir_all(&target_path)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
                tracing::debug!("Created directory: {:?}", target_path);
            } else {
                // Copy files
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create parent directory: {}", e))?;
                }

                // Log config.xml files specifically for verification
                if entry_path.file_name().and_then(|n| n.to_str()) == Some("config.xml") {
                    tracing::info!("Copying config.xml: {:?} -> {:?}", entry_path, target_path);
                }

                fs::copy(entry_path, &target_path)
                    .map_err(|e| format!("Failed to copy file: {}", e))?;

                files_copied += 1;

                // Emit progress every 50 files or on completion
                if files_copied % 50 == 0 || files_copied == total_files {
                    let current_file_name = entry_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();
                    progress_callback(files_copied, current_file_name);
                }
            }
        }
    }

    // Emit final progress update
    if files_copied > 0 {
        progress_callback(files_copied, "Complete".to_string());
    }

    Ok(())
}

// Helper function for recursive directory copy
fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> Result<(), String> {
    use std::fs;
    use walkdir::WalkDir;

    fs::create_dir_all(dst)
        .map_err(|e| format!("Failed to create destination directory: {}", e))?;

    for entry in WalkDir::new(src) {
        let entry = entry.map_err(|e| format!("Failed to walk directory: {}", e))?;
        let path = entry.path();

        if let Ok(rel_path) = path.strip_prefix(src) {
            let target_path = dst.join(rel_path);

            if path.is_dir() {
                fs::create_dir_all(&target_path)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            } else {
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create parent directory: {}", e))?;
                }
                fs::copy(path, &target_path).map_err(|e| format!("Failed to copy file: {}", e))?;
            }
        }
    }

    Ok(())
}

fn main() {
    // Initialize logging first
    if let Err(e) = logging::init_logging() {
        eprintln!("Failed to initialize logging: {}", e);
    }

    tracing::info!("Starting FMMLoader26");

    let app_version = env!("CARGO_PKG_VERSION");
    tracing::info!("Application version: {}", app_version);
    tracing::info!("Updater endpoint: https://github.com/justinlevinedotme/FMMLoader-26/releases/latest/download/latest.json");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            init_app,
            get_app_version,
            get_config,
            update_config,
            detect_game_path,
            set_game_target,
            detect_user_dir,
            get_mods_list,
            get_mod_details,
            enable_mod,
            disable_mod,
            apply_mods,
            remove_mod,
            import_mod,
            detect_mod_type,
            check_conflicts,
            get_restore_points,
            restore_from_point,
            create_backup_point,
            open_logs_folder,
            open_mods_folder,
            get_logs_path,
            log_update_event,
            check_name_fix_installed,
            install_name_fix,
            uninstall_name_fix,
            list_name_fixes,
            import_name_fix,
            install_name_fix_by_id,
            delete_name_fix,
            get_active_name_fix,
            import_graphics_pack,
            import_graphics_pack_with_type,
            list_graphics_packs,
            analyze_graphics_pack,
            validate_graphics,
            migrate_graphics_pack,
            check_graphics_conflicts,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
