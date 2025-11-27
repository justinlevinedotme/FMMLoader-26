//! Shared utility functions for file operations and directory management.

use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Recursively copy a directory and all its contents.
///
/// This is the single source of truth for directory copying across the application.
/// Use this instead of implementing copy logic inline.
pub fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| format!("Failed to create directory {:?}: {}", dst, e))?;

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
                fs::copy(path, &target_path).map_err(|e| format!("Failed to copy file: {}", e))?;
            }
        }
    }

    Ok(())
}

/// Count the number of files in a directory recursively.
pub fn count_files_in_dir(dir: &Path) -> Result<usize, String> {
    Ok(WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .count())
}

/// Find the actual graphics content root in an extracted directory.
///
/// Skips wrapper folders and finds where faces/, logos/, kits/ actually live.
pub fn find_graphics_content_root(extracted_dir: &PathBuf) -> Result<PathBuf, String> {
    // Graphics subdirectory names to look for
    let _graphics_dirs = ["faces", "logos", "kits", "badges"];

    // Check if current directory contains any graphics subdirectories
    fn has_graphics_subdirs(path: &Path) -> bool {
        let graphics_dirs = ["faces", "logos", "kits", "badges"];
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        let name_lower = name.to_lowercase();
                        if graphics_dirs.iter().any(|&gd| name_lower.contains(gd)) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    // If extracted_dir itself has graphics subdirs, it's the content root
    if has_graphics_subdirs(extracted_dir) {
        return Ok(extracted_dir.clone());
    }

    // Otherwise, search one or two levels deep for the content root
    if let Ok(entries) = fs::read_dir(extracted_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Check if this subdirectory has graphics subdirs
                if has_graphics_subdirs(&path) {
                    return Ok(path);
                }

                // Check one more level deep (for deeply nested structures)
                if let Ok(sub_entries) = fs::read_dir(&path) {
                    for sub_entry in sub_entries.flatten() {
                        let sub_path = sub_entry.path();
                        if sub_path.is_dir() && has_graphics_subdirs(&sub_path) {
                            return Ok(sub_path);
                        }
                    }
                }
            }
        }
    }

    // If we didn't find graphics subdirs, just return the extracted dir
    Ok(extracted_dir.clone())
}
