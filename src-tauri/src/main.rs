// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod game_detection;
mod mod_manager;
mod types;

use config::{get_mods_dir, init_storage, load_config, save_config};
use game_detection::{get_default_candidates, get_fm_user_dir};
use mod_manager::{cleanup_old_backups, cleanup_old_restore_points, get_mod_info, install_mod, list_mods, uninstall_mod};
use types::{Config, ModManifest};

#[tauri::command]
fn init_app() -> Result<(), String> {
    init_storage()?;
    cleanup_old_backups(10)?;
    cleanup_old_restore_points(10)?;
    Ok(())
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

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            init_app,
            get_config,
            update_config,
            detect_game_path,
            set_game_target,
            get_mods_list,
            get_mod_details,
            enable_mod,
            disable_mod,
            apply_mods,
            remove_mod,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
