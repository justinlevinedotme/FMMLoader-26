use crate::types::{Config, GraphicsPackMetadata, GraphicsPacksRegistry};
use std::env;
use std::fs;
use std::path::PathBuf;

pub fn get_app_data_dir() -> PathBuf {
    if let Ok(override_dir) = env::var("FMML_TEST_APPDATA") {
        let path = PathBuf::from(override_dir);
        if !path.as_os_str().is_empty() {
            return path;
        }
    }

    let app_name = "FMMLoader26";

    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| {
            let home = dirs::home_dir().unwrap();
            home.join("AppData")
                .join("Roaming")
                .to_string_lossy()
                .to_string()
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
        base_dir.join("name_fixes"),
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
            dark_mode: false,
            language: None,
            active_name_fix: None,
        });
    }

    let contents =
        fs::read_to_string(&config_path).map_err(|e| format!("Failed to read config: {}", e))?;

    serde_json::from_str(&contents).map_err(|e| format!("Failed to parse config: {}", e))
}

pub fn save_config(config: &Config) -> Result<(), String> {
    let config_path = get_config_path();

    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&config_path, json).map_err(|e| format!("Failed to write config: {}", e))?;

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

#[allow(dead_code)]
pub fn get_logs_dir() -> PathBuf {
    get_app_data_dir().join("logs")
}

pub fn get_name_fixes_dir() -> PathBuf {
    get_app_data_dir().join("name_fixes")
}

pub fn get_graphics_packs_path() -> PathBuf {
    get_app_data_dir().join("graphics_packs.json")
}

pub fn load_graphics_packs() -> Result<GraphicsPacksRegistry, String> {
    let path = get_graphics_packs_path();

    if !path.exists() {
        return Ok(GraphicsPacksRegistry::default());
    }

    let contents = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read graphics packs registry: {}", e))?;

    serde_json::from_str(&contents)
        .map_err(|e| format!("Failed to parse graphics packs registry: {}", e))
}

pub fn save_graphics_packs(registry: &GraphicsPacksRegistry) -> Result<(), String> {
    let path = get_graphics_packs_path();

    let json = serde_json::to_string_pretty(registry)
        .map_err(|e| format!("Failed to serialize graphics packs registry: {}", e))?;

    fs::write(&path, json)
        .map_err(|e| format!("Failed to write graphics packs registry: {}", e))?;

    Ok(())
}

#[allow(dead_code)]
pub fn add_graphics_pack(metadata: GraphicsPackMetadata) -> Result<(), String> {
    let mut registry = load_graphics_packs()?;
    registry.graphics_packs.push(metadata);
    save_graphics_packs(&registry)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = env::temp_dir().join(format!("fmml_test_appdata_{}", nanos));
        let _ = fs::create_dir_all(&path);
        path
    }

    #[test]
    fn get_app_data_dir_honors_test_override() {
        let temp_dir = unique_temp_dir();
        env::set_var("FMML_TEST_APPDATA", &temp_dir);

        let base = get_app_data_dir();
        assert_eq!(base, temp_dir);
        assert_eq!(get_mods_dir(), base.join("mods"));
        assert_eq!(get_backup_dir(), base.join("backups"));
        assert_eq!(get_restore_points_dir(), base.join("restore_points"));
        assert_eq!(get_logs_dir(), base.join("logs"));
        assert_eq!(get_name_fixes_dir(), base.join("name_fixes"));
        assert_eq!(get_graphics_packs_path(), base.join("graphics_packs.json"));

        env::remove_var("FMML_TEST_APPDATA");
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
