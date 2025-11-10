use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

pub fn extract_zip(zip_path: &Path, dest_dir: &Path) -> Result<PathBuf, String> {
    let file = fs::File::open(zip_path)
        .map_err(|e| format!("Failed to open zip file: {}", e))?;

    let mut archive = ZipArchive::new(file)
        .map_err(|e| format!("Failed to read zip archive: {}", e))?;

    fs::create_dir_all(dest_dir)
        .map_err(|e| format!("Failed to create destination directory: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("Failed to read file from archive: {}", e))?;

        let outpath = match file.enclosed_name() {
            Some(path) => dest_dir.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        } else {
            if let Some(p) = outpath.parent() {
                fs::create_dir_all(p)
                    .map_err(|e| format!("Failed to create parent directory: {}", e))?;
            }
            let mut outfile = fs::File::create(&outpath)
                .map_err(|e| format!("Failed to create output file: {}", e))?;
            io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("Failed to extract file: {}", e))?;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))
                    .ok();
            }
        }
    }

    Ok(dest_dir.to_path_buf())
}

pub fn has_manifest(dir: &Path) -> bool {
    for name in &["manifest.json", "Manifest.json", "MANIFEST.JSON"] {
        if dir.join(name).exists() {
            return true;
        }
    }
    false
}

pub fn find_mod_root(path: &Path) -> Result<PathBuf, String> {
    if path.is_dir() {
        // If this directory has a manifest, use it
        if has_manifest(path) {
            return Ok(path.to_path_buf());
        }

        // Otherwise search one level deep
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_dir() && has_manifest(&entry_path) {
                    return Ok(entry_path);
                }
            }
        }

        // If still no manifest, just return the original path
        return Ok(path.to_path_buf());
    }

    // If it's a file, return its parent directory
    Ok(path.parent()
        .ok_or("Invalid path")?
        .to_path_buf())
}

pub fn auto_detect_mod_type(path: &Path) -> String {
    // Handle single files
    if path.is_file() {
        if let Some(ext) = path.extension() {
            let ext_lower = ext.to_string_lossy().to_lowercase();
            let name_lower = path.file_name()
                .map(|n| n.to_string_lossy().to_lowercase())
                .unwrap_or_default();

            match ext_lower.as_str() {
                "fmf" => return "tactics".to_string(),
                "bundle" => {
                    // Check if it's a UI bundle
                    if name_lower.contains("ui-") || name_lower.contains("panelids") {
                        return "ui".to_string();
                    }
                    return "bundle".to_string();
                }
                _ => {}
            }
        }
        return "misc".to_string();
    }

    // For directories, check contents
    let mut has_bundle = false;
    let mut has_fmf = false;
    let mut has_graphics = false;
    let mut has_editor_data = false;

    if let Ok(entries) = walkdir::WalkDir::new(path).into_iter().collect::<Result<Vec<_>, _>>() {
        for entry in entries {
            let entry_path = entry.path();

            // Check for file extensions
            if entry_path.is_file() {
                if let Some(ext) = entry_path.extension() {
                    let ext_lower = ext.to_string_lossy().to_lowercase();
                    match ext_lower.as_str() {
                        "fmf" => has_fmf = true,
                        "bundle" => has_bundle = true,
                        _ => {}
                    }
                }
            }

            // Check for directory names indicating graphics or editor data
            if entry_path.is_dir() {
                if let Some(name) = entry_path.file_name() {
                    let name_lower = name.to_string_lossy().to_lowercase();
                    let name_normalized = name_lower.replace(' ', "").replace('_', "");

                    // Check for graphics directories
                    if ["kits", "faces", "logos", "graphics", "badges"].contains(&name_lower.as_str()) {
                        has_graphics = true;
                    }

                    // Check for editor data directories
                    if ["editordata", "editor"].contains(&name_normalized.as_str()) {
                        has_editor_data = true;
                    }
                }
            }
        }
    }

    // Determine type based on what we found
    // Editor data takes priority if we have FMF files in editor data folder
    if has_fmf && has_editor_data {
        return "editor-data".to_string();
    }

    if has_fmf {
        return "tactics".to_string();
    }

    if has_bundle {
        return "ui".to_string();
    }

    if has_graphics {
        return "graphics".to_string();
    }

    "misc".to_string()
}

pub fn generate_manifest(
    dir: &Path,
    name: String,
    version: String,
    mod_type: String,
    author: String,
    description: String,
) -> Result<(), String> {
    use crate::types::{Compatibility, FileEntry, ModManifest};

    // Collect all files in the mod directory
    let mut files = Vec::new();

    if let Ok(entries) = walkdir::WalkDir::new(dir).into_iter().collect::<Result<Vec<_>, _>>() {
        for entry in entries {
            let path = entry.path();
            if path.is_file() {
                if let Ok(rel_path) = path.strip_prefix(dir) {
                    let rel_str = rel_path.to_string_lossy().to_string();
                    files.push(FileEntry {
                        source: rel_str.clone(),
                        target_subpath: rel_str,
                        platform: None,
                    });
                }
            }
        }
    }

    let manifest = ModManifest {
        name,
        version,
        mod_type,
        author,
        homepage: String::new(),
        description,
        license: String::new(),
        compatibility: Compatibility {
            fm_version: String::new(),
        },
        dependencies: Vec::new(),
        conflicts: Vec::new(),
        load_after: Vec::new(),
        files,
    };

    let manifest_path = dir.join("manifest.json");
    let json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| format!("Failed to serialize manifest: {}", e))?;

    fs::write(&manifest_path, json)
        .map_err(|e| format!("Failed to write manifest: {}", e))?;

    Ok(())
}
