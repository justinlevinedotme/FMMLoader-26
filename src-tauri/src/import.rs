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

    // Log validation warnings for missing fields
    if author.is_empty() {
        tracing::warn!("Manifest for '{}' is missing 'author' field", name);
    }
    if description.is_empty() {
        tracing::warn!("Manifest for '{}' is missing 'description' field", name);
    }
    // Note: fm_version will be empty by default, warn about it
    tracing::warn!("Manifest for '{}' is missing 'compatibility.fm_version' field", name);

    // Collect all files in the mod directory with platform detection
    let mut files = Vec::new();
    let mut has_platform_folders = false;
    let mut has_bundle_files = false;

    // First pass: Check if we have platform-specific folders and bundle files
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                if let Some(folder_name) = entry_path.file_name() {
                    let folder_str = folder_name.to_string_lossy().to_lowercase();
                    // Check for platform folder variants
                    if ["windows", "win", "macos", "mac", "osx", "linux"].contains(&folder_str.as_str()) {
                        has_platform_folders = true;
                    }
                }
            }
        }
    }

    // Check if mod contains bundle files (indicates platform-specific content)
    if let Ok(entries) = walkdir::WalkDir::new(dir).into_iter().collect::<Result<Vec<_>, _>>() {
        for entry in &entries {
            if entry.path().is_file() {
                if let Some(ext) = entry.path().extension() {
                    if ext.to_string_lossy().to_lowercase() == "bundle" {
                        has_bundle_files = true;
                        break;
                    }
                }
            }
        }
    }

    // Determine if we should use platform-specific detection
    // Only use platform detection if:
    // 1. Platform folders exist AND
    // 2. The mod type is UI/bundle-based (has .bundle files) OR mod_type is "ui" or "bundle"
    let use_platform_detection = has_platform_folders && 
        (has_bundle_files || mod_type == "ui" || mod_type == "bundle");

    // Second pass: Collect files with appropriate platform tags
    if let Ok(entries) = walkdir::WalkDir::new(dir).into_iter().collect::<Result<Vec<_>, _>>() {
        for entry in entries {
            let path = entry.path();
            if path.is_file() {
                if let Ok(rel_path) = path.strip_prefix(dir) {
                    let rel_str = rel_path.to_string_lossy().to_string();
                    
                    // Determine platform based on path
                    let platform = if use_platform_detection {
                        detect_platform_from_path(&rel_str)
                    } else {
                        None
                    };

                    // For platform-specific files, adjust target_subpath to remove platform folder
                    let target_subpath = if let Some(ref plat) = platform {
                        // Remove the platform folder prefix from target path
                        remove_platform_prefix(&rel_str, plat)
                    } else {
                        rel_str.clone()
                    };

                    files.push(FileEntry {
                        source: rel_str,
                        target_subpath,
                        platform,
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

/// Detect platform from file path based on platform-specific folder names
/// Supports common variations: windows/win, macos/mac/osx, linux
fn detect_platform_from_path(path: &str) -> Option<String> {
    let components: Vec<&str> = path.split('/').collect();
    
    for component in components {
        let comp_lower = component.to_lowercase();
        match comp_lower.as_str() {
            // Windows variants
            "windows" | "win" => return Some("windows".to_string()),
            // macOS variants
            "macos" | "mac" | "osx" => return Some("macos".to_string()),
            // Linux variants
            "linux" => return Some("linux".to_string()),
            _ => continue,
        }
    }
    
    None
}

/// Remove platform folder prefix from target path
/// Handles common platform name variations
fn remove_platform_prefix(path: &str, platform: &str) -> String {
    let parts: Vec<&str> = path.split('/').collect();
    
    // Build a list of platform folder names to remove based on the detected platform
    let platform_variants: Vec<&str> = match platform {
        "windows" => vec!["windows", "win"],
        "macos" => vec!["macos", "mac", "osx"],
        "linux" => vec!["linux"],
        _ => vec![],
    };
    
    // Find and remove any platform folder variant from the path
    let filtered_parts: Vec<&str> = parts.into_iter()
        .filter(|&part| {
            let part_lower = part.to_lowercase();
            !platform_variants.contains(&part_lower.as_str())
        })
        .collect();
    
    filtered_parts.join("/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_detect_platform_from_path_windows() {
        assert_eq!(detect_platform_from_path("windows/test.bundle"), Some("windows".to_string()));
        assert_eq!(detect_platform_from_path("Windows/test.bundle"), Some("windows".to_string()));
        assert_eq!(detect_platform_from_path("win/test.bundle"), Some("windows".to_string()));
    }

    #[test]
    fn test_detect_platform_from_path_macos() {
        assert_eq!(detect_platform_from_path("macos/test.bundle"), Some("macos".to_string()));
        assert_eq!(detect_platform_from_path("macOS/test.bundle"), Some("macos".to_string()));
        assert_eq!(detect_platform_from_path("mac/test.bundle"), Some("macos".to_string()));
        assert_eq!(detect_platform_from_path("osx/test.bundle"), Some("macos".to_string()));
    }

    #[test]
    fn test_detect_platform_from_path_linux() {
        assert_eq!(detect_platform_from_path("linux/test.bundle"), Some("linux".to_string()));
        assert_eq!(detect_platform_from_path("Linux/ui/test.bundle"), Some("linux".to_string()));
    }

    #[test]
    fn test_detect_platform_from_path_no_platform() {
        assert_eq!(detect_platform_from_path("test.bundle"), None);
        assert_eq!(detect_platform_from_path("ui/test.bundle"), None);
    }

    #[test]
    fn test_remove_platform_prefix_windows() {
        assert_eq!(remove_platform_prefix("windows/test.bundle", "windows"), "test.bundle");
        assert_eq!(remove_platform_prefix("Windows/ui/test.bundle", "windows"), "ui/test.bundle");
        assert_eq!(remove_platform_prefix("win/test.bundle", "windows"), "test.bundle");
    }

    #[test]
    fn test_remove_platform_prefix_macos() {
        assert_eq!(remove_platform_prefix("macos/test.bundle", "macos"), "test.bundle");
        assert_eq!(remove_platform_prefix("macOS/graphics/test.png", "macos"), "graphics/test.png");
        assert_eq!(remove_platform_prefix("mac/test.bundle", "macos"), "test.bundle");
        assert_eq!(remove_platform_prefix("osx/ui/test.bundle", "macos"), "ui/test.bundle");
    }

    #[test]
    fn test_remove_platform_prefix_linux() {
        assert_eq!(remove_platform_prefix("linux/test.bundle", "linux"), "test.bundle");
    }

    #[test]
    fn test_generate_manifest_with_platform_folders() {
        let temp_dir = std::env::temp_dir().join(format!("test_manifest_{}", uuid::Uuid::new_v4()));
        
        // Create test structure with platform folders
        let windows_dir = temp_dir.join("windows");
        fs::create_dir_all(&windows_dir).expect("Failed to create windows dir");
        
        let mut file = fs::File::create(windows_dir.join("test.bundle")).expect("Failed to create file");
        file.write_all(b"test content").expect("Failed to write content");
        drop(file);

        // Generate manifest
        let result = generate_manifest(
            &temp_dir,
            "Test Mod".to_string(),
            "1.0.0".to_string(),
            "ui".to_string(),
            "Test Author".to_string(),
            "Test Description".to_string(),
        );

        assert!(result.is_ok(), "generate_manifest should succeed");

        // Read the generated manifest
        let manifest_path = temp_dir.join("manifest.json");
        assert!(manifest_path.exists(), "manifest.json should be created");

        let manifest_content = fs::read_to_string(&manifest_path).expect("Failed to read manifest");
        let manifest: crate::types::ModManifest = serde_json::from_str(&manifest_content).expect("Failed to parse manifest");

        // Verify platform-specific file entry
        assert!(!manifest.files.is_empty(), "Manifest should have files");
        
        let windows_file = manifest.files.iter().find(|f| f.source.contains("windows"));
        assert!(windows_file.is_some(), "Should have a file from windows folder");
        
        if let Some(file) = windows_file {
            assert_eq!(file.platform, Some("windows".to_string()), "File should have windows platform tag");
            assert!(!file.target_subpath.contains("windows"), "Target path should not contain 'windows' folder");
        }

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_generate_manifest_without_platform_folders() {
        let temp_dir = std::env::temp_dir().join(format!("test_manifest_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");
        
        // Create a regular file without platform folder
        let mut file = fs::File::create(temp_dir.join("test.bundle")).expect("Failed to create file");
        file.write_all(b"test content").expect("Failed to write content");
        drop(file);

        // Generate manifest
        let result = generate_manifest(
            &temp_dir,
            "Test Mod".to_string(),
            "1.0.0".to_string(),
            "ui".to_string(),
            "Test Author".to_string(),
            "Test Description".to_string(),
        );

        assert!(result.is_ok(), "generate_manifest should succeed");

        // Read the generated manifest
        let manifest_path = temp_dir.join("manifest.json");
        let manifest_content = fs::read_to_string(&manifest_path).expect("Failed to read manifest");
        let manifest: crate::types::ModManifest = serde_json::from_str(&manifest_content).expect("Failed to parse manifest");

        // Verify no platform tags
        for file in &manifest.files {
            assert_eq!(file.platform, None, "Files should not have platform tags when no platform folders exist");
        }

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_auto_detect_mod_type_bundle() {
        let temp_dir = std::env::temp_dir().join(format!("test_detect_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");
        
        let bundle_path = temp_dir.join("ui-test.bundle");
        fs::File::create(&bundle_path).expect("Failed to create bundle file");

        let mod_type = auto_detect_mod_type(&bundle_path);
        assert_eq!(mod_type, "ui");

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_auto_detect_mod_type_tactics() {
        let temp_dir = std::env::temp_dir().join(format!("test_detect_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");
        
        let fmf_path = temp_dir.join("tactic.fmf");
        fs::File::create(&fmf_path).expect("Failed to create fmf file");

        let mod_type = auto_detect_mod_type(&fmf_path);
        assert_eq!(mod_type, "tactics");

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
