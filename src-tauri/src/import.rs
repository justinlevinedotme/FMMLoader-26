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

pub fn auto_detect_mod_type(dir: &Path) -> String {
    let mut has_bundle = false;
    let mut has_fmf = false;
    let mut has_xml = false;
    let mut has_config = false;
    let mut has_panel = false;
    let mut has_edt = false;

    if let Ok(entries) = walkdir::WalkDir::new(dir).max_depth(2).into_iter().collect::<Result<Vec<_>, _>>() {
        for entry in entries {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                let ext_lower = ext.to_string_lossy().to_lowercase();
                match ext_lower.as_str() {
                    "bundle" => has_bundle = true,
                    "fmf" => has_fmf = true,
                    "xml" => has_xml = true,
                    "inc" if path.to_string_lossy().contains("config") => has_config = true,
                    "xml" if path.to_string_lossy().contains("panel") => has_panel = true,
                    "edt" => has_edt = true,
                    _ => {}
                }
            }

            // Check for graphics subdirectories
            if let Some(name) = path.file_name() {
                let name_lower = name.to_string_lossy().to_lowercase();
                if ["faces", "kits", "logos", "badges"].contains(&name_lower.as_str()) {
                    return "graphics".to_string();
                }
            }
        }
    }

    // Determine type based on files found
    if has_bundle || has_fmf {
        "ui".to_string()
    } else if has_edt {
        "editor-data".to_string()
    } else if has_config && has_panel {
        "skins".to_string()
    } else if has_xml {
        // Check if it's in a tactics-like structure
        if dir.to_string_lossy().to_lowercase().contains("tactics") {
            "tactics".to_string()
        } else {
            "misc".to_string()
        }
    } else {
        "misc".to_string()
    }
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
