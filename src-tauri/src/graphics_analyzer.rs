//! Graphics Pack Analysis Module
//!
//! This module provides intelligent analysis and classification of FM graphics packs.
//! It detects pack types (faces, logos, kits, or mixed), calculates confidence scores,
//! and handles both flat and structured pack layouts.
//!
//! # Pack Type Detection
//!
//! The analyzer uses multiple signals to determine pack type:
//! - **High confidence**: config.xml mappings (person portraits, team logos, team kits)
//! - **Medium confidence**: Directory structure (faces/, logos/, kits/ subdirectories)
//!
//! # Supported Pack Structures
//!
//! **Flat Packs**: PNGs and config.xml at root with no type-specific subdirectories
//! - Example: `megapack/12345.png`, `megapack/config.xml`
//!
//! **Structured Packs**: Type-specific subdirectories containing assets
//! - Example: `megapack/faces/12345.png`, `megapack/logos/clubs/45.png`
//!
//! **Mixed Packs**: Contains multiple types (faces + logos + kits)
//! - Can be split into separate directories per type
//!
//! # Usage
//!
//! ```rust,ignore
//! use graphics_analyzer::analyze_graphics_pack;
//!
//! let analysis = analyze_graphics_pack(&pack_path)?;
//! println!("Type: {:?}, Confidence: {}", analysis.pack_type, analysis.confidence);
//! ```
//!
//! # Security
//!
//! The analyzer limits directory traversal depth to 3 levels to prevent excessive
//! processing on malformed pack structures.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum GraphicsPackType {
    Faces,
    Logos,
    Kits,
    Mixed(Vec<GraphicsPackType>), // Contains multiple types
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsPackAnalysis {
    pub pack_type: GraphicsPackType,
    pub confidence: f32,              // 0.0 to 1.0
    pub suggested_paths: Vec<String>, // e.g., ["graphics/faces/PackName"]
    pub file_count: usize,
    pub total_size_bytes: u64,
    pub has_config_xml: bool,
    pub subdirectory_breakdown: HashMap<String, usize>, // type -> file count
    pub is_flat_pack: bool, // true if PNGs are at root with no subdirectories
}

#[derive(Debug)]
struct PackContents {
    has_faces_dir: bool,
    has_logos_dir: bool,
    has_kits_dir: bool,
    png_files: Vec<PathBuf>,
    xml_files: Vec<PathBuf>,
    #[allow(dead_code)]
    subdirs: Vec<PathBuf>,
    total_size: u64,
}

/// Analyzes a graphics pack directory to determine its type and routing
pub fn analyze_graphics_pack(pack_path: &Path) -> Result<GraphicsPackAnalysis, String> {
    if !pack_path.exists() {
        return Err(format!("Pack path does not exist: {}", pack_path.display()));
    }

    // Gather pack contents
    let contents = scan_pack_contents(pack_path)?;

    // Check for config.xml and parse it
    let config_analysis = analyze_config_xml(pack_path);

    // Determine type based on multiple signals
    let (pack_type, confidence) = determine_pack_type(&contents, &config_analysis);

    // Generate suggested installation paths
    let suggested_paths = generate_suggested_paths(&pack_type, pack_path);

    // Create breakdown by subdirectory
    let subdirectory_breakdown = analyze_subdirectories(pack_path, &pack_type)?;

    // Detect if this is a flat pack
    let is_flat_pack = detect_flat_pack(&contents);

    Ok(GraphicsPackAnalysis {
        pack_type,
        confidence,
        suggested_paths,
        file_count: contents.png_files.len(),
        total_size_bytes: contents.total_size,
        has_config_xml: !contents.xml_files.is_empty(),
        subdirectory_breakdown,
        is_flat_pack,
    })
}

/// Detects if a pack is "flat" - PNGs and config.xml at root with no type-specific subdirectories
fn detect_flat_pack(contents: &PackContents) -> bool {
    // Flat pack criteria:
    // 1. Has image files at the root level
    // 2. Does NOT have type-specific subdirectories (faces/, logos/, kits/)
    // 3. Usually has config.xml at root

    let has_images_at_root = !contents.png_files.is_empty();
    let has_type_subdirs =
        contents.has_faces_dir || contents.has_logos_dir || contents.has_kits_dir;

    has_images_at_root && !has_type_subdirs
}

fn scan_pack_contents(pack_path: &Path) -> Result<PackContents, String> {
    let mut png_files = Vec::new();
    let mut xml_files = Vec::new();
    let mut subdirs = Vec::new();
    let mut total_size = 0u64;

    let mut has_faces_dir = false;
    let mut has_logos_dir = false;
    let mut has_kits_dir = false;

    for entry in WalkDir::new(pack_path).max_depth(3) {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(name) = path.file_name() {
                let name_lower = name.to_string_lossy().to_lowercase();
                if name_lower == "faces" || name_lower.contains("face") {
                    has_faces_dir = true;
                }
                if name_lower == "logos"
                    || name_lower.contains("logo")
                    || name_lower.contains("badge")
                {
                    has_logos_dir = true;
                }
                if name_lower == "kits" || name_lower.contains("kit") {
                    has_kits_dir = true;
                }
                subdirs.push(path.to_path_buf());
            }
        } else if path.is_file() {
            if let Some(ext) = path.extension() {
                let ext_lower = ext.to_string_lossy().to_lowercase();
                match ext_lower.as_str() {
                    "png" | "jpg" | "jpeg" => {
                        png_files.push(path.to_path_buf());
                        if let Ok(metadata) = fs::metadata(path) {
                            total_size += metadata.len();
                        }
                    }
                    "xml" => {
                        xml_files.push(path.to_path_buf());
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(PackContents {
        has_faces_dir,
        has_logos_dir,
        has_kits_dir,
        png_files,
        xml_files,
        subdirs,
        total_size,
    })
}

#[derive(Debug, Default)]
struct ConfigAnalysis {
    has_person_portraits: bool,
    has_team_logos: bool,
    has_team_kits: bool,
    mapping_count: usize,
}

fn analyze_config_xml(pack_path: &Path) -> ConfigAnalysis {
    let config_path = pack_path.join("config.xml");
    if !config_path.exists() {
        return ConfigAnalysis::default();
    }

    let mut analysis = ConfigAnalysis::default();

    if let Ok(content) = fs::read_to_string(&config_path) {
        // Count mapping types by looking for patterns
        let lines: Vec<&str> = content.lines().collect();

        for line in &lines {
            if line.contains("graphics/pictures/person") && line.contains("portrait") {
                analysis.has_person_portraits = true;
                analysis.mapping_count += 1;
            }
            if line.contains("graphics/pictures/team") && line.contains("logo") {
                analysis.has_team_logos = true;
                analysis.mapping_count += 1;
            }
            if line.contains("graphics/pictures/team") && line.contains("kit") {
                analysis.has_team_kits = true;
                analysis.mapping_count += 1;
            }
        }
    }

    analysis
}

fn determine_pack_type(
    contents: &PackContents,
    config: &ConfigAnalysis,
) -> (GraphicsPackType, f32) {
    let mut type_scores: HashMap<GraphicsPackType, f32> = HashMap::new();

    // High confidence signals from config.xml
    if config.has_person_portraits {
        *type_scores.entry(GraphicsPackType::Faces).or_insert(0.0) += 0.8;
    }
    if config.has_team_logos {
        *type_scores.entry(GraphicsPackType::Logos).or_insert(0.0) += 0.8;
    }
    if config.has_team_kits {
        *type_scores.entry(GraphicsPackType::Kits).or_insert(0.0) += 0.8;
    }

    // Medium confidence signals from directory structure
    if contents.has_faces_dir {
        *type_scores.entry(GraphicsPackType::Faces).or_insert(0.0) += 0.5;
    }
    if contents.has_logos_dir {
        *type_scores.entry(GraphicsPackType::Logos).or_insert(0.0) += 0.5;
    }
    if contents.has_kits_dir {
        *type_scores.entry(GraphicsPackType::Kits).or_insert(0.0) += 0.5;
    }

    // Check if it's a mixed pack
    let significant_types: Vec<GraphicsPackType> = type_scores
        .iter()
        .filter(|(_, &score)| score >= 0.5)
        .map(|(t, _)| t.clone())
        .collect();

    if significant_types.len() > 1 {
        let max_score = type_scores.values().cloned().fold(0.0f32, f32::max);
        return (GraphicsPackType::Mixed(significant_types), max_score);
    }

    // Single type pack
    if let Some((pack_type, &score)) = type_scores
        .iter()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
    {
        if score >= 0.5 {
            return (pack_type.clone(), score);
        }
    }

    // Default to unknown with low confidence
    (GraphicsPackType::Unknown, 0.0)
}

fn generate_suggested_paths(pack_type: &GraphicsPackType, _pack_path: &Path) -> Vec<String> {
    // Return only type directories - contents will be installed directly into these
    match pack_type {
        GraphicsPackType::Faces => vec!["faces".to_string()],
        GraphicsPackType::Logos => vec!["logos".to_string()],
        GraphicsPackType::Kits => vec!["kits".to_string()],
        GraphicsPackType::Mixed(types) => {
            let mut paths = Vec::new();
            for t in types {
                match t {
                    GraphicsPackType::Faces => {
                        if !paths.contains(&"faces".to_string()) {
                            paths.push("faces".to_string());
                        }
                    }
                    GraphicsPackType::Logos => {
                        if !paths.contains(&"logos".to_string()) {
                            paths.push("logos".to_string());
                        }
                    }
                    GraphicsPackType::Kits => {
                        if !paths.contains(&"kits".to_string()) {
                            paths.push("kits".to_string());
                        }
                    }
                    _ => {}
                }
            }
            paths
        }
        GraphicsPackType::Unknown => {
            vec!["logos".to_string(), "faces".to_string(), "kits".to_string()]
        }
    }
}

fn analyze_subdirectories(
    pack_path: &Path,
    pack_type: &GraphicsPackType,
) -> Result<HashMap<String, usize>, String> {
    let mut breakdown = HashMap::new();

    match pack_type {
        GraphicsPackType::Mixed(types) => {
            // For mixed packs, count files in each type-specific subdirectory
            for entry in WalkDir::new(pack_path).max_depth(2) {
                let entry = entry.map_err(|e| e.to_string())?;
                let path = entry.path();

                if path.is_dir() {
                    if let Some(name) = path.file_name() {
                        let name_lower = name.to_string_lossy().to_lowercase();

                        for t in types {
                            let type_name = match t {
                                GraphicsPackType::Faces => "faces",
                                GraphicsPackType::Logos => "logos",
                                GraphicsPackType::Kits => "kits",
                                _ => continue,
                            };

                            if name_lower.contains(type_name) {
                                let file_count = count_image_files(path)?;
                                breakdown.insert(type_name.to_string(), file_count);
                            }
                        }
                    }
                }
            }
        }
        _ => {
            // Single type pack - just count all files
            let total = count_image_files(pack_path)?;
            let type_name = match pack_type {
                GraphicsPackType::Faces => "faces",
                GraphicsPackType::Logos => "logos",
                GraphicsPackType::Kits => "kits",
                GraphicsPackType::Unknown => "unknown",
                _ => "other",
            };
            breakdown.insert(type_name.to_string(), total);
        }
    }

    Ok(breakdown)
}

fn count_image_files(dir: &Path) -> Result<usize, String> {
    let mut count = 0;
    for entry in WalkDir::new(dir) {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.path().is_file() {
            if let Some(ext) = entry.path().extension() {
                let ext_lower = ext.to_string_lossy().to_lowercase();
                if matches!(ext_lower.as_str(), "png" | "jpg" | "jpeg" | "tga" | "bmp") {
                    count += 1;
                }
            }
        }
    }
    Ok(count)
}

/// Splits a mixed graphics pack into separate type-specific directories
/// Returns a map of type -> source directory path
pub fn split_mixed_pack(
    pack_path: &Path,
    analysis: &GraphicsPackAnalysis,
) -> Result<HashMap<String, PathBuf>, String> {
    let mut split_map: HashMap<String, PathBuf> = HashMap::new();

    // Only process if this is actually a mixed pack
    let types = match &analysis.pack_type {
        GraphicsPackType::Mixed(types) => types,
        _ => return Err("Pack is not a mixed type".to_string()),
    };

    // Find subdirectories that correspond to each type
    for entry in WalkDir::new(pack_path).max_depth(2) {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if !path.is_dir() || path == pack_path {
            continue;
        }

        if let Some(name) = path.file_name() {
            let name_lower = name.to_string_lossy().to_lowercase();

            for pack_type in types {
                let (type_key, type_patterns) = match pack_type {
                    GraphicsPackType::Faces => ("faces", vec!["faces", "face"]),
                    GraphicsPackType::Logos => ("logos", vec!["logos", "logo", "badges", "badge"]),
                    GraphicsPackType::Kits => ("kits", vec!["kits", "kit"]),
                    _ => continue,
                };

                // Check if directory name matches this type
                if type_patterns
                    .iter()
                    .any(|pattern| name_lower.contains(pattern))
                {
                    split_map.insert(type_key.to_string(), path.to_path_buf());
                }
            }
        }
    }

    // If we couldn't identify separate subdirectories, the pack might be flat
    // In this case, we need to analyze the config.xml more carefully
    if split_map.is_empty() {
        // For flat packs, we can't really split them - they should stay together
        // Return the whole pack as each type (the config.xml will handle routing)
        for pack_type in types {
            let type_key = match pack_type {
                GraphicsPackType::Faces => "faces",
                GraphicsPackType::Logos => "logos",
                GraphicsPackType::Kits => "kits",
                _ => continue,
            };
            split_map.insert(type_key.to_string(), pack_path.to_path_buf());
        }
    }

    Ok(split_map)
}

/// Checks if a pack can be split based on its structure
#[allow(dead_code)]
pub fn can_split_pack(analysis: &GraphicsPackAnalysis) -> bool {
    matches!(analysis.pack_type, GraphicsPackType::Mixed(_))
}

/// Gets installation targets for a graphics pack based on its type
#[allow(dead_code)]
pub fn get_installation_targets(
    pack_name: &str,
    analysis: &GraphicsPackAnalysis,
    graphics_base_dir: &Path,
) -> Vec<(String, PathBuf)> {
    let mut targets = Vec::new();

    match &analysis.pack_type {
        GraphicsPackType::Faces => {
            targets.push((
                "faces".to_string(),
                graphics_base_dir.join("faces").join(pack_name),
            ));
        }
        GraphicsPackType::Logos => {
            targets.push((
                "logos".to_string(),
                graphics_base_dir.join("logos").join(pack_name),
            ));
        }
        GraphicsPackType::Kits => {
            targets.push((
                "kits".to_string(),
                graphics_base_dir.join("kits").join(pack_name),
            ));
        }
        GraphicsPackType::Mixed(types) => {
            for pack_type in types {
                match pack_type {
                    GraphicsPackType::Faces => {
                        targets.push((
                            "faces".to_string(),
                            graphics_base_dir
                                .join("faces")
                                .join(format!("{}-Faces", pack_name)),
                        ));
                    }
                    GraphicsPackType::Logos => {
                        targets.push((
                            "logos".to_string(),
                            graphics_base_dir
                                .join("logos")
                                .join(format!("{}-Logos", pack_name)),
                        ));
                    }
                    GraphicsPackType::Kits => {
                        targets.push((
                            "kits".to_string(),
                            graphics_base_dir
                                .join("kits")
                                .join(format!("{}-Kits", pack_name)),
                        ));
                    }
                    _ => {}
                }
            }
        }
        GraphicsPackType::Unknown => {
            targets.push(("unknown".to_string(), graphics_base_dir.join(pack_name)));
        }
    }

    targets
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_type_serialization() {
        let mixed = GraphicsPackType::Mixed(vec![GraphicsPackType::Faces, GraphicsPackType::Logos]);

        let json = serde_json::to_string(&mixed).unwrap();
        let deserialized: GraphicsPackType = serde_json::from_str(&json).unwrap();

        assert_eq!(mixed, deserialized);
    }
}
