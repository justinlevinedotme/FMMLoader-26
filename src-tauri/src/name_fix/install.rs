//! Install and uninstall name fixes.

use crate::config::{get_app_data_dir, get_name_fixes_dir, load_config, save_config};
use crate::types::{NameFixInstallType, NameFixSource};
use crate::utils;
use std::fs;
use std::path::{Path, PathBuf};

use super::backup::{create_backups, create_folder_backups, restore_from_backup};
use super::constants::FILES_TO_DELETE;
use super::db::get_db_dir;

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
        NameFixInstallType::Folders => {
            install_folders_type(&fix_dir, &db_dir, config.user_dir_path.as_deref())?
        }
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
    let entries =
        fs::read_dir(fix_dir).map_err(|e| format!("Failed to read name fix directory: {}", e))?;

    for entry in entries.flatten() {
        let file_path = entry.path();
        if !file_path.is_file() {
            continue;
        }

        let filename = file_path
            .file_name()
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
            if filename.to_lowercase().contains("_chn")
                || (filename.to_lowercase().contains("licensing") && !filename.contains("_post_"))
            {
                db_dir.join("dbc").join("language").join(filename.as_ref())
            } else {
                db_dir.join("dbc").join("permanent").join(filename.as_ref())
            }
        } else {
            continue; // Skip unknown file types
        };

        // Create parent directory if needed
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
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
fn install_folders_type(
    fix_dir: &Path,
    db_dir: &Path,
    user_dir: Option<&str>,
) -> Result<(), String> {
    tracing::info!("Installing folder-based name fix (Sortitoutsi style)");

    // Delete the existing dbc, edt, lnc folders
    for folder_name in &["dbc", "edt", "lnc"] {
        let folder_path = db_dir.join(folder_name);
        if folder_path.exists() {
            tracing::info!(
                "Deleting existing {} folder: {:?}",
                folder_name,
                folder_path
            );
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
            tracing::info!(
                "Copying {} folder: {:?} -> {:?}",
                folder_name,
                src_folder,
                dest_folder
            );
            utils::copy_dir_recursive(&src_folder, &dest_folder)?;
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

            tracing::info!(
                "Copying editor data: {:?} -> {:?}",
                editor_data_src,
                editor_data_dest
            );

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
                tracing::debug!(
                    "Licensing file not found (already deleted or doesn't exist): {}",
                    file
                );
            }
        }
    }

    tracing::info!(
        "Deleted {} licensing files, {} files not found",
        deleted_count,
        not_found_count
    );
    Ok(())
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
    Ok(
        "FM Name Fix uninstalled successfully! Original licensing files have been restored."
            .to_string(),
    )
}
