use crate::config::get_mods_dir;
use crate::game_detection::get_fm_user_dir;
use crate::mod_manager::{get_target_for_type, read_manifest};
use crate::types::ConflictInfo;
use std::collections::HashMap;
use std::path::PathBuf;

pub fn find_conflicts(
    enabled_mods: &[String],
    game_target: &PathBuf,
    user_dir: Option<&str>,
) -> Result<Vec<ConflictInfo>, String> {
    let mods_dir = get_mods_dir();
    let mut file_to_mods: HashMap<String, Vec<String>> = HashMap::new();

    // Build index of which mods touch which files
    for mod_name in enabled_mods {
        let mod_dir = mods_dir.join(mod_name);

        if !mod_dir.exists() {
            continue;
        }

        let manifest = match read_manifest(&mod_dir) {
            Ok(m) => m,
            Err(_) => continue,
        };

        let target_base = get_target_for_type(&manifest.mod_type, game_target, user_dir);

        for file_entry in &manifest.files {
            let target_path = target_base.join(&file_entry.target_subpath);
            let target_str = target_path.to_string_lossy().to_string();

            file_to_mods
                .entry(target_str)
                .or_insert_with(Vec::new)
                .push(mod_name.clone());
        }
    }

    // Find files touched by multiple mods
    let mut conflicts = Vec::new();

    for (file_path, mods) in file_to_mods {
        if mods.len() > 1 {
            conflicts.push(ConflictInfo {
                file_path,
                conflicting_mods: mods,
            });
        }
    }

    Ok(conflicts)
}

pub fn build_mod_index(mod_name: &str) -> Result<Vec<String>, String> {
    let mod_dir = get_mods_dir().join(mod_name);

    if !mod_dir.exists() {
        return Err(format!("Mod not found: {}", mod_name));
    }

    let manifest = read_manifest(&mod_dir)?;
    let files: Vec<String> = manifest
        .files
        .iter()
        .map(|f| f.target_subpath.clone())
        .collect();

    Ok(files)
}
