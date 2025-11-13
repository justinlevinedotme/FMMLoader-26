use crate::config::{load_config, save_config, get_app_data_dir, get_name_fixes_dir};
use crate::types::{NameFixSource, NameFixSourceType, NameFixInstallType};
use std::fs;
use std::path::{Path, PathBuf};
use reqwest::blocking::Client;
use zip::ZipArchive;
use std::io::{Read, Write};

const NAME_FIX_RELEASE_URL: &str = "https://github.com/jo13310/NameFixFM26/archive/refs/tags/v1.0.zip";
const NAME_FIX_FILE: &str = "FM26-open-names.lnc";
pub const GITHUB_NAME_FIX_ID: &str = "github-namefix";

// Files to delete as part of the installation
const FILES_TO_DELETE: &[(&str, &[&str])] = &[
    // From lnc/all/
    ("lnc/all", &[
        "ac milan (wom).lnc",
        "acc Inter unlic 26.lnc",
        "acmilan unlic 26.lnc",
        "fake.lnc",
        "inter (wom).lnc",
        "inter unlic 26.lnc",
        "japanese names.lnc",
        "lazio (wom).lnc",
        "lic_dan_swe_fra.lnc",
        "licensing club name_fm26.lnc",
        "men lazio.lnc",
        "men.atalanta.lnc",
    ]),
    // From edt/permanent/
    ("edt/permanent", &["fake.edt"]),
    // From dbc/permanent/
    ("dbc/permanent", &[
        "1_japan_removed_clubs.dbc",
        "brazil_kits.dbc",
        "england.dbc",
        "forbidden names.dbc",
        "france.dbc",
        "japan_fake.dbc",
        "japan_unlicensed_random_names.dbc",
        "j league non player.dbc",
        "licensing_post_data_lock.dbc",
        "licensing2.dbc",
        "licensing26.dbc",
        "netherlands.dbc",
    ]),
    // From dbc/language/
    ("dbc/language", &[
        "Licensing2.dbc",
        "Licensing2_chn.dbc",
    ]),
];

/// Get the FM26 database directory based on game installation path (target_path)
///
/// The database directory structure differs by platform:
/// - Windows: <game_root>/shared/data/database/db/2600/
/// - macOS: <game_root>/fm.app/Contents/PlugIns/game_plugin.bundle/Contents/Resources/shared/data/database/db/2600/
/// - Linux: <game_root>/shared/data/database/db/2600/
fn get_db_dir(target_path: Option<&str>) -> Result<PathBuf, String> {
    let target_path = target_path.ok_or(
        "Game target path not set. Please detect or set your FM26 game directory first."
    )?;

    let game_target = PathBuf::from(target_path);

    if !game_target.exists() {
        return Err(format!(
            "Game target path does not exist: {}",
            game_target.display()
        ));
    }

    // The target_path points to StreamingAssets (e.g., fm_Data/StreamingAssets/aa/StandaloneWindows64)
    // We need to navigate to the database directory from there

    #[cfg(target_os = "windows")]
    {
        // From: Football Manager 26/fm_Data/StreamingAssets/aa/StandaloneWindows64
        // To:   Football Manager 26/shared/data/database/db/2600
        // Navigate up to game root, then to shared/data/database/db/2600

        let game_root = game_target
            .parent() // aa
            .and_then(|p| p.parent()) // StreamingAssets
            .and_then(|p| p.parent()) // fm_Data or data
            .and_then(|p| p.parent()) // Football Manager 26
            .ok_or("Could not determine game root directory")?;

        let db_dir = game_root
            .join("shared")
            .join("data")
            .join("database")
            .join("db")
            .join("2600");

        if !db_dir.exists() {
            return Err(format!(
                "FM26 database directory not found at: {}. Please ensure FM26 is installed and you've launched it at least once.",
                db_dir.display()
            ));
        }

        Ok(db_dir)
    }

    #[cfg(target_os = "macos")]
    {
        // From: Football Manager 26/fm.app/Contents/Resources/Data/StreamingAssets/aa/StandaloneOSX
        // To:   Football Manager 26/fm.app/Contents/PlugIns/game_plugin.bundle/Contents/Resources/shared/data/database/db/2600

        // Navigate up to fm.app/Contents
        let fm_app_contents = game_target
            .parent() // aa
            .and_then(|p| p.parent()) // StreamingAssets
            .and_then(|p| p.parent()) // Data
            .and_then(|p| p.parent()) // Resources
            .and_then(|p| p.parent()) // Contents
            .ok_or("Could not determine fm.app/Contents directory")?;

        let db_dir = fm_app_contents
            .join("PlugIns")
            .join("game_plugin.bundle")
            .join("Contents")
            .join("Resources")
            .join("shared")
            .join("data")
            .join("database")
            .join("db")
            .join("2600");

        if !db_dir.exists() {
            return Err(format!(
                "FM26 database directory not found at: {}. Please ensure FM26 is installed and you've launched it at least once.",
                db_dir.display()
            ));
        }

        Ok(db_dir)
    }

    #[cfg(target_os = "linux")]
    {
        // From: Football Manager 26/fm_Data/StreamingAssets/aa/StandaloneLinux64
        // To:   Football Manager 26/shared/data/database/db/2600

        let game_root = game_target
            .parent() // aa
            .and_then(|p| p.parent()) // StreamingAssets
            .and_then(|p| p.parent()) // fm_Data or data
            .and_then(|p| p.parent()) // Football Manager 26
            .ok_or("Could not determine game root directory")?;

        let db_dir = game_root
            .join("shared")
            .join("data")
            .join("database")
            .join("db")
            .join("2600");

        if !db_dir.exists() {
            return Err(format!(
                "FM26 database directory not found at: {}. Please ensure FM26 is installed and you've launched it at least once.",
                db_dir.display()
            ));
        }

        Ok(db_dir)
    }
}

/// Check if FM Name Fix is installed
/// Returns true if there's an active name fix in the config
pub fn check_installed(target_path: Option<&str>) -> Result<bool, String> {
    // Check if there's an active name fix in the config
    let config = load_config()?;
    Ok(config.active_name_fix.is_some())
}

/// Download the FM Name Fix archive from GitHub
fn download_name_fix() -> Result<Vec<u8>, String> {
    tracing::info!("Downloading FM Name Fix from {}", NAME_FIX_RELEASE_URL);

    let client = Client::builder()
        .user_agent("FMMLoader26")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(NAME_FIX_RELEASE_URL)
        .send()
        .map_err(|e| format!("Failed to download FM Name Fix: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Failed to download FM Name Fix: HTTP {}", response.status()));
    }

    let bytes = response
        .bytes()
        .map_err(|e| format!("Failed to read download data: {}", e))?;

    tracing::info!("Downloaded {} bytes", bytes.len());
    Ok(bytes.to_vec())
}

/// Extract the FM26-open-names.lnc file from the zip archive
fn extract_lnc_file(zip_data: &[u8]) -> Result<Vec<u8>, String> {
    let cursor = std::io::Cursor::new(zip_data);
    let mut archive = ZipArchive::new(cursor)
        .map_err(|e| format!("Failed to read zip archive: {}", e))?;

    // Look for the .lnc file in the archive
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("Failed to read zip entry: {}", e))?;

        if file.name().ends_with(NAME_FIX_FILE) {
            let mut contents = Vec::new();
            file.read_to_end(&mut contents)
                .map_err(|e| format!("Failed to read .lnc file from archive: {}", e))?;

            tracing::info!("Extracted {} ({} bytes)", file.name(), contents.len());
            return Ok(contents);
        }
    }

    Err("FM26-open-names.lnc not found in downloaded archive".to_string())
}

/// Create backups of files that will be modified or deleted
fn create_backups(db_dir: &Path) -> Result<(), String> {
    let app_data_dir = get_app_data_dir();
    let backup_dir = app_data_dir.join("name_fix_backup");

    // Clean up old backup if it exists
    if backup_dir.exists() {
        tracing::info!("Removing old backup at {:?}", backup_dir);
        fs::remove_dir_all(&backup_dir)
            .map_err(|e| format!("Failed to remove old backup: {}", e))?;
    }

    fs::create_dir_all(&backup_dir)
        .map_err(|e| format!("Failed to create backup directory: {}", e))?;

    tracing::info!("Creating backups at {:?}", backup_dir);

    // Backup files that will be deleted
    for (subdir, files) in FILES_TO_DELETE {
        let source_dir = db_dir.join(subdir);
        let backup_subdir = backup_dir.join(subdir);

        fs::create_dir_all(&backup_subdir)
            .map_err(|e| format!("Failed to create backup subdirectory: {}", e))?;

        for file in *files {
            let source_file = source_dir.join(file);
            if source_file.exists() {
                let backup_file = backup_subdir.join(file);
                fs::copy(&source_file, &backup_file)
                    .map_err(|e| format!("Failed to backup {}: {}", file, e))?;
                tracing::debug!("Backed up {}", file);
            }
        }
    }

    tracing::info!("Backups created successfully");
    Ok(())
}

/// Create backups for folder-based name fixes (Sortitoutsi style)
/// Backs up entire dbc, edt, lnc folders
fn create_folder_backups(db_dir: &Path) -> Result<(), String> {
    let app_data_dir = get_app_data_dir();
    let backup_dir = app_data_dir.join("name_fix_backup");

    // Clean up old backup if it exists
    if backup_dir.exists() {
        tracing::info!("Removing old backup at {:?}", backup_dir);
        fs::remove_dir_all(&backup_dir)
            .map_err(|e| format!("Failed to remove old backup: {}", e))?;
    }

    fs::create_dir_all(&backup_dir)
        .map_err(|e| format!("Failed to create backup directory: {}", e))?;

    tracing::info!("Creating folder backups at {:?}", backup_dir);

    // Backup entire dbc, edt, lnc folders
    for folder_name in &["dbc", "edt", "lnc"] {
        let source_folder = db_dir.join(folder_name);
        if source_folder.exists() {
            let backup_folder = backup_dir.join(folder_name);
            tracing::info!("Backing up {} folder: {:?} -> {:?}", folder_name, source_folder, backup_folder);
            copy_dir_recursive(&source_folder, &backup_folder)?;
        } else {
            tracing::warn!("{} folder does not exist, skipping backup", folder_name);
        }
    }

    tracing::info!("Folder backups created successfully");
    Ok(())
}

/// Restore files from backup
fn restore_from_backup(db_dir: &Path) -> Result<(), String> {
    let app_data_dir = get_app_data_dir();
    let backup_dir = app_data_dir.join("name_fix_backup");

    if !backup_dir.exists() {
        return Err("No backup found. Cannot uninstall FM Name Fix.".to_string());
    }

    tracing::info!("Restoring from backup at {:?}", backup_dir);

    // Get the active name fix to determine restore type
    let config = load_config()?;
    if let Some(active_fix_id) = config.active_name_fix {
        let name_fixes_dir = get_name_fixes_dir();
        let fix_dir = name_fixes_dir.join(&active_fix_id);
        
        if fix_dir.exists() {
            let metadata_file = fix_dir.join("metadata.json");
            if let Ok(metadata_str) = fs::read_to_string(&metadata_file) {
                if let Ok(source) = serde_json::from_str::<NameFixSource>(&metadata_str) {
                    match source.install_type {
                        NameFixInstallType::Files => restore_files_backup(db_dir, &backup_dir, &fix_dir)?,
                        NameFixInstallType::Folders => restore_folders_backup(db_dir, &backup_dir)?,
                    }
                } else {
                    // Fallback to files type if can't read metadata
                    restore_files_backup(db_dir, &backup_dir, &fix_dir)?;
                }
            } else {
                // Fallback to files type if can't read metadata
                restore_files_backup(db_dir, &backup_dir, &fix_dir)?;
            }
        } else {
            tracing::warn!("Active name fix directory not found, assuming files type");
            // Can't determine type, try files restore
            restore_files_backup_without_fix_dir(db_dir, &backup_dir)?;
        }
    }

    // Remove backup directory
    fs::remove_dir_all(&backup_dir)
        .map_err(|e| format!("Failed to remove backup directory: {}", e))?;

    tracing::info!("Restore completed successfully");
    Ok(())
}

/// Restore file-based name fix
fn restore_files_backup(db_dir: &Path, backup_dir: &Path, fix_dir: &Path) -> Result<(), String> {
    let mut restored_count = 0;
    
    // Restore backed up files
    for (subdir, files) in FILES_TO_DELETE {
        let dest_dir = db_dir.join(subdir);
        let backup_subdir = backup_dir.join(subdir);

        for file in *files {
            let backup_file = backup_subdir.join(file);
            if backup_file.exists() {
                let dest_file = dest_dir.join(file);
                fs::copy(&backup_file, &dest_file)
                    .map_err(|e| format!("Failed to restore {}: {}", file, e))?;
                restored_count += 1;
                tracing::info!("Restored licensing file: {}", file);
            } else {
                tracing::debug!("Backup file not found (was not present during backup): {}", file);
            }
        }
    }

    tracing::info!("Restored {} licensing files", restored_count);

    // Remove installed name fix files based on what was imported
    let mut removed_count = 0;
    
    if let Ok(entries) = fs::read_dir(fix_dir) {
        for entry in entries.flatten() {
            let source_file = entry.path();
            if !source_file.is_file() {
                continue;
            }
            
            let filename = source_file.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            
            // Skip metadata.json
            if filename == "metadata.json" {
                continue;
            }
            
            // Determine where this file was installed and remove it
            let installed_path = if filename.ends_with(".lnc") {
                db_dir.join("lnc").join("all").join(filename)
            } else if filename.ends_with(".edt") {
                db_dir.join("edt").join("permanent").join(filename)
            } else if filename.ends_with(".dbc") {
                if filename.to_lowercase().contains("_chn") || 
                   (filename.to_lowercase().contains("licensing") && !filename.contains("_post_")) {
                    db_dir.join("dbc").join("language").join(filename)
                } else {
                    db_dir.join("dbc").join("permanent").join(filename)
                }
            } else {
                continue;
            };
            
            if installed_path.exists() {
                if let Err(e) = fs::remove_file(&installed_path) {
                    tracing::warn!("Failed to remove {}: {}", filename, e);
                } else {
                    removed_count += 1;
                    tracing::info!("Removed installed name fix file: {}", filename);
                }
            }
        }
    }
    
    tracing::info!("Removed {} installed name fix files", removed_count);
    Ok(())
}

/// Restore file-based name fix without access to fix_dir
fn restore_files_backup_without_fix_dir(db_dir: &Path, backup_dir: &Path) -> Result<(), String> {
    let mut restored_count = 0;
    
    // Restore backed up files
    for (subdir, files) in FILES_TO_DELETE {
        let dest_dir = db_dir.join(subdir);
        let backup_subdir = backup_dir.join(subdir);

        for file in *files {
            let backup_file = backup_subdir.join(file);
            if backup_file.exists() {
                let dest_file = dest_dir.join(file);
                fs::copy(&backup_file, &dest_file)
                    .map_err(|e| format!("Failed to restore {}: {}", file, e))?;
                restored_count += 1;
                tracing::info!("Restored licensing file: {}", file);
            }
        }
    }

    tracing::info!("Restored {} licensing files", restored_count);
    Ok(())
}

/// Restore folder-based name fix
fn restore_folders_backup(db_dir: &Path, backup_dir: &Path) -> Result<(), String> {
    tracing::info!("Restoring folder-based name fix");
    
    // Delete current dbc, edt, lnc folders
    for folder_name in &["dbc", "edt", "lnc"] {
        let folder_path = db_dir.join(folder_name);
        if folder_path.exists() {
            tracing::info!("Deleting current {} folder", folder_name);
            fs::remove_dir_all(&folder_path)
                .map_err(|e| format!("Failed to delete {} folder: {}", folder_name, e))?;
        }
    }
    
    // Restore backed up folders
    let mut restored_count = 0;
    for folder_name in &["dbc", "edt", "lnc"] {
        let backup_folder = backup_dir.join(folder_name);
        if backup_folder.exists() {
            let dest_folder = db_dir.join(folder_name);
            tracing::info!("Restoring {} folder: {:?} -> {:?}", folder_name, backup_folder, dest_folder);
            copy_dir_recursive(&backup_folder, &dest_folder)?;
            restored_count += 1;
        } else {
            tracing::warn!("Backup {} folder not found", folder_name);
        }
    }
    
    tracing::info!("Restored {} folders", restored_count);
    
    // Note: Editor data files are not removed on uninstall as they don't interfere
    // User can manually delete them if desired
    tracing::info!("Note: Editor data files in user directory were not removed");
    
    Ok(())
}

/// Delete licensing files as part of installation
fn delete_licensing_files(db_dir: &Path) -> Result<(), String> {
    tracing::info!("Deleting licensing files from: {:?}", db_dir);

    let mut deleted_count = 0;
    let mut not_found_count = 0;
    
    for (subdir, files) in FILES_TO_DELETE {
        let dir = db_dir.join(subdir);
        tracing::debug!("Checking directory: {:?}", dir);
        
        if !dir.exists() {
            tracing::warn!("Directory does not exist: {:?}", dir);
            continue;
        }

        for file in *files {
            let file_path = dir.join(file);
            if file_path.exists() {
                fs::remove_file(&file_path)
                    .map_err(|e| format!("Failed to delete {}: {}", file, e))?;
                deleted_count += 1;
                tracing::info!("Deleted licensing file: {}", file);
            } else {
                not_found_count += 1;
                tracing::debug!("Licensing file not found (already deleted or doesn't exist): {}", file);
            }
        }
    }

    tracing::info!("Deleted {} licensing files, {} files not found", deleted_count, not_found_count);
    Ok(())
}

/// Install FM Name Fix
pub fn install() -> Result<String, String> {
    let config = load_config()?;
    let db_dir = get_db_dir(config.target_path.as_deref())?;

    tracing::info!("Starting FM Name Fix installation");

    // Create backups before making any changes
    create_backups(&db_dir)?;

    // Download the name fix
    let zip_data = download_name_fix()?;

    // Extract the .lnc file
    let lnc_data = extract_lnc_file(&zip_data)?;

    // Write the .lnc file to the correct location
    let lnc_dir = db_dir.join("lnc").join("all");
    fs::create_dir_all(&lnc_dir)
        .map_err(|e| format!("Failed to create lnc directory: {}", e))?;

    let lnc_file = lnc_dir.join(NAME_FIX_FILE);
    let mut file = fs::File::create(&lnc_file)
        .map_err(|e| format!("Failed to create {}: {}", NAME_FIX_FILE, e))?;

    file.write_all(&lnc_data)
        .map_err(|e| format!("Failed to write {}: {}", NAME_FIX_FILE, e))?;

    tracing::info!("Wrote {} to {:?}", NAME_FIX_FILE, lnc_file);

    // Delete licensing files
    delete_licensing_files(&db_dir)?;

    tracing::info!("FM Name Fix installation completed successfully");
    let app_data_dir = get_app_data_dir();
    Ok(format!(
        "FM Name Fix installed successfully! The following changes were made:\n\
        - Installed {} to fix licensing issues\n\
        - Removed stock licensing files\n\
        - Created backup at {}\n\n\
        Note: For existing saves, Brazilian clubs will update after you start a new save.",
        NAME_FIX_FILE,
        app_data_dir.join("name_fix_backup").display()
    ))
}

/// Uninstall FM Name Fix
pub fn uninstall() -> Result<String, String> {
    let config = load_config()?;
    let db_dir = get_db_dir(config.target_path.as_deref())?;

    tracing::info!("Starting FM Name Fix uninstallation");

    restore_from_backup(&db_dir)?;

    // Clear active name fix from config
    let mut config = load_config()?;
    config.active_name_fix = None;
    save_config(&config)?;

    tracing::info!("FM Name Fix uninstallation completed successfully");
    Ok("FM Name Fix uninstalled successfully! Original licensing files have been restored.".to_string())
}

/// Get list of all available name fix sources
pub fn list_name_fixes() -> Result<Vec<NameFixSource>, String> {
    let name_fixes_dir = get_name_fixes_dir();
    let mut sources = Vec::new();

    // Scan for imported name fixes
    if name_fixes_dir.exists() {
        if let Ok(entries) = fs::read_dir(&name_fixes_dir) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    // Check if this directory has a metadata.json file
                    let metadata_file = entry_path.join("metadata.json");
                    if metadata_file.exists() {
                        if let Ok(metadata_str) = fs::read_to_string(&metadata_file) {
                            if let Ok(source) = serde_json::from_str::<NameFixSource>(&metadata_str) {
                                sources.push(source);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(sources)
}

/// Import a name fix from a ZIP file
pub fn import_name_fix(file_path: String, name: String) -> Result<String, String> {
    tracing::info!("=== IMPORT NAME FIX CALLED ===");
    tracing::info!("File path: {}", file_path);
    tracing::info!("Name: {}", name);

    let source_path = PathBuf::from(&file_path);
    if !source_path.exists() {
        tracing::error!("Source file does not exist: {:?}", source_path);
        return Err("Source file does not exist".to_string());
    }
    
    tracing::info!("Source file exists: {:?}", source_path);

    // Detect the install type from ZIP structure
    let install_type = detect_install_type(&source_path)?;
    tracing::info!("Detected install type: {:?}", install_type);

    // Generate a unique ID for this name fix
    let id = format!("imported-{}", uuid::Uuid::new_v4());
    tracing::info!("Generated ID: {}", id);
    
    // Create directory for this name fix
    let name_fixes_dir = get_name_fixes_dir();
    tracing::info!("Name fixes directory: {:?}", name_fixes_dir);
    
    let fix_dir = name_fixes_dir.join(&id);
    tracing::info!("Creating directory: {:?}", fix_dir);
    
    fs::create_dir_all(&fix_dir)
        .map_err(|e| format!("Failed to create name fix directory: {}", e))?;
    
    tracing::info!("Directory created successfully");

    // Extract files based on install type
    tracing::info!("Starting extraction...");
    let file_count = match install_type {
        NameFixInstallType::Files => extract_files_type(&source_path, &fix_dir)?,
        NameFixInstallType::Folders => extract_folders_type(&source_path, &fix_dir)?,
    };
    tracing::info!("Extraction complete, {} items extracted", file_count);

    // Create metadata
    let type_desc = match install_type {
        NameFixInstallType::Files => "File-based",
        NameFixInstallType::Folders => "Folder-based",
    };
    
    let source = NameFixSource {
        id: id.clone(),
        name: name.clone(),
        source_type: NameFixSourceType::Imported,
        install_type,
        description: format!("{} - Imported from {} ({} items)", type_desc, source_path.file_name().unwrap_or_default().to_string_lossy(), file_count),
        imported_date: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };

    // Save metadata
    let metadata_file = fix_dir.join("metadata.json");
    let metadata_json = serde_json::to_string_pretty(&source)
        .map_err(|e| format!("Failed to serialize metadata: {}", e))?;
    fs::write(&metadata_file, metadata_json)
        .map_err(|e| format!("Failed to save metadata: {}", e))?;

    tracing::info!("Name fix imported successfully: {}", name);
    Ok(format!("Successfully imported '{}' ({}) with {} items", name, type_desc, file_count))
}

/// Detect whether this is a file-based or folder-based name fix
fn detect_install_type(zip_path: &Path) -> Result<NameFixInstallType, String> {
    let file = fs::File::open(zip_path)
        .map_err(|e| format!("Failed to open file: {}", e))?;

    let mut archive = ZipArchive::new(file)
        .map_err(|e| format!("Failed to read ZIP archive: {}", e))?;

    let mut has_folders = false;
    let mut has_editor_data = false;
    let mut has_individual_files = false;

    for i in 0..archive.len() {
        let file = archive.by_index(i)
            .map_err(|e| format!("Failed to read ZIP entry: {}", e))?;
        
        let name = file.name();
        
        // Check for Sortitoutsi style (folders + editor data) at any depth
        if name.contains("dbc/") || name.contains("edt/") || name.contains("lnc/") {
            has_folders = true;
        }
        if name.contains("editor data/") {
            has_editor_data = true;
        }
        
        // Check for FMScout style (individual .lnc/.edt/.dbc files in root or shallow depth)
        // These files should not be deep in the folder structure
        let path_depth = name.matches('/').count();
        if path_depth <= 1 && (name.ends_with(".lnc") || name.ends_with(".edt") || name.ends_with(".dbc")) {
            has_individual_files = true;
        }
    }

    // Sortitoutsi if it has the folder structure and editor data
    if has_folders && has_editor_data {
        Ok(NameFixInstallType::Folders)
    } else if has_individual_files || has_folders {
        // FMScout if it has individual files or just folders without editor data
        Ok(NameFixInstallType::Files)
    } else {
        Err("Could not determine name fix type. ZIP should contain either .lnc/.edt/.dbc files or dbc/edt/lnc folders.".to_string())
    }
}

/// Extract file-based name fix (FMScout style)
fn extract_files_type(zip_path: &Path, dest_dir: &Path) -> Result<usize, String> {
    tracing::info!("Extracting file-based name fix");
    extract_all_namefix_files(zip_path, dest_dir)
}

/// Extract all name fix files from a ZIP archive
/// Extracts all .lnc, .edt, and .dbc files from the ZIP
fn extract_all_namefix_files(zip_path: &Path, dest_dir: &Path) -> Result<usize, String> {
    tracing::info!("Extracting name fix files from: {:?} to {:?}", zip_path, dest_dir);
    
    let file = fs::File::open(zip_path)
        .map_err(|e| format!("Failed to open file: {}", e))?;

    let mut archive = ZipArchive::new(file)
        .map_err(|e| format!("Failed to read ZIP archive: {}", e))?;

    tracing::info!("ZIP archive contains {} entries", archive.len());
    
    let mut file_count = 0;

    // Extract all .lnc, .edt, and .dbc files
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("Failed to read ZIP entry {}: {}", i, e))?;

        let file_name = file.name().to_string(); // Clone the name before using the file
        
        tracing::debug!("Processing ZIP entry: {}", file_name);
        
        // Skip directories
        if file_name.ends_with('/') {
            tracing::debug!("Skipping directory: {}", file_name);
            continue;
        }
        
        // Check if this is a relevant file type
        if file_name.ends_with(".lnc") || file_name.ends_with(".edt") || file_name.ends_with(".dbc") {
            // Extract just the filename (remove any directory structure from the ZIP)
            let path = PathBuf::from(&file_name);
            let filename = path.file_name()
                .ok_or_else(|| format!("Invalid file name in archive: {}", file_name))?;
            
            let dest_file = dest_dir.join(filename);
            
            let mut contents = Vec::new();
            file.read_to_end(&mut contents)
                .map_err(|e| format!("Failed to read file from archive: {}", e))?;

            fs::write(&dest_file, &contents)
                .map_err(|e| format!("Failed to write file: {}", e))?;

            tracing::info!("Extracted: {} ({} bytes) -> {:?}", filename.to_string_lossy(), contents.len(), dest_file);
            file_count += 1;
        } else {
            tracing::debug!("Skipping non-namefix file: {}", file_name);
        }
    }

    if file_count == 0 {
        return Err("No valid name fix files (.lnc, .edt, or .dbc) found in ZIP archive".to_string());
    }

    tracing::info!("Successfully extracted {} files", file_count);
    Ok(file_count)
}

/// Extract folder-based name fix (Sortitoutsi style)
/// Extracts dbc, edt, lnc folders and editor data folder
fn extract_folders_type(zip_path: &Path, dest_dir: &Path) -> Result<usize, String> {
    tracing::info!("Extracting folder-based name fix (Sortitoutsi style)");
    
    let file = fs::File::open(zip_path)
        .map_err(|e| format!("Failed to open file: {}", e))?;

    let mut archive = ZipArchive::new(file)
        .map_err(|e| format!("Failed to read ZIP archive: {}", e))?;

    tracing::info!("ZIP archive contains {} entries", archive.len());
    
    let mut item_count = 0;

    // Extract all files, preserving folder structure but stripping leading path
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("Failed to read ZIP entry {}: {}", i, e))?;

        let file_name = file.name().to_string();
        
        // Skip directories
        if file_name.ends_with('/') {
            continue;
        }
        
        // Check if this file is in one of the target folders (at any depth)
        let relevant_path = if let Some(idx) = file_name.find("dbc/") {
            Some(&file_name[idx..])
        } else if let Some(idx) = file_name.find("edt/") {
            Some(&file_name[idx..])
        } else if let Some(idx) = file_name.find("lnc/") {
            Some(&file_name[idx..])
        } else if let Some(idx) = file_name.find("editor data/") {
            Some(&file_name[idx..])
        } else {
            None
        };
        
        if let Some(rel_path) = relevant_path {
            let dest_file = dest_dir.join(rel_path);
            
            // Create parent directories
            if let Some(parent) = dest_file.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            }
            
            let mut contents = Vec::new();
            file.read_to_end(&mut contents)
                .map_err(|e| format!("Failed to read file from archive: {}", e))?;

            fs::write(&dest_file, &contents)
                .map_err(|e| format!("Failed to write file: {}", e))?;

            tracing::debug!("Extracted: {} -> {:?}", file_name, dest_file);
            item_count += 1;
        }
    }

    if item_count == 0 {
        return Err("No valid name fix folders (dbc/, edt/, lnc/, editor data/) found in ZIP archive".to_string());
    }

    tracing::info!("Successfully extracted {} items", item_count);
    Ok(item_count)
}

/// Extract .lnc file from a ZIP archive
fn extract_lnc_from_file(zip_path: &Path) -> Result<Vec<u8>, String> {
    let file = fs::File::open(zip_path)
        .map_err(|e| format!("Failed to open file: {}", e))?;

    let mut archive = ZipArchive::new(file)
        .map_err(|e| format!("Failed to read ZIP archive: {}", e))?;

    // Look for any .lnc file in the archive
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("Failed to read ZIP entry: {}", e))?;

        if file.name().ends_with(".lnc") {
            let mut contents = Vec::new();
            file.read_to_end(&mut contents)
                .map_err(|e| format!("Failed to read .lnc file from archive: {}", e))?;

            tracing::info!("Found .lnc file: {} ({} bytes)", file.name(), contents.len());
            return Ok(contents);
        }
    }

    Err("No .lnc file found in ZIP archive".to_string())
}

/// Install a specific name fix by ID
pub fn install_name_fix(name_fix_id: String) -> Result<String, String> {
    let config = load_config()?;
    let db_dir = get_db_dir(config.target_path.as_deref())?;

    tracing::info!("Installing name fix: {}", name_fix_id);

    // Get the name fix metadata to determine install type
    let name_fixes_dir = get_name_fixes_dir();
    let fix_dir = name_fixes_dir.join(&name_fix_id);
    
    if !fix_dir.exists() {
        return Err("Name fix not found".to_string());
    }
    
    let metadata_file = fix_dir.join("metadata.json");
    let metadata_str = fs::read_to_string(&metadata_file)
        .map_err(|e| format!("Failed to read metadata: {}", e))?;
    let source: NameFixSource = serde_json::from_str(&metadata_str)
        .map_err(|e| format!("Failed to parse metadata: {}", e))?;

    tracing::info!("Install type: {:?}", source.install_type);

    // Create backups before making any changes
    match source.install_type {
        NameFixInstallType::Files => create_backups(&db_dir)?,
        NameFixInstallType::Folders => create_folder_backups(&db_dir)?,
    }

    // Install based on type
    match source.install_type {
        NameFixInstallType::Files => install_files_type(&fix_dir, &db_dir)?,
        NameFixInstallType::Folders => install_folders_type(&fix_dir, &db_dir, config.user_dir_path.as_deref())?,
    }

    // Update config to track active name fix
    let mut config = load_config()?;
    config.active_name_fix = Some(name_fix_id);
    save_config(&config)?;

    tracing::info!("Name fix installation completed successfully");
    let app_data_dir = get_app_data_dir();
    
    let message = match source.install_type {
        NameFixInstallType::Files => format!(
            "Name fix installed successfully! The following changes were made:\n\
            - Installed name fix files to fix licensing issues\n\
            - Removed stock licensing files\n\
            - Created backup at {}\n\n\
            Note: For existing saves, Brazilian clubs will update after you start a new save.",
            app_data_dir.join("name_fix_backup").display()
        ),
        NameFixInstallType::Folders => format!(
            "Name fix installed successfully! The following changes were made:\n\
            - Replaced dbc, edt, and lnc folders\n\
            - Added editor data files\n\
            - Created backup at {}\n\n\
            Note: You must restart FM26 for changes to take effect. For existing saves, some changes require a new game.",
            app_data_dir.join("name_fix_backup").display()
        ),
    };
    
    Ok(message)
}

/// Install file-based name fix (FMScout style)
fn install_files_type(fix_dir: &Path, db_dir: &Path) -> Result<(), String> {
    tracing::info!("Installing file-based name fix");
    
    let mut installed_count = 0;

    // Read all files from the imported name fix directory
    let entries = fs::read_dir(fix_dir)
        .map_err(|e| format!("Failed to read name fix directory: {}", e))?;

    for entry in entries.flatten() {
        let file_path = entry.path();
        if !file_path.is_file() {
            continue;
        }

        let filename = file_path.file_name()
            .ok_or_else(|| "Invalid file name".to_string())?
            .to_string_lossy();

        // Skip metadata.json
        if filename == "metadata.json" {
            continue;
        }

        // Determine destination based on file extension
        let dest_path = if filename.ends_with(".lnc") {
            db_dir.join("lnc").join("all").join(filename.as_ref())
        } else if filename.ends_with(".edt") {
            db_dir.join("edt").join("permanent").join(filename.as_ref())
        } else if filename.ends_with(".dbc") {
            // Language files typically have _chn suffix or contain "licensing" without "_post_"
            if filename.to_lowercase().contains("_chn") || 
               (filename.to_lowercase().contains("licensing") && !filename.contains("_post_")) {
                db_dir.join("dbc").join("language").join(filename.as_ref())
            } else {
                db_dir.join("dbc").join("permanent").join(filename.as_ref())
            }
        } else {
            continue; // Skip unknown file types
        };

        // Create parent directory if needed
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        // Copy the file
        fs::copy(&file_path, &dest_path)
            .map_err(|e| format!("Failed to copy {}: {}", filename, e))?;

        tracing::info!("Installed: {} -> {:?}", filename, dest_path);
        installed_count += 1;
    }

    if installed_count == 0 {
        return Err("No valid files found in imported name fix".to_string());
    }

    tracing::info!("Installed {} files from imported name fix", installed_count);
    
    // Delete stock licensing files
    delete_licensing_files(db_dir)?;
    
    Ok(())
}

/// Install folder-based name fix (Sortitoutsi style)
fn install_folders_type(fix_dir: &Path, db_dir: &Path, user_dir: Option<&str>) -> Result<(), String> {
    tracing::info!("Installing folder-based name fix (Sortitoutsi style)");
    
    // Delete the existing dbc, edt, lnc folders
    for folder_name in &["dbc", "edt", "lnc"] {
        let folder_path = db_dir.join(folder_name);
        if folder_path.exists() {
            tracing::info!("Deleting existing {} folder: {:?}", folder_name, folder_path);
            fs::remove_dir_all(&folder_path)
                .map_err(|e| format!("Failed to delete {} folder: {}", folder_name, e))?;
        }
    }
    
    // Copy the new folders from the imported name fix
    let mut installed_count = 0;
    for folder_name in &["dbc", "edt", "lnc"] {
        let src_folder = fix_dir.join(folder_name);
        if src_folder.exists() {
            let dest_folder = db_dir.join(folder_name);
            tracing::info!("Copying {} folder: {:?} -> {:?}", folder_name, src_folder, dest_folder);
            copy_dir_recursive(&src_folder, &dest_folder)?;
            installed_count += 1;
        }
    }
    
    if installed_count == 0 {
        return Err("No dbc/edt/lnc folders found in name fix".to_string());
    }
    
    tracing::info!("Installed {} folders", installed_count);
    
    // Handle editor data folder
    let editor_data_src = fix_dir.join("editor data");
    if editor_data_src.exists() {
        if let Some(user_dir_path) = user_dir {
            let user_dir_path = PathBuf::from(user_dir_path);
            let editor_data_dest = user_dir_path.join("editor data");
            
            tracing::info!("Copying editor data: {:?} -> {:?}", editor_data_src, editor_data_dest);
            
            // Create editor data directory if it doesn't exist
            fs::create_dir_all(&editor_data_dest)
                .map_err(|e| format!("Failed to create editor data directory: {}", e))?;
            
            // Copy all files from editor data folder
            if let Ok(entries) = fs::read_dir(&editor_data_src) {
                for entry in entries.flatten() {
                    let src_file = entry.path();
                    if src_file.is_file() {
                        let filename = src_file.file_name().ok_or("Invalid filename")?;
                        let dest_file = editor_data_dest.join(filename);
                        fs::copy(&src_file, &dest_file)
                            .map_err(|e| format!("Failed to copy editor data file: {}", e))?;
                        tracing::info!("Copied editor data file: {:?}", filename);
                    }
                }
            }
        } else {
            tracing::warn!("User directory not set, skipping editor data installation");
        }
    }
    
    Ok(())
}

/// Recursively copy a directory
fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest)
        .map_err(|e| format!("Failed to create directory {:?}: {}", dest, e))?;
    
    let entries = fs::read_dir(src)
        .map_err(|e| format!("Failed to read directory {:?}: {}", src, e))?;
    
    for entry in entries.flatten() {
        let src_path = entry.path();
        let filename = entry.file_name();
        let dest_path = dest.join(&filename);
        
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            fs::copy(&src_path, &dest_path)
                .map_err(|e| format!("Failed to copy file {:?}: {}", src_path, e))?;
        }
    }
    
    Ok(())
}

/// Delete an imported name fix
pub fn delete_name_fix(name_fix_id: String) -> Result<String, String> {
    if name_fix_id == GITHUB_NAME_FIX_ID {
        return Err("Cannot delete the built-in GitHub name fix".to_string());
    }

    let fix_dir = get_name_fixes_dir().join(&name_fix_id);
    if !fix_dir.exists() {
        return Err("Name fix not found".to_string());
    }

    // If this is the active name fix, uninstall it first
    let config = load_config()?;
    if config.active_name_fix.as_deref() == Some(&name_fix_id) {
        uninstall()?;
    }

    // Delete the directory
    fs::remove_dir_all(&fix_dir)
        .map_err(|e| format!("Failed to delete name fix: {}", e))?;

    tracing::info!("Deleted name fix: {}", name_fix_id);
    Ok("Name fix deleted successfully".to_string())
}

/// Get the currently active name fix ID
pub fn get_active_name_fix() -> Result<Option<String>, String> {
    let config = load_config()?;
    Ok(config.active_name_fix)
}
