//! Name fix management for Football Manager 26.
//!
//! This module handles importing, installing, and uninstalling name fixes
//! that correct licensing issues in FM26.

mod backup;
mod db;
mod import;
mod install;

pub use import::import_name_fix;
pub use install::{install_name_fix, uninstall};

// Re-export common constants at module level
pub use constants::GITHUB_NAME_FIX_ID;

use crate::config::{get_name_fixes_dir, load_config};
use crate::types::NameFixSource;
use std::fs;

/// Constants used across the name fix module
pub mod constants {
    pub const GITHUB_NAME_FIX_ID: &str = "github-namefix";

    /// Files to delete as part of the installation
    pub const FILES_TO_DELETE: &[(&str, &[&str])] = &[
        // From lnc/all/
        (
            "lnc/all",
            &[
                "ac milan (wom).lnc",
                "acc Inter unlic 26.lnc",
                "acmilan unlic 26.lnc",
                "fake.lnc",
                "inter (wom).lnc",
                "inter unlic 26.lnc",
                "japanese names.lnc",
                "lazio (wom).lnc",
                "lic_dan_swe_fra.lnc",
                "licensing club name_fm26.lnc",
                "men lazio.lnc",
                "men.atalanta.lnc",
            ],
        ),
        // From edt/permanent/
        ("edt/permanent", &["fake.edt"]),
        // From dbc/permanent/
        (
            "dbc/permanent",
            &[
                "1_japan_removed_clubs.dbc",
                "brazil_kits.dbc",
                "england.dbc",
                "forbidden names.dbc",
                "france.dbc",
                "japan_fake.dbc",
                "japan_unlicensed_random_names.dbc",
                "j league non player.dbc",
                "licensing_post_data_lock.dbc",
                "licensing2.dbc",
                "licensing26.dbc",
                "netherlands.dbc",
            ],
        ),
        // From dbc/language/
        ("dbc/language", &["Licensing2.dbc", "Licensing2_chn.dbc"]),
    ];
}

/// Check if FM Name Fix is installed
/// Returns true if there's an active name fix in the config
pub fn check_installed(_target_path: Option<&str>) -> Result<bool, String> {
    let config = load_config()?;
    Ok(config.active_name_fix.is_some())
}

/// Get list of all available name fix sources
pub fn list_name_fixes() -> Result<Vec<NameFixSource>, String> {
    let name_fixes_dir = get_name_fixes_dir();
    let mut sources = Vec::new();

    // Scan for imported name fixes
    if name_fixes_dir.exists() {
        if let Ok(entries) = fs::read_dir(&name_fixes_dir) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    // Check if this directory has a metadata.json file
                    let metadata_file = entry_path.join("metadata.json");
                    if metadata_file.exists() {
                        if let Ok(metadata_str) = fs::read_to_string(&metadata_file) {
                            if let Ok(source) = serde_json::from_str::<NameFixSource>(&metadata_str)
                            {
                                sources.push(source);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(sources)
}

/// Delete an imported name fix
pub fn delete_name_fix(name_fix_id: String) -> Result<String, String> {
    if name_fix_id == constants::GITHUB_NAME_FIX_ID {
        return Err("Cannot delete the built-in GitHub name fix".to_string());
    }

    let fix_dir = get_name_fixes_dir().join(&name_fix_id);
    if !fix_dir.exists() {
        return Err("Name fix not found".to_string());
    }

    // If this is the active name fix, uninstall it first
    let config = load_config()?;
    if config.active_name_fix.as_deref() == Some(&name_fix_id) {
        uninstall()?;
    }

    // Delete the directory
    fs::remove_dir_all(&fix_dir).map_err(|e| format!("Failed to delete name fix: {}", e))?;

    tracing::info!("Deleted name fix: {}", name_fix_id);
    Ok("Name fix deleted successfully".to_string())
}

/// Get the currently active name fix ID
pub fn get_active_name_fix() -> Result<Option<String>, String> {
    let config = load_config()?;
    Ok(config.active_name_fix)
}
