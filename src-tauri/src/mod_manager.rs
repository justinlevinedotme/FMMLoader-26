use crate::config::{get_backup_dir, get_mods_dir, get_restore_points_dir};
use crate::game_detection::get_fm_user_dir;
use crate::types::{FileEntry, ModManifest};
use chrono::Local;
use std::collections::HashMap;
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

    let mut manifest: ModManifest = serde_json::from_str(&contents)
        .map_err(|e| format!("Failed to parse manifest: {}", e))?;

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

    let entries = fs::read_dir(&mods_dir)
        .map_err(|e| format!("Failed to read mods directory: {}", e))?;

    let mut mods = Vec::new();

    for entry in entries {
        if let Ok(entry) = entry {
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    mods.push(name.to_string());
                }
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
    fs::create_dir_all(&backup_dir)
        .map_err(|e| format!("Failed to create backup dir: {}", e))?;

    let filename = target_file
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Invalid filename")?;

    // Create a unique backup filename
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_name = format!("{}_{}.bak", filename, timestamp);
    let backup_path = backup_dir.join(&backup_name);

    fs::copy(target_file, &backup_path)
        .map_err(|e| format!("Failed to backup file: {}", e))?;

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

pub fn get_target_for_type(
    mod_type: &str,
    game_target: &Path,
    user_dir: Option<&str>,
) -> PathBuf {
    let user_path = get_fm_user_dir(user_dir);

    match mod_type {
        "ui" | "bundle" => game_target.to_path_buf(),
        "tactics" => user_path.join("tactics"),
        "graphics" => user_path.join("graphics"),
        "skins" => user_path.join("skins"),
        "editor-data" => user_path.join("editor data"),
        _ => game_target.to_path_buf(),
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

    for file_entry in &manifest.files {
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

pub fn create_restore_point(name: &str) -> Result<PathBuf, String> {
    let restore_dir = get_restore_points_dir();
    fs::create_dir_all(&restore_dir)
        .map_err(|e| format!("Failed to create restore points dir: {}", e))?;

    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let point_name = format!("{}_{}", timestamp, name);
    let point_dir = restore_dir.join(&point_name);

    fs::create_dir_all(&point_dir)
        .map_err(|e| format!("Failed to create restore point: {}", e))?;

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
