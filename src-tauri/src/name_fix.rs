use crate::config::{load_config, save_config, get_app_data_dir, get_name_fixes_dir};
use crate::types::{NameFixSource, NameFixSourceType};
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
pub fn check_installed(target_path: Option<&str>) -> Result<bool, String> {
    let db_dir = get_db_dir(target_path)?;
    let lnc_file = db_dir.join("lnc").join("all").join(NAME_FIX_FILE);

    Ok(lnc_file.exists())
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

/// Restore files from backup
fn restore_from_backup(db_dir: &Path) -> Result<(), String> {
    let app_data_dir = get_app_data_dir();
    let backup_dir = app_data_dir.join("name_fix_backup");

    if !backup_dir.exists() {
        return Err("No backup found. Cannot uninstall FM Name Fix.".to_string());
    }

    tracing::info!("Restoring from backup at {:?}", backup_dir);

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
                tracing::debug!("Restored {}", file);
            }
        }
    }

    // Remove the FM26-open-names.lnc file
    let lnc_file = db_dir.join("lnc").join("all").join(NAME_FIX_FILE);
    if lnc_file.exists() {
        fs::remove_file(&lnc_file)
            .map_err(|e| format!("Failed to remove {}: {}", NAME_FIX_FILE, e))?;
        tracing::debug!("Removed {}", NAME_FIX_FILE);
    }

    // Remove backup directory
    fs::remove_dir_all(&backup_dir)
        .map_err(|e| format!("Failed to remove backup directory: {}", e))?;

    tracing::info!("Restore completed successfully");
    Ok(())
}

/// Delete licensing files as part of installation
fn delete_licensing_files(db_dir: &Path) -> Result<(), String> {
    tracing::info!("Deleting licensing files");

    let mut deleted_count = 0;
    for (subdir, files) in FILES_TO_DELETE {
        let dir = db_dir.join(subdir);

        for file in *files {
            let file_path = dir.join(file);
            if file_path.exists() {
                fs::remove_file(&file_path)
                    .map_err(|e| format!("Failed to delete {}: {}", file, e))?;
                deleted_count += 1;
                tracing::debug!("Deleted {}", file);
            }
        }
    }

    tracing::info!("Deleted {} licensing files", deleted_count);
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

    // Add the built-in GitHub source
    sources.push(NameFixSource {
        id: GITHUB_NAME_FIX_ID.to_string(),
        name: "NameFix FM26 (GitHub)".to_string(),
        source_type: NameFixSourceType::GitHub,
        description: "Official open-source name fix from jo13310/NameFixFM26".to_string(),
        imported_date: "Built-in".to_string(),
    });

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
    tracing::info!("Importing name fix from: {}", file_path);

    let source_path = PathBuf::from(&file_path);
    if !source_path.exists() {
        return Err("Source file does not exist".to_string());
    }

    // Extract and validate the ZIP file
    let lnc_data = extract_lnc_from_file(&source_path)?;

    // Generate a unique ID for this name fix
    let id = format!("imported-{}", uuid::Uuid::new_v4());
    
    // Create directory for this name fix
    let name_fixes_dir = get_name_fixes_dir();
    let fix_dir = name_fixes_dir.join(&id);
    fs::create_dir_all(&fix_dir)
        .map_err(|e| format!("Failed to create name fix directory: {}", e))?;

    // Save the .lnc file
    let lnc_file = fix_dir.join(NAME_FIX_FILE);
    fs::write(&lnc_file, &lnc_data)
        .map_err(|e| format!("Failed to save name fix file: {}", e))?;

    // Create metadata
    let source = NameFixSource {
        id: id.clone(),
        name: name.clone(),
        source_type: NameFixSourceType::Imported,
        description: format!("Imported from {}", source_path.file_name().unwrap_or_default().to_string_lossy()),
        imported_date: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };

    // Save metadata
    let metadata_file = fix_dir.join("metadata.json");
    let metadata_json = serde_json::to_string_pretty(&source)
        .map_err(|e| format!("Failed to serialize metadata: {}", e))?;
    fs::write(&metadata_file, metadata_json)
        .map_err(|e| format!("Failed to save metadata: {}", e))?;

    tracing::info!("Name fix imported successfully: {}", name);
    Ok(format!("Successfully imported '{}'", name))
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

    // Create backups before making any changes
    create_backups(&db_dir)?;

    let lnc_data = if name_fix_id == GITHUB_NAME_FIX_ID {
        // Download from GitHub
        let zip_data = download_name_fix()?;
        extract_lnc_file(&zip_data)?
    } else {
        // Load from imported name fix
        let fix_dir = get_name_fixes_dir().join(&name_fix_id);
        if !fix_dir.exists() {
            return Err("Name fix not found".to_string());
        }

        let lnc_file = fix_dir.join(NAME_FIX_FILE);
        fs::read(&lnc_file)
            .map_err(|e| format!("Failed to read name fix file: {}", e))?
    };

    // Write the .lnc file to the game database
    let lnc_dir = db_dir.join("lnc").join("all");
    fs::create_dir_all(&lnc_dir)
        .map_err(|e| format!("Failed to create lnc directory: {}", e))?;

    let lnc_file = lnc_dir.join(NAME_FIX_FILE);
    fs::write(&lnc_file, &lnc_data)
        .map_err(|e| format!("Failed to write {}: {}", NAME_FIX_FILE, e))?;

    tracing::info!("Wrote {} to {:?}", NAME_FIX_FILE, lnc_file);

    // Delete licensing files
    delete_licensing_files(&db_dir)?;

    // Update config to track active name fix
    let mut config = load_config()?;
    config.active_name_fix = Some(name_fix_id);
    save_config(&config)?;

    tracing::info!("Name fix installation completed successfully");
    let app_data_dir = get_app_data_dir();
    Ok(format!(
        "Name fix installed successfully! The following changes were made:\n\
        - Installed {} to fix licensing issues\n\
        - Removed stock licensing files\n\
        - Created backup at {}\n\n\
        Note: For existing saves, Brazilian clubs will update after you start a new save.",
        NAME_FIX_FILE,
        app_data_dir.join("name_fix_backup").display()
    ))
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
