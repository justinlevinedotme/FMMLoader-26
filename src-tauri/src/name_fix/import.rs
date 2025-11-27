//! Import name fixes from ZIP files.

use crate::config::get_name_fixes_dir;
use crate::types::{NameFixInstallType, NameFixSource, NameFixSourceType};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

/// Import a name fix from a ZIP file
pub fn import_name_fix(file_path: String, name: String) -> Result<String, String> {
    tracing::info!("=== IMPORT NAME FIX CALLED ===");
    tracing::info!("File path: {}", file_path);
    tracing::info!("Name: {}", name);

    let source_path = PathBuf::from(&file_path);
    if !source_path.exists() {
        tracing::error!("Source file does not exist: {:?}", source_path);
        return Err("Source file does not exist".to_string());
    }

    tracing::info!("Source file exists: {:?}", source_path);

    // Detect the install type from ZIP structure
    let install_type = detect_install_type(&source_path)?;
    tracing::info!("Detected install type: {:?}", install_type);

    // Generate a unique ID for this name fix
    let id = format!("imported-{}", uuid::Uuid::new_v4());
    tracing::info!("Generated ID: {}", id);

    // Create directory for this name fix
    let name_fixes_dir = get_name_fixes_dir();
    tracing::info!("Name fixes directory: {:?}", name_fixes_dir);

    let fix_dir = name_fixes_dir.join(&id);
    tracing::info!("Creating directory: {:?}", fix_dir);

    fs::create_dir_all(&fix_dir)
        .map_err(|e| format!("Failed to create name fix directory: {}", e))?;

    tracing::info!("Directory created successfully");

    // Extract files based on install type
    tracing::info!("Starting extraction...");
    let file_count = match install_type {
        NameFixInstallType::Files => extract_files_type(&source_path, &fix_dir)?,
        NameFixInstallType::Folders => extract_folders_type(&source_path, &fix_dir)?,
    };
    tracing::info!("Extraction complete, {} items extracted", file_count);

    // Create metadata
    let type_desc = match install_type {
        NameFixInstallType::Files => "File-based",
        NameFixInstallType::Folders => "Folder-based",
    };

    let source = NameFixSource {
        id: id.clone(),
        name: name.clone(),
        source_type: NameFixSourceType::Imported,
        install_type,
        description: format!(
            "{} - Imported from {} ({} items)",
            type_desc,
            source_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy(),
            file_count
        ),
        imported_date: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };

    // Save metadata
    let metadata_file = fix_dir.join("metadata.json");
    let metadata_json = serde_json::to_string_pretty(&source)
        .map_err(|e| format!("Failed to serialize metadata: {}", e))?;
    fs::write(&metadata_file, metadata_json)
        .map_err(|e| format!("Failed to save metadata: {}", e))?;

    tracing::info!("Name fix imported successfully: {}", name);
    Ok(format!(
        "Successfully imported '{}' ({}) with {} items",
        name, type_desc, file_count
    ))
}

/// Detect whether this is a file-based or folder-based name fix
fn detect_install_type(zip_path: &Path) -> Result<NameFixInstallType, String> {
    let file = fs::File::open(zip_path).map_err(|e| format!("Failed to open file: {}", e))?;

    let mut archive =
        ZipArchive::new(file).map_err(|e| format!("Failed to read ZIP archive: {}", e))?;

    let mut has_folders = false;
    let mut has_editor_data = false;
    let mut has_individual_files = false;

    for i in 0..archive.len() {
        let file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read ZIP entry: {}", e))?;

        let name = file.name();

        // Check for Sortitoutsi style (folders + editor data) at any depth
        if name.contains("dbc/") || name.contains("edt/") || name.contains("lnc/") {
            has_folders = true;
        }
        if name.contains("editor data/") {
            has_editor_data = true;
        }

        // Check for FMScout style (individual .lnc/.edt/.dbc files in root or shallow depth)
        // These files should not be deep in the folder structure
        let path_depth = name.matches('/').count();
        if path_depth <= 1
            && (name.ends_with(".lnc") || name.ends_with(".edt") || name.ends_with(".dbc"))
        {
            has_individual_files = true;
        }
    }

    // Sortitoutsi if it has the folder structure and editor data
    if has_folders && has_editor_data {
        Ok(NameFixInstallType::Folders)
    } else if has_individual_files || has_folders {
        // FMScout if it has individual files or just folders without editor data
        Ok(NameFixInstallType::Files)
    } else {
        Err("Could not determine name fix type. ZIP should contain either .lnc/.edt/.dbc files or dbc/edt/lnc folders.".to_string())
    }
}

/// Extract file-based name fix (FMScout style)
fn extract_files_type(zip_path: &Path, dest_dir: &Path) -> Result<usize, String> {
    tracing::info!("Extracting file-based name fix");
    extract_all_namefix_files(zip_path, dest_dir)
}

/// Extract all name fix files from a ZIP archive
/// Extracts all .lnc, .edt, and .dbc files from the ZIP
fn extract_all_namefix_files(zip_path: &Path, dest_dir: &Path) -> Result<usize, String> {
    tracing::info!(
        "Extracting name fix files from: {:?} to {:?}",
        zip_path,
        dest_dir
    );

    let file = fs::File::open(zip_path).map_err(|e| format!("Failed to open file: {}", e))?;

    let mut archive =
        ZipArchive::new(file).map_err(|e| format!("Failed to read ZIP archive: {}", e))?;

    tracing::info!("ZIP archive contains {} entries", archive.len());

    let mut file_count = 0;

    // Extract all .lnc, .edt, and .dbc files
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read ZIP entry {}: {}", i, e))?;

        let file_name = file.name().to_string(); // Clone the name before using the file

        tracing::debug!("Processing ZIP entry: {}", file_name);

        // Skip directories
        if file_name.ends_with('/') {
            tracing::debug!("Skipping directory: {}", file_name);
            continue;
        }

        // Check if this is a relevant file type
        if file_name.ends_with(".lnc") || file_name.ends_with(".edt") || file_name.ends_with(".dbc")
        {
            // Extract just the filename (remove any directory structure from the ZIP)
            let path = PathBuf::from(&file_name);
            let filename = path
                .file_name()
                .ok_or_else(|| format!("Invalid file name in archive: {}", file_name))?;

            let dest_file = dest_dir.join(filename);

            let mut contents = Vec::new();
            file.read_to_end(&mut contents)
                .map_err(|e| format!("Failed to read file from archive: {}", e))?;

            fs::write(&dest_file, &contents).map_err(|e| format!("Failed to write file: {}", e))?;

            tracing::info!(
                "Extracted: {} ({} bytes) -> {:?}",
                filename.to_string_lossy(),
                contents.len(),
                dest_file
            );
            file_count += 1;
        } else {
            tracing::debug!("Skipping non-namefix file: {}", file_name);
        }
    }

    if file_count == 0 {
        return Err(
            "No valid name fix files (.lnc, .edt, or .dbc) found in ZIP archive".to_string(),
        );
    }

    tracing::info!("Successfully extracted {} files", file_count);
    Ok(file_count)
}

/// Extract folder-based name fix (Sortitoutsi style)
/// Extracts dbc, edt, lnc folders and editor data folder
fn extract_folders_type(zip_path: &Path, dest_dir: &Path) -> Result<usize, String> {
    tracing::info!("Extracting folder-based name fix (Sortitoutsi style)");

    let file = fs::File::open(zip_path).map_err(|e| format!("Failed to open file: {}", e))?;

    let mut archive =
        ZipArchive::new(file).map_err(|e| format!("Failed to read ZIP archive: {}", e))?;

    tracing::info!("ZIP archive contains {} entries", archive.len());

    let mut item_count = 0;

    // Extract all files, preserving folder structure but stripping leading path
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read ZIP entry {}: {}", i, e))?;

        let file_name = file.name().to_string();

        // Skip directories
        if file_name.ends_with('/') {
            continue;
        }

        // Check if this file is in one of the target folders (at any depth)
        let relevant_path = if let Some(idx) = file_name.find("dbc/") {
            Some(&file_name[idx..])
        } else if let Some(idx) = file_name.find("edt/") {
            Some(&file_name[idx..])
        } else if let Some(idx) = file_name.find("lnc/") {
            Some(&file_name[idx..])
        } else {
            file_name.find("editor data/").map(|idx| &file_name[idx..])
        };

        if let Some(rel_path) = relevant_path {
            let dest_file = dest_dir.join(rel_path);

            // Create parent directories
            if let Some(parent) = dest_file.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            }

            let mut contents = Vec::new();
            file.read_to_end(&mut contents)
                .map_err(|e| format!("Failed to read file from archive: {}", e))?;

            fs::write(&dest_file, &contents).map_err(|e| format!("Failed to write file: {}", e))?;

            tracing::debug!("Extracted: {} -> {:?}", file_name, dest_file);
            item_count += 1;
        }
    }

    if item_count == 0 {
        return Err(
            "No valid name fix folders (dbc/, edt/, lnc/, editor data/) found in ZIP archive"
                .to_string(),
        );
    }

    tracing::info!("Successfully extracted {} items", item_count);
    Ok(item_count)
}
