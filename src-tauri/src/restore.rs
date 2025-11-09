use crate::config::get_restore_points_dir;
use crate::types::RestorePoint;
use chrono::Local;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

pub fn create_restore_point(name: &str, source_paths: &[PathBuf]) -> Result<PathBuf, String> {
    let restore_dir = get_restore_points_dir();
    fs::create_dir_all(&restore_dir)
        .map_err(|e| format!("Failed to create restore points dir: {}", e))?;

    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let point_name = format!("{}_{}", timestamp, name);
    let point_dir = restore_dir.join(&point_name);

    fs::create_dir_all(&point_dir)
        .map_err(|e| format!("Failed to create restore point: {}", e))?;

    // Copy all source paths to restore point
    for (i, source_path) in source_paths.iter().enumerate() {
        if !source_path.exists() {
            continue;
        }

        let dest_name = format!("backup_{}", i);
        let dest_path = point_dir.join(&dest_name);

        if source_path.is_dir() {
            copy_dir_all(source_path, &dest_path)?;
        } else {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent dir: {}", e))?;
            }
            fs::copy(source_path, &dest_path)
                .map_err(|e| format!("Failed to copy file: {}", e))?;
        }

        // Save metadata about original location
        let metadata_path = point_dir.join(format!("{}.meta", dest_name));
        fs::write(&metadata_path, source_path.to_string_lossy().as_bytes())
            .map_err(|e| format!("Failed to write metadata: {}", e))?;
    }

    Ok(point_dir)
}

pub fn list_restore_points() -> Result<Vec<RestorePoint>, String> {
    let restore_dir = get_restore_points_dir();

    if !restore_dir.exists() {
        return Ok(Vec::new());
    }

    let mut points = Vec::new();

    let entries = fs::read_dir(&restore_dir)
        .map_err(|e| format!("Failed to read restore points dir: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name() {
                let timestamp = entry
                    .metadata()
                    .and_then(|m| m.modified())
                    .ok()
                    .and_then(|t| {
                        use std::time::UNIX_EPOCH;
                        t.duration_since(UNIX_EPOCH).ok()
                    })
                    .map(|d| {
                        let datetime = chrono::DateTime::<chrono::Utc>::from(
                            UNIX_EPOCH + d
                        );
                        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
                    })
                    .unwrap_or_else(|| "Unknown".to_string());

                points.push(RestorePoint {
                    timestamp,
                    path,
                });
            }
        }
    }

    // Sort by timestamp (newest first)
    points.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    Ok(points)
}

pub fn rollback_to_restore_point(point_path: &PathBuf) -> Result<String, String> {
    if !point_path.exists() {
        return Err("Restore point not found".to_string());
    }

    let entries = fs::read_dir(point_path)
        .map_err(|e| format!("Failed to read restore point: {}", e))?;

    let mut restored_count = 0;

    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = match path.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => continue,
        };

        // Skip metadata files
        if file_name.ends_with(".meta") {
            continue;
        }

        // Read metadata to get original location
        let meta_path = point_path.join(format!("{}.meta", file_name));
        if !meta_path.exists() {
            continue;
        }

        let original_location = fs::read_to_string(&meta_path)
            .map_err(|e| format!("Failed to read metadata: {}", e))?;

        let original_path = PathBuf::from(original_location.trim());

        // Restore the file/directory
        if path.is_dir() {
            if original_path.exists() {
                fs::remove_dir_all(&original_path)
                    .map_err(|e| format!("Failed to remove existing dir: {}", e))?;
            }
            copy_dir_all(&path, &original_path)?;
        } else {
            if let Some(parent) = original_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent dir: {}", e))?;
            }
            fs::copy(&path, &original_path)
                .map_err(|e| format!("Failed to restore file: {}", e))?;
        }

        restored_count += 1;
    }

    Ok(format!("Restored {} items", restored_count))
}

fn copy_dir_all(src: &PathBuf, dst: &PathBuf) -> Result<(), String> {
    fs::create_dir_all(dst)
        .map_err(|e| format!("Failed to create directory: {}", e))?;

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
