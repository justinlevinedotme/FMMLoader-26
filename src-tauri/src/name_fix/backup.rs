//! Backup and restore operations for name fixes.

use crate::config::{get_app_data_dir, get_name_fixes_dir, load_config};
use crate::types::{NameFixInstallType, NameFixSource};
use crate::utils;
use std::fs;
use std::path::Path;

use super::constants::FILES_TO_DELETE;

/// Create backups of files that will be modified or deleted (for file-based installs)
pub fn create_backups(db_dir: &Path) -> Result<(), String> {
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
pub fn create_folder_backups(db_dir: &Path) -> Result<(), String> {
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
            tracing::info!(
                "Backing up {} folder: {:?} -> {:?}",
                folder_name,
                source_folder,
                backup_folder
            );
            utils::copy_dir_recursive(&source_folder, &backup_folder)?;
        } else {
            tracing::warn!("{} folder does not exist, skipping backup", folder_name);
        }
    }

    tracing::info!("Folder backups created successfully");
    Ok(())
}

/// Restore files from backup
pub fn restore_from_backup(db_dir: &Path) -> Result<(), String> {
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
                        NameFixInstallType::Files => {
                            restore_files_backup(db_dir, &backup_dir, &fix_dir)?
                        }
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
                tracing::debug!(
                    "Backup file not found (was not present during backup): {}",
                    file
                );
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

            let filename = source_file
                .file_name()
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
                if filename.to_lowercase().contains("_chn")
                    || (filename.to_lowercase().contains("licensing")
                        && !filename.contains("_post_"))
                {
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
            tracing::info!(
                "Restoring {} folder: {:?} -> {:?}",
                folder_name,
                backup_folder,
                dest_folder
            );
            utils::copy_dir_recursive(&backup_folder, &dest_folder)?;
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
