//! Import Module - Archive Extraction and Mod Type Detection
//!
//! This module handles importing mods and graphics packs from various sources.
//! It provides both synchronous and asynchronous archive extraction with progress tracking.
//!
//! # Archive Extraction
//!
//! **Synchronous**: `extract_zip()` - Used for small mod imports where blocking is acceptable
//! **Asynchronous**: `extract_zip_async()` - Used for large graphics packs (5GB+) with progress events
//!
//! # Zip Bomb Protection
//!
//! The async extractor implements security limits:
//! - Maximum 50GB total extraction size
//! - Maximum 500,000 files per archive
//! - Early termination when limits exceeded
//!
//! # Progress Tracking
//!
//! Progress callbacks emit every 50 files (not per file) to balance responsiveness with performance.
//! Progress includes current file number, total files, current filename, and bytes processed.
//!
//! # Mod Type Detection
//!
//! `auto_detect_mod_type()` analyzes directory structure and file types to classify mods:
//! - Graphics: Contains faces/, logos/, kits/ directories or PNG files
//! - Tactics: Contains .fmf files
//! - Editor Data: Contains .dbc, .edt, .lnc files or editor data/ directory
//! - UI/Bundle: Default for other content

use crate::types::ExtractionProgress;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

pub fn extract_zip(zip_path: &Path, dest_dir: &Path) -> Result<PathBuf, String> {
    let file = fs::File::open(zip_path).map_err(|e| format!("Failed to open zip file: {}", e))?;

    let mut archive =
        ZipArchive::new(file).map_err(|e| format!("Failed to read zip archive: {}", e))?;

    fs::create_dir_all(dest_dir)
        .map_err(|e| format!("Failed to create destination directory: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
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
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).ok();
            }
        }
    }

    Ok(dest_dir.to_path_buf())
}

/// Async version of extract_zip that emits progress events
pub async fn extract_zip_async<F>(
    zip_path: PathBuf,
    dest_dir: PathBuf,
    mut progress_callback: F,
) -> Result<PathBuf, String>
where
    F: FnMut(ExtractionProgress) + Send + 'static,
{
    // Zip bomb protection limits
    const MAX_TOTAL_BYTES: u64 = 50 * 1024 * 1024 * 1024; // 50GB max extraction size
    const MAX_ENTRIES: usize = 500_000; // 500k files max

    tokio::task::spawn_blocking(move || {
        let file = fs::File::open(&zip_path)
            .map_err(|e| format!("Failed to open zip file: {}", e))?;

        let mut archive = ZipArchive::new(file)
            .map_err(|e| format!("Failed to read zip archive: {}", e))?;

        fs::create_dir_all(&dest_dir)
            .map_err(|e| format!("Failed to create destination directory: {}", e))?;

        let total = archive.len();

        // Check for excessive entry count (zip bomb indicator)
        if total > MAX_ENTRIES {
            return Err(format!(
                "Archive contains too many files ({}). Maximum allowed is {}. This may be a corrupted or malicious file.",
                total, MAX_ENTRIES
            ));
        }

        let mut bytes_processed = 0u64;

        for i in 0..total {
            let mut file = archive.by_index(i)
                .map_err(|e| format!("Failed to read file from archive: {}", e))?;

            let outpath = match file.enclosed_name() {
                Some(path) => dest_dir.join(path),
                None => continue,
            };

            let file_name = file.name().to_string();

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
                let bytes_copied = io::copy(&mut file, &mut outfile)
                    .map_err(|e| format!("Failed to extract file: {}", e))?;
                bytes_processed += bytes_copied;

                // Check for excessive extraction size (zip bomb indicator)
                if bytes_processed > MAX_TOTAL_BYTES {
                    return Err(format!(
                        "Archive extraction exceeded size limit ({}GB). Extracted {}GB so far. This may be a corrupted or malicious file.",
                        MAX_TOTAL_BYTES / 1024 / 1024 / 1024,
                        bytes_processed / 1024 / 1024 / 1024
                    ));
                }
            }

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))
                        .ok();
                }
            }

            // Emit progress every 50 files or on last file
            if i % 50 == 0 || i == total - 1 {
                progress_callback(ExtractionProgress {
                    current: i + 1,
                    total,
                    current_file: file_name,
                    bytes_processed,
                    phase: "extracting".to_string(),
                });
            }
        }

        Ok(dest_dir)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
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
    Ok(path.parent().ok_or("Invalid path")?.to_path_buf())
}

pub fn auto_detect_mod_type(path: &Path) -> String {
    // Handle single files
    if path.is_file() {
        if let Some(ext) = path.extension() {
            let ext_lower = ext.to_string_lossy().to_lowercase();
            let name_lower = path
                .file_name()
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

    if let Ok(entries) = walkdir::WalkDir::new(path)
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
    {
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
                    let name_normalized = name_lower.replace([' ', '_'], "");

                    // Check for graphics directories
                    if ["kits", "faces", "logos", "graphics", "badges"]
                        .contains(&name_lower.as_str())
                    {
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
    tracing::warn!(
        "Manifest for '{}' is missing 'compatibility.fm_version' field",
        name
    );

    // Collect all files in the mod directory with platform detection
    let mut files = Vec::new();
    let mut has_platform_folders = false;
    let mut has_bundle_files = false;

    // First pass: Check if we have platform-specific folders and bundle files
    if let Ok(entries) = fs::read_dir(dir) {
        let child_dirs: Vec<_> = entries.flatten().filter(|e| e.path().is_dir()).collect();

        // Check immediate children for platform folders
        for entry in &child_dirs {
            if let Some(folder_name) = entry.path().file_name() {
                let folder_str = folder_name.to_string_lossy().to_lowercase();
                if is_platform_component(&folder_str) {
                    has_platform_folders = true;
                }
            }
        }

        // If not found and there's exactly one non-platform folder, check one level deeper
        if !has_platform_folders && child_dirs.len() == 1 {
            if let Some(sole_child) = child_dirs.first() {
                if let Ok(nested) = fs::read_dir(sole_child.path()) {
                    for entry in nested.flatten() {
                        if let Some(folder_name) = entry.path().file_name() {
                            let folder_str = folder_name.to_string_lossy().to_lowercase();
                            if is_platform_component(&folder_str) {
                                has_platform_folders = true;
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    // Check if mod contains bundle files (indicates platform-specific content)
    if let Ok(entries) = walkdir::WalkDir::new(dir)
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
    {
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
    let use_platform_detection =
        has_platform_folders && (has_bundle_files || mod_type == "ui" || mod_type == "bundle");

    // Second pass: Collect files with appropriate platform tags
    if let Ok(entries) = walkdir::WalkDir::new(dir)
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
    {
        for entry in entries {
            let path = entry.path();
            if path.is_file() {
                if let Ok(rel_path) = path.strip_prefix(dir) {
                    // Normalize by removing any leading non-platform folder (common top-level zip folder)
                    let mut parts: Vec<String> = rel_path
                        .components()
                        .map(|c| c.as_os_str().to_string_lossy().into_owned())
                        .collect();

                    while parts.len() > 1 {
                        let first_lower = parts[0].to_lowercase();
                        if is_platform_component(&first_lower) {
                            break;
                        }
                        parts.remove(0);
                    }

                    let rel_joined = parts.join("/");

                    // Determine platform based on path
                    let platform = if use_platform_detection {
                        detect_platform_from_parts(&parts)
                    } else {
                        None
                    };

                    // For platform-specific files, adjust target_subpath to remove platform folder
                    let target_subpath = if let Some(ref plat) = platform {
                        remove_platform_parts(&parts, plat)
                    } else {
                        rel_joined.clone()
                    };

                    files.push(FileEntry {
                        source: rel_joined,
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

    fs::write(&manifest_path, json).map_err(|e| format!("Failed to write manifest: {}", e))?;

    Ok(())
}

/// Detect platform from path parts based on platform-specific folder names
/// Supports common variations: windows/win, macos/mac/osx, linux
fn detect_platform_from_parts(parts: &[String]) -> Option<String> {
    for component in parts {
        if let Some(platform) = platform_from_component(&component.to_lowercase()) {
            return Some(platform.to_string());
        }
    }

    None
}

fn platform_from_component(component: &str) -> Option<&'static str> {
    match component {
        "windows" | "win" => Some("windows"),
        "macos" | "mac" | "osx" => Some("macos"),
        "linux" => Some("linux"),
        _ => None,
    }
}

fn is_platform_component(component: &str) -> bool {
    platform_from_component(component).is_some()
}

/// Remove platform folder prefix from target path
/// Handles common platform name variations
fn remove_platform_parts(parts: &[String], platform: &str) -> String {
    // Build a list of platform folder names to remove based on the detected platform
    let platform_variants: Vec<&str> = match platform {
        "windows" => vec!["windows", "win"],
        "macos" => vec!["macos", "mac", "osx"],
        "linux" => vec!["linux"],
        _ => vec![],
    };

    // Find and remove any platform folder variant from the path
    let filtered_parts: Vec<&String> = parts
        .iter()
        .filter(|part| {
            let part_lower = part.to_lowercase();
            !platform_variants.contains(&part_lower.as_str())
        })
        .collect();

    filtered_parts
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_detect_platform_from_path_windows() {
        assert_eq!(
            detect_platform_from_parts(&vec!["windows".into(), "test.bundle".into()]),
            Some("windows".to_string())
        );
        assert_eq!(
            detect_platform_from_parts(&vec!["Windows".into(), "test.bundle".into()]),
            Some("windows".to_string())
        );
        assert_eq!(
            detect_platform_from_parts(&vec!["win".into(), "test.bundle".into()]),
            Some("windows".to_string())
        );
    }

    #[test]
    fn test_detect_platform_from_path_macos() {
        assert_eq!(
            detect_platform_from_parts(&vec!["macos".into(), "test.bundle".into()]),
            Some("macos".to_string())
        );
        assert_eq!(
            detect_platform_from_parts(&vec!["macOS".into(), "test.bundle".into()]),
            Some("macos".to_string())
        );
        assert_eq!(
            detect_platform_from_parts(&vec!["mac".into(), "test.bundle".into()]),
            Some("macos".to_string())
        );
        assert_eq!(
            detect_platform_from_parts(&vec!["osx".into(), "test.bundle".into()]),
            Some("macos".to_string())
        );
    }

    #[test]
    fn test_detect_platform_from_path_linux() {
        assert_eq!(
            detect_platform_from_parts(&vec!["linux".into(), "test.bundle".into()]),
            Some("linux".to_string())
        );
        assert_eq!(
            detect_platform_from_parts(&vec!["Linux".into(), "ui".into(), "test.bundle".into()]),
            Some("linux".to_string())
        );
    }

    #[test]
    fn test_detect_platform_from_path_no_platform() {
        assert_eq!(
            detect_platform_from_parts(&vec!["test.bundle".into()]),
            None
        );
        assert_eq!(
            detect_platform_from_parts(&vec!["ui".into(), "test.bundle".into()]),
            None
        );
    }

    #[test]
    fn test_remove_platform_prefix_windows() {
        assert_eq!(
            remove_platform_parts(&vec!["windows".into(), "test.bundle".into()], "windows"),
            "test.bundle"
        );
        assert_eq!(
            remove_platform_parts(
                &vec!["Windows".into(), "ui".into(), "test.bundle".into()],
                "windows"
            ),
            "ui/test.bundle"
        );
        assert_eq!(
            remove_platform_parts(&vec!["win".into(), "test.bundle".into()], "windows"),
            "test.bundle"
        );
    }

    #[test]
    fn test_remove_platform_prefix_macos() {
        assert_eq!(
            remove_platform_parts(&vec!["macos".into(), "test.bundle".into()], "macos"),
            "test.bundle"
        );
        assert_eq!(
            remove_platform_parts(
                &vec!["macOS".into(), "graphics".into(), "test.png".into()],
                "macos"
            ),
            "graphics/test.png"
        );
        assert_eq!(
            remove_platform_parts(&vec!["mac".into(), "test.bundle".into()], "macos"),
            "test.bundle"
        );
        assert_eq!(
            remove_platform_parts(
                &vec!["osx".into(), "ui".into(), "test.bundle".into()],
                "macos"
            ),
            "ui/test.bundle"
        );
    }

    #[test]
    fn test_remove_platform_prefix_linux() {
        assert_eq!(
            remove_platform_parts(&vec!["linux".into(), "test.bundle".into()], "linux"),
            "test.bundle"
        );
    }

    #[test]
    fn test_generate_manifest_removes_top_level_folder_and_platform() {
        let temp_dir = std::env::temp_dir().join(format!("test_manifest_{}", uuid::Uuid::new_v4()));
        let nested_root = temp_dir.join("tinyhips-darkmode-v4.5_fm26.0.5");
        let mac_dir = nested_root.join("MacOS");
        fs::create_dir_all(&mac_dir).expect("Failed to create MacOS dir");

        let mut file =
            fs::File::create(mac_dir.join("ui-styles_assets_common.bundle")).expect("create file");
        file.write_all(b"test content").expect("write content");
        drop(file);

        let result = generate_manifest(
            &temp_dir,
            "Nested Mod".to_string(),
            "1.0.0".to_string(),
            "ui".to_string(),
            "Author".to_string(),
            "Desc".to_string(),
        );

        assert!(result.is_ok(), "generate_manifest should succeed");

        let manifest_path = temp_dir.join("manifest.json");
        let manifest_content = fs::read_to_string(&manifest_path).expect("read manifest");
        let manifest: crate::types::ModManifest =
            serde_json::from_str(&manifest_content).expect("parse manifest");

        let mac_file = manifest
            .files
            .iter()
            .find(|f| f.source.contains("MacOS/ui-styles_assets_common.bundle"));

        assert!(mac_file.is_some(), "Mac file should be present");

        if let Some(file) = mac_file {
            assert_eq!(file.source, "MacOS/ui-styles_assets_common.bundle");
            assert_eq!(file.target_subpath, "ui-styles_assets_common.bundle");
            assert_eq!(file.platform, Some("macos".to_string()));
        }

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_generate_manifest_with_platform_folders() {
        let temp_dir = std::env::temp_dir().join(format!("test_manifest_{}", uuid::Uuid::new_v4()));

        // Create test structure with platform folders
        let windows_dir = temp_dir.join("windows");
        fs::create_dir_all(&windows_dir).expect("Failed to create windows dir");

        let mut file =
            fs::File::create(windows_dir.join("test.bundle")).expect("Failed to create file");
        file.write_all(b"test content")
            .expect("Failed to write content");
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
        let manifest: crate::types::ModManifest =
            serde_json::from_str(&manifest_content).expect("Failed to parse manifest");

        // Verify platform-specific file entry
        assert!(!manifest.files.is_empty(), "Manifest should have files");

        let windows_file = manifest.files.iter().find(|f| f.source.contains("windows"));
        assert!(
            windows_file.is_some(),
            "Should have a file from windows folder"
        );

        if let Some(file) = windows_file {
            assert_eq!(
                file.platform,
                Some("windows".to_string()),
                "File should have windows platform tag"
            );
            assert!(
                !file.target_subpath.contains("windows"),
                "Target path should not contain 'windows' folder"
            );
        }

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_generate_manifest_without_platform_folders() {
        let temp_dir = std::env::temp_dir().join(format!(
            "test_manifest_no_platform_{}",
            uuid::Uuid::new_v4()
        ));
        // Create a nested top-level folder to mimic extracted ZIP root
        let nested_root = temp_dir.join("my_mod_root");
        fs::create_dir_all(&nested_root).expect("Failed to create temp dir");

        // Create a regular file without platform folder
        let mut file =
            fs::File::create(nested_root.join("test.bundle")).expect("Failed to create file");
        file.write_all(b"test content")
            .expect("Failed to write content");
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
        let manifest: crate::types::ModManifest =
            serde_json::from_str(&manifest_content).expect("Failed to parse manifest");

        // Verify no platform tags
        for file in &manifest.files {
            assert_eq!(
                file.platform, None,
                "Files should not have platform tags when no platform folders exist"
            );
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
