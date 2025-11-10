use crate::types::Config;
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

pub fn get_app_data_dir() -> PathBuf {
    let app_name = "FMMLoader26";

    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA")
            .unwrap_or_else(|_| {
                let home = dirs::home_dir().unwrap();
                home.join("AppData").join("Roaming").to_string_lossy().to_string()
            });
        PathBuf::from(appdata).join(app_name)
    }

    #[cfg(target_os = "macos")]
    {
        dirs::home_dir()
            .unwrap()
            .join("Library")
            .join("Application Support")
            .join(app_name)
    }

    #[cfg(target_os = "linux")]
    {
        dirs::home_dir()
            .unwrap()
            .join(".local")
            .join("share")
            .join(app_name)
    }
}

pub fn init_storage() -> Result<(), String> {
    let base_dir = get_app_data_dir();

    // Create necessary directories
    let dirs = vec![
        base_dir.join("backups"),
        base_dir.join("mods"),
        base_dir.join("logs"),
        base_dir.join("restore_points"),
    ];

    for dir in dirs {
        fs::create_dir_all(&dir).map_err(|e| format!("Failed to create dir {:?}: {}", dir, e))?;
    }

    Ok(())
}

pub fn get_config_path() -> PathBuf {
    get_app_data_dir().join("config.json")
}

pub fn load_config() -> Result<Config, String> {
    let config_path = get_config_path();

    if !config_path.exists() {
        return Ok(Config {
            target_path: None,
            user_dir_path: None,
            enabled_mods: Vec::new(),
        });
    }

    let contents = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    serde_json::from_str(&contents)
        .map_err(|e| format!("Failed to parse config: {}", e))
}

pub fn save_config(config: &Config) -> Result<(), String> {
    let config_path = get_config_path();

    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&config_path, json)
        .map_err(|e| format!("Failed to write config: {}", e))?;

    Ok(())
}

pub fn get_mods_dir() -> PathBuf {
    get_app_data_dir().join("mods")
}

pub fn get_backup_dir() -> PathBuf {
    get_app_data_dir().join("backups")
}

pub fn get_restore_points_dir() -> PathBuf {
    get_app_data_dir().join("restore_points")
}

pub fn get_logs_dir() -> PathBuf {
    get_app_data_dir().join("logs")
}
