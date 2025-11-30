//! Graphics pack management - import, analysis, validation, and migration.

use crate::config::{get_app_data_dir, load_config, load_graphics_packs, save_graphics_packs};
use crate::game_detection;
use crate::graphics_analyzer::{self, analyze_graphics_pack, split_mixed_pack};
use crate::import::{extract_zip, extract_zip_async};
use crate::types::{ExtractionProgress, GraphicsConflictInfo, GraphicsPackMetadata};
use crate::utils;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{Emitter, Manager};
use walkdir::WalkDir;

/// Issue found during graphics pack validation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphicsPackIssue {
    pub pack_name: String,
    pub current_path: String,
    pub suggested_path: String,
    pub reason: String,
    pub pack_type: String,
}

/// List all installed graphics packs
#[tauri::command]
pub fn list_graphics_packs() -> Result<Vec<GraphicsPackMetadata>, String> {
    let registry = load_graphics_packs()?;
    Ok(registry.graphics_packs)
}

/// Analyzes a graphics pack (file or directory) to determine its type
#[tauri::command]
pub async fn analyze_graphics_pack_cmd(
    source_path: String,
) -> Result<graphics_analyzer::GraphicsPackAnalysis, String> {
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
        let content_root = utils::find_graphics_content_root(&temp_dir)?;
        (content_root, Some(temp_dir))
    } else {
        (source, None)
    };

    // Analyze the pack
    let analysis = analyze_graphics_pack(&analysis_path);

    // Clean up temp directory if it was created
    if let Some(temp_dir) = temp_dir_to_cleanup {
        if let Err(e) = fs::remove_dir_all(&temp_dir) {
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

/// Validates existing graphics packs and identifies misplaced ones
#[tauri::command]
pub fn validate_graphics() -> Result<Vec<GraphicsPackIssue>, String> {
    tracing::info!("Validating installed graphics packs");

    let config = load_config()?;
    let user_dir = game_detection::get_fm_user_dir(config.user_dir_path.as_deref());
    let graphics_dir = user_dir.join("graphics");

    if !graphics_dir.exists() {
        return Ok(Vec::new());
    }

    let mut issues = Vec::new();

    // Scan all subdirectories in graphics/
    for entry in fs::read_dir(&graphics_dir).map_err(|e| e.to_string())? {
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

/// Adds a prefix to all PNG files in the provided directory (non-recursive).
/// Useful for quickly migrating face packs from `123.png` to `face_123.png`.
#[tauri::command]
pub fn prefix_graphics_files(
    directory: String,
    prefix: String,
    rename_files: Option<bool>,
    update_config: Option<bool>,
) -> Result<usize, String> {
    if prefix.is_empty() {
        return Err("Prefix cannot be empty".to_string());
    }

    let do_rename = rename_files.unwrap_or(true);
    let do_config = update_config.unwrap_or(true);

    if !do_rename && !do_config {
        return Err("Nothing to do: enable file renaming and/or config updates".to_string());
    }

    let dir_path = PathBuf::from(&directory);
    if !dir_path.exists() || !dir_path.is_dir() {
        return Err("Provided path is not a directory".to_string());
    }

    tracing::info!(
        "Prefixing graphics files (recursive) in {:?} with '{}' (rename_files={}, update_config={})",
        dir_path,
        prefix,
        do_rename,
        do_config
    );

    let mut files_to_rename: Vec<(PathBuf, PathBuf)> = Vec::new();
    let mut config_files: Vec<PathBuf> = Vec::new();
    let mut seen_targets = std::collections::HashSet::new();

    for entry in WalkDir::new(&dir_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path().to_path_buf();
        if !path.is_file() {
            continue;
        }

        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let extension_lower = extension.to_ascii_lowercase();

        if do_config && extension_lower == "xml" {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.eq_ignore_ascii_case("config.xml") {
                    config_files.push(path.clone());
                }
            }
        }

        if !do_rename || extension_lower != "png" {
            continue;
        }

        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => continue,
        };

        if file_name.starts_with(&prefix) {
            continue;
        }

        let parent = path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| dir_path.clone());
        let target_path = parent.join(format!("{}{}", prefix, file_name));

        if target_path.exists() {
            return Err(format!(
                "Target file already exists and would conflict: {}",
                target_path.display()
            ));
        }

        if !seen_targets.insert(target_path.clone()) {
            return Err(format!(
                "Duplicate target filename detected: {}",
                target_path.display()
            ));
        }

        files_to_rename.push((path, target_path));
    }

    if do_rename {
        for (source, target) in &files_to_rename {
            let file_name = source
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown.png");
            fs::rename(source, target)
                .map_err(|e| format!("Failed to rename {}: {}", file_name, e))?;
        }
    }

    tracing::info!(
        "Prefixed {} file(s) in {:?}",
        files_to_rename.len(),
        dir_path
    );

    if do_config && !config_files.is_empty() {
        // Match from="..."; we skip if already prefixed.
        let from_regex = regex::Regex::new("from=\"([^\"]+)\"")
            .map_err(|e| format!("Failed to build regex: {e}"))?;

        for config_path in config_files {
            let contents = fs::read_to_string(&config_path)
                .map_err(|e| format!("Failed to read {}: {}", config_path.display(), e))?;

            let replaced = from_regex.replace_all(&contents, |caps: &regex::Captures| {
                let current = &caps[1];
                if current.starts_with(&prefix) {
                    format!("from=\"{}\"", current)
                } else {
                    format!("from=\"{}{}\"", prefix, current)
                }
            });

            if replaced != contents {
                fs::write(&config_path, replaced.as_ref()).map_err(|e| {
                    format!(
                        "Failed to write updated config.xml at {}: {}",
                        config_path.display(),
                        e
                    )
                })?;
                tracing::info!("Updated config.xml prefixes at {}", config_path.display());
            }
        }
    }

    Ok(files_to_rename.len())
}

/// Migrates a graphics pack to the correct subdirectory
#[tauri::command]
pub async fn migrate_graphics_pack(
    app: tauri::AppHandle,
    pack_name: String,
    target_subdir: String,
) -> Result<String, String> {
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
    let app_data_dir = get_app_data_dir();
    let backup_dir = app_data_dir.join("graphics_migration_backup");
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let backup_path = backup_dir.join(format!("{}_{}", pack_name, timestamp));

    tracing::info!("Creating backup at: {:?}", backup_path);

    fs::create_dir_all(&backup_dir)
        .map_err(|e| format!("Failed to create backup directory: {}", e))?;

    // Copy to backup first
    utils::copy_dir_recursive(&current_path, &backup_path)?;

    tracing::info!("Backup created, now moving to new location");

    if is_flat {
        // For flat packs, copy contents directly to target directory (e.g., graphics/faces/)
        tracing::info!(
            "Migrating flat pack - copying contents to: {:?}",
            target_dir
        );

        // Check if target directory has existing files (conflict detection)
        if target_dir.exists() {
            let existing_file_count = utils::count_files_in_dir(&target_dir)?;
            if existing_file_count > 0 {
                tracing::warn!(
                    "Target directory {} already contains {} files. Migration may overwrite files.",
                    target_dir.display(),
                    existing_file_count
                );
            }
        }

        // Count total files for progress tracking
        let total_files = utils::count_files_in_dir(&current_path)?;
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
                let dir_file_count = utils::count_files_in_dir(&src_path)?;
                utils::copy_dir_recursive(&src_path, &dst_path)?;
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

/// Check for conflicts before installing graphics pack
#[tauri::command]
pub fn check_graphics_conflicts(
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
        let file_count = utils::count_files_in_dir(&install_dir)?;
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
pub async fn import_graphics_pack_with_type(
    app: tauri::AppHandle,
    source_path: String,
    target_path: String,
    should_split: bool,
    _force: bool,
) -> Result<String, String> {
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
    let source_size = fs::metadata(&source)
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
    fs::create_dir_all(&graphics_dir)
        .map_err(|e| format!("Failed to create graphics directory: {}", e))?;

    // Find the actual graphics content root
    let content_root = utils::find_graphics_content_root(&extracted_dir)?;
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

            fs::create_dir_all(&target_dir)
                .map_err(|e| format!("Failed to create target directory: {}", e))?;

            // Copy this portion
            let file_count = utils::count_files_in_dir(&source_dir)?;
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

        fs::create_dir_all(&install_path)
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
        let file_count = utils::count_files_in_dir(&content_root)?;
        total_installed_files = file_count;

        // Copy with progress tracking based on pack type
        let app_clone_copy = app.clone();

        // Copy files based on pack type
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
    if let Err(e) = fs::remove_dir_all(&temp_dir) {
        tracing::warn!("Failed to cleanup temp directory: {}", e);
    }

    tracing::info!("Graphics pack imported successfully");
    Ok("Graphics pack installed successfully".to_string())
}

/// Legacy import function (kept for backwards compatibility)
#[tauri::command]
pub async fn import_graphics_pack(
    app: tauri::AppHandle,
    source_path: String,
) -> Result<String, String> {
    // Delegate to new function with default behavior (no splitting, auto-detect path, no force)
    import_graphics_pack_with_type(app, source_path, "graphics".to_string(), false, false).await
}

// Helper function to copy graphics content, preserving subdirectories
fn copy_graphics_content<F>(
    content_root: &Path,
    graphics_dir: &Path,
    total_files: usize,
    mut progress_callback: F,
) -> Result<(), String>
where
    F: FnMut(usize, String),
{
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
    let mut files_copied = 0;

    // Copy all files from content_root directly to install_dir
    for entry in WalkDir::new(content_root) {
        let entry = entry.map_err(|e| format!("Failed to walk directory: {}", e))?;
        let entry_path = entry.path();

        if let Ok(rel_path) = entry_path.strip_prefix(content_root) {
            let target_path = install_dir.join(rel_path);

            if entry_path.is_dir() {
                fs::create_dir_all(&target_path)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
                tracing::debug!("Created directory: {:?}", target_path);
            } else {
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create parent directory: {}", e))?;
                }

                if entry_path.file_name().and_then(|n| n.to_str()) == Some("config.xml") {
                    tracing::info!("Copying config.xml: {:?} -> {:?}", entry_path, target_path);
                }

                fs::copy(entry_path, &target_path)
                    .map_err(|e| format!("Failed to copy file: {}", e))?;

                files_copied += 1;

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

    if files_copied > 0 {
        progress_callback(files_copied, "Complete".to_string());
    }

    Ok(())
}
