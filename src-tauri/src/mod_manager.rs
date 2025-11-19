use crate::config::{get_backup_dir, get_mods_dir, get_restore_points_dir};
use crate::game_detection::get_fm_user_dir;
use crate::types::{FileEntry, ModInstallPreview, ModManifest, ResolvedFilePreview};
use chrono::Local;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn read_manifest(mod_dir: &Path) -> Result<ModManifest, String> {
    let manifest_path = mod_dir.join("manifest.json");

    if !manifest_path.exists() {
        return Err(format!("No manifest.json in {:?}", mod_dir));
    }

    let contents = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read manifest: {}", e))?;

    let mut manifest: ModManifest =
        serde_json::from_str(&contents).map_err(|e| format!("Failed to parse manifest: {}", e))?;

    // Set defaults
    if manifest.name.is_empty() {
        manifest.name = mod_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();
    }

    Ok(manifest)
}

pub fn list_mods() -> Result<Vec<String>, String> {
    let mods_dir = get_mods_dir();

    if !mods_dir.exists() {
        return Ok(Vec::new());
    }

    let entries =
        fs::read_dir(&mods_dir).map_err(|e| format!("Failed to read mods directory: {}", e))?;

    let mut mods = Vec::new();

    for entry in entries.flatten() {
        if entry.path().is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                mods.push(name.to_string());
            }
        }
    }

    Ok(mods)
}

pub fn get_mod_info(mod_name: &str) -> Result<ModManifest, String> {
    let mod_dir = get_mods_dir().join(mod_name);

    if !mod_dir.exists() {
        return Err(format!("Mod not found: {}", mod_name));
    }

    read_manifest(&mod_dir)
}

pub fn backup_file(target_file: &Path) -> Result<Option<PathBuf>, String> {
    if !target_file.exists() {
        return Ok(None);
    }

    let backup_dir = get_backup_dir();
    fs::create_dir_all(&backup_dir).map_err(|e| format!("Failed to create backup dir: {}", e))?;

    let filename = target_file
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Invalid filename")?;

    // Create a unique backup filename
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_name = format!("{}_{}.bak", filename, timestamp);
    let backup_path = backup_dir.join(&backup_name);

    fs::copy(target_file, &backup_path).map_err(|e| format!("Failed to backup file: {}", e))?;

    Ok(Some(backup_path))
}

fn copy_recursive(src: &Path, dst: &Path) -> io::Result<u64> {
    let mut count = 0;

    if src.is_dir() {
        fs::create_dir_all(dst)?;

        for entry in WalkDir::new(src) {
            let entry = entry?;
            let path = entry.path();

            if let Ok(rel_path) = path.strip_prefix(src) {
                let target_path = dst.join(rel_path);

                if path.is_dir() {
                    fs::create_dir_all(&target_path)?;
                } else {
                    if let Some(parent) = target_path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::copy(path, &target_path)?;
                    count += 1;
                }
            }
        }
    } else {
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(src, dst)?;
        count = 1;
    }

    Ok(count)
}

pub fn get_target_for_type(mod_type: &str, game_target: &Path, user_dir: Option<&str>) -> PathBuf {
    let user_path = get_fm_user_dir(user_dir);

    match mod_type {
        "ui" | "bundle" | "skins" => game_target.to_path_buf(),
        "tactics" => user_path.join("tactics"),
        "graphics" => user_path.join("graphics"),
        "editor-data" => user_path.join("editor data"),
        _ => game_target.to_path_buf(),
    }
}

pub fn preview_mod_install(
    mod_type: &str,
    game_target: &Path,
    user_dir: Option<&str>,
    files: &[FileEntry],
) -> ModInstallPreview {
    let base_target = get_target_for_type(mod_type, game_target, user_dir);
    let resolved_files = files
        .iter()
        .map(|file| ResolvedFilePreview {
            target_subpath: file.target_subpath.clone(),
            resolved_path: base_target
                .join(&file.target_subpath)
                .to_string_lossy()
                .to_string(),
        })
        .collect();

    ModInstallPreview {
        base_target: base_target.to_string_lossy().to_string(),
        resolved_files,
    }
}

pub fn install_mod(
    mod_name: &str,
    game_target: &Path,
    user_dir: Option<&str>,
) -> Result<String, String> {
    let mod_dir = get_mods_dir().join(mod_name);

    if !mod_dir.exists() {
        return Err(format!("Mod not found: {}", mod_name));
    }

    let manifest = read_manifest(&mod_dir)?;
    let target_base = get_target_for_type(&manifest.mod_type, game_target, user_dir);

    if manifest.files.is_empty() {
        return Err("Mod has no files to install".to_string());
    }

    let mut installed_count = 0;
    let current_platform = get_current_platform();

    for file_entry in &manifest.files {
        // Skip files that don't match the current platform
        if let Some(ref platform) = file_entry.platform {
            if platform != &current_platform {
                continue;
            }
        }

        let src = mod_dir.join(&file_entry.source);

        if !src.exists() {
            continue;
        }

        let dst = target_base.join(&file_entry.target_subpath);

        // Backup existing file
        if dst.exists() {
            backup_file(&dst)?;
        }

        // Copy file or directory
        match copy_recursive(&src, &dst) {
            Ok(count) => installed_count += count,
            Err(e) => return Err(format!("Failed to install file: {}", e)),
        }
    }

    Ok(format!(
        "Installed {} with {} files",
        mod_name, installed_count
    ))
}

/// Get the current platform identifier
fn get_current_platform() -> String {
    #[cfg(target_os = "windows")]
    {
        "windows".to_string()
    }
    #[cfg(target_os = "macos")]
    {
        "macos".to_string()
    }
    #[cfg(target_os = "linux")]
    {
        "linux".to_string()
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        "unknown".to_string()
    }
}

#[allow(dead_code)]
pub fn uninstall_mod(
    mod_name: &str,
    game_target: &Path,
    user_dir: Option<&str>,
) -> Result<String, String> {
    let mod_dir = get_mods_dir().join(mod_name);

    if !mod_dir.exists() {
        return Err(format!("Mod not found: {}", mod_name));
    }

    let manifest = read_manifest(&mod_dir)?;
    let target_base = get_target_for_type(&manifest.mod_type, game_target, user_dir);

    let mut removed_count = 0;

    for file_entry in &manifest.files {
        let dst = target_base.join(&file_entry.target_subpath);

        if dst.exists() {
            if dst.is_dir() {
                fs::remove_dir_all(&dst)
                    .map_err(|e| format!("Failed to remove directory: {}", e))?;
            } else {
                fs::remove_file(&dst).map_err(|e| format!("Failed to remove file: {}", e))?;
            }
            removed_count += 1;
        }
    }

    Ok(format!(
        "Uninstalled {} - removed {} items",
        mod_name, removed_count
    ))
}

#[allow(dead_code)]
pub fn create_restore_point(name: &str) -> Result<PathBuf, String> {
    let restore_dir = get_restore_points_dir();
    fs::create_dir_all(&restore_dir)
        .map_err(|e| format!("Failed to create restore points dir: {}", e))?;

    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let point_name = format!("{}_{}", timestamp, name);
    let point_dir = restore_dir.join(&point_name);

    fs::create_dir_all(&point_dir).map_err(|e| format!("Failed to create restore point: {}", e))?;

    Ok(point_dir)
}

pub fn cleanup_old_backups(keep: usize) -> Result<(), String> {
    let backup_dir = get_backup_dir();

    if !backup_dir.exists() {
        return Ok(());
    }

    let mut backups: Vec<_> = fs::read_dir(&backup_dir)
        .map_err(|e| format!("Failed to read backup dir: {}", e))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect();

    backups.sort_by_key(|e| {
        e.metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });

    backups.reverse();

    for old_backup in backups.iter().skip(keep) {
        let _ = fs::remove_file(old_backup.path());
    }

    Ok(())
}

pub fn cleanup_old_restore_points(keep: usize) -> Result<(), String> {
    let restore_dir = get_restore_points_dir();

    if !restore_dir.exists() {
        return Ok(());
    }

    let mut points: Vec<_> = fs::read_dir(&restore_dir)
        .map_err(|e| format!("Failed to read restore points dir: {}", e))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();

    points.sort_by_key(|e| {
        e.metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });

    points.reverse();

    for old_point in points.iter().skip(keep) {
        let _ = fs::remove_dir_all(old_point.path());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("fmml_mod_manager_test_{}", nanos));
        let _ = std::fs::create_dir_all(&path);
        path
    }

    #[test]
    fn test_get_current_platform() {
        let platform = get_current_platform();

        // Verify it returns one of the expected values
        assert!(
            platform == "windows"
                || platform == "macos"
                || platform == "linux"
                || platform == "unknown",
            "Platform should be one of: windows, macos, linux, unknown"
        );

        // Verify it matches the current OS
        #[cfg(target_os = "windows")]
        assert_eq!(platform, "windows");

        #[cfg(target_os = "macos")]
        assert_eq!(platform, "macos");

        #[cfg(target_os = "linux")]
        assert_eq!(platform, "linux");
    }

    #[test]
    fn test_get_target_for_type_skins() {
        let game_target = PathBuf::from("/test/game/path");
        let user_dir = Some("/should/not/use/user");

        let target = get_target_for_type("skins", &game_target, user_dir);

        // Verify skins go to game target (bundle folder), not user skins folder
        assert_eq!(target, game_target);
    }

    #[test]
    fn test_get_target_for_type_ui() {
        let game_target = PathBuf::from("/test/game/path");
        let user_dir = Some("/should/not/use/user");

        let target = get_target_for_type("ui", &game_target, user_dir);
        assert_eq!(target, game_target);
    }

    #[test]
    fn test_get_target_for_type_bundle() {
        let game_target = PathBuf::from("/test/game/path");
        let user_dir = Some("/should/not/use/user");

        let target = get_target_for_type("bundle", &game_target, user_dir);
        assert_eq!(target, game_target);
    }

    #[test]
    fn test_get_target_for_type_tactics() {
        let game_target = PathBuf::from("/test/game/path");
        let user_dir = unique_temp_dir();
        let user_dir_str = user_dir.to_string_lossy().to_string();

        let target = get_target_for_type("tactics", &game_target, Some(&user_dir_str));

        assert_eq!(target, user_dir.join("tactics"));
        let _ = std::fs::remove_dir_all(&user_dir);
    }

    #[test]
    fn test_get_target_for_type_graphics() {
        let game_target = PathBuf::from("/test/game/path");
        let user_dir = unique_temp_dir();
        let user_dir_str = user_dir.to_string_lossy().to_string();

        let target = get_target_for_type("graphics", &game_target, Some(&user_dir_str));

        assert_eq!(target, user_dir.join("graphics"));
        let _ = std::fs::remove_dir_all(&user_dir);
    }

    #[test]
    fn test_get_target_for_type_editor_data() {
        let game_target = PathBuf::from("/test/game/path");
        let user_dir = unique_temp_dir();
        let user_dir_str = user_dir.to_string_lossy().to_string();

        let target = get_target_for_type("editor-data", &game_target, Some(&user_dir_str));

        assert_eq!(target, user_dir.join("editor data"));
        let _ = std::fs::remove_dir_all(&user_dir);
    }

    #[test]
    fn test_preview_mod_install_maps_paths() {
        let game_target = PathBuf::from("/test/game/path");
        let user_dir = unique_temp_dir();
        let user_dir_str = user_dir.to_string_lossy().to_string();

        let files = vec![
            FileEntry {
                source: "src/file1".to_string(),
                target_subpath: "graphics/faces/config.xml".to_string(),
                platform: None,
            },
            FileEntry {
                source: "src/file2".to_string(),
                target_subpath: "graphics/faces/face.png".to_string(),
                platform: None,
            },
        ];

        let preview = preview_mod_install("graphics", &game_target, Some(&user_dir_str), &files);

        assert_eq!(
            preview.base_target,
            user_dir.join("graphics").to_string_lossy().to_string()
        );
        assert_eq!(preview.resolved_files.len(), 2);
        assert_eq!(
            preview.resolved_files[0].resolved_path,
            user_dir
                .join("graphics")
                .join("graphics/faces/config.xml")
                .to_string_lossy()
                .to_string()
        );

        let _ = std::fs::remove_dir_all(&user_dir);
    }
}
