// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod conflicts;
mod game_detection;
mod import;
mod logging;
mod mod_manager;
mod name_fix;
mod restore;
mod types;

use config::{get_mods_dir, init_storage, load_config, save_config};
use conflicts::find_conflicts;
use game_detection::get_default_candidates;
use import::{auto_detect_mod_type, extract_zip, find_mod_root, generate_manifest, has_manifest};
use mod_manager::{cleanup_old_backups, cleanup_old_restore_points, get_mod_info, install_mod, list_mods};
use restore::{create_restore_point, list_restore_points, rollback_to_restore_point};
use types::{Config, ConflictInfo, ModManifest, RestorePoint};
use std::path::PathBuf;

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
    Ok(candidates.iter().map(|p| p.to_string_lossy().to_string()).collect())
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

    let target_path = config
        .target_path
        .as_ref()
        .ok_or("Game target not set")?;

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
    tracing::debug!("Import params - name: {:?}, version: {:?}, type: {:?}", mod_name, version, mod_type);

    let source = PathBuf::from(&source_path);
    let mods_dir = get_mods_dir();

    if !source.exists() {
        tracing::error!("Source path does not exist: {}", source_path);
        return Err("Source path does not exist".to_string());
    }

    tracing::info!("Source exists: {:?}, is_file: {}, is_dir: {}", source, source.is_file(), source.is_dir());

    // Handle different source types
    let mod_root = if source.is_file() {
        let ext = source.extension().and_then(|s| s.to_str());
        tracing::info!("File extension: {:?}", ext);

        if ext == Some("zip") {
            // Extract ZIP to temp directory
            let temp_dir = std::env::temp_dir().join(format!("fmmloader_import_{}", uuid::Uuid::new_v4()));
            tracing::info!("Extracting ZIP to: {:?}", temp_dir);
            extract_zip(&source, &temp_dir)?;
            let root = find_mod_root(&temp_dir)?;
            tracing::info!("Found mod root in ZIP: {:?}", root);
            root
        } else {
            // Single file (.bundle, .fmf, etc) - create temp dir with just this file
            let temp_dir = std::env::temp_dir().join(format!("fmmloader_import_{}", uuid::Uuid::new_v4()));
            tracing::info!("Creating temp directory for single file: {:?}", temp_dir);
            fs::create_dir_all(&temp_dir)
                .map_err(|e| {
                    tracing::error!("Failed to create temp directory: {}", e);
                    format!("Failed to create temp directory: {}", e)
                })?;

            let file_name = source.file_name()
                .ok_or("Invalid file name")?;
            let dest_file = temp_dir.join(file_name);

            tracing::info!("Copying file to: {:?}", dest_file);
            fs::copy(&source, &dest_file)
                .map_err(|e| {
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

    let target_path = config
        .target_path
        .as_ref()
        .ok_or("Game target not set")?;

    let target = PathBuf::from(target_path);

    find_conflicts(&config.enabled_mods, &target, config.user_dir_path.as_deref())
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

    let target_path = config
        .target_path
        .as_ref()
        .ok_or("Game target not set")?;

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
fn check_name_fix_installed() -> Result<bool, String> {
    let config = load_config()?;
    name_fix::check_installed(config.target_path.as_deref())
}

#[tauri::command]
fn install_name_fix() -> Result<String, String> {
    name_fix::install()
}

#[tauri::command]
fn uninstall_name_fix() -> Result<String, String> {
    name_fix::uninstall()
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
                fs::copy(path, &target_path)
                    .map_err(|e| format!("Failed to copy file: {}", e))?;
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
        .plugin({
            tracing::info!("Initializing updater plugin");
            tauri_plugin_updater::Builder::new().build()
        })
        .setup(|app| {
            tracing::info!("Tauri app setup complete");
            tracing::info!("Updater is active and will check for updates");
            Ok(())
        })
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
            get_logs_path,
            check_name_fix_installed,
            install_name_fix,
            uninstall_name_fix,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
