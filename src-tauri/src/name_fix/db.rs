//! Database path detection for FM26 across different platforms.

use std::path::{Path, PathBuf};

/// Get the FM26 database directory based on game installation path (target_path)
///
/// The database directory structure differs by platform:
/// - Windows: <game_root>/shared/data/database/db/2600/
/// - macOS: <game_root>/fm.app/Contents/PlugIns/game_plugin.bundle/Contents/Resources/shared/data/database/db/2600/
/// - Linux: <game_root>/shared/data/database/db/2600/
pub fn get_db_dir(target_path: Option<&str>) -> Result<PathBuf, String> {
    let target_path = target_path
        .ok_or("Game target path not set. Please detect or set your FM26 game directory first.")?;

    let game_target = PathBuf::from(target_path);

    if !game_target.exists() {
        return Err(format!(
            "Game target path does not exist: {}",
            game_target.display()
        ));
    }

    get_db_dir_for_platform(&game_target)
}

#[cfg(target_os = "windows")]
fn get_db_dir_for_platform(game_target: &Path) -> Result<PathBuf, String> {
    fn build_db_dir(root: &Path) -> PathBuf {
        root.join("shared")
            .join("data")
            .join("database")
            .join("db")
            .join("2600")
    }

    let mut game_root_candidate = game_target.to_path_buf();
    let mut db_dir = build_db_dir(&game_root_candidate);

    if !db_dir.exists() {
        // Try to discover game root by walking up from a StreamingAssets-style path.
        let mut current = Some(game_target);
        while let Some(p) = current {
            if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                if name.eq_ignore_ascii_case("StreamingAssets") {
                    if let Some(root) = p.parent().and_then(|p| p.parent()) {
                        game_root_candidate = root.to_path_buf();
                        db_dir = build_db_dir(&game_root_candidate);
                    }
                    break;
                }
            }
            current = p.parent();
        }
    }

    if !db_dir.exists() {
        return Err(format!(
            "FM26 database directory not found at: {}. Please ensure FM26 is installed and you've launched it at least once.",
            db_dir.display()
        ));
    }

    Ok(db_dir)
}

#[cfg(target_os = "macos")]
fn get_db_dir_for_platform(game_target: &Path) -> Result<PathBuf, String> {
    fn build_db_dir(fm_app: &Path) -> PathBuf {
        fm_app
            .join("Contents")
            .join("PlugIns")
            .join("game_plugin.bundle")
            .join("Contents")
            .join("Resources")
            .join("shared")
            .join("data")
            .join("database")
            .join("db")
            .join("2600")
    }

    let mut fm_app_candidate: Option<PathBuf> = None;

    // 1) If target_path is the fm.app itself
    if game_target
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.eq_ignore_ascii_case("fm.app"))
        .unwrap_or(false)
    {
        fm_app_candidate = Some(game_target.to_path_buf());
    }

    // 2) If target_path is a game root containing fm.app
    if fm_app_candidate.is_none() {
        let candidate = game_target.join("fm.app");
        if candidate.exists() {
            fm_app_candidate = Some(candidate);
        }
    }

    // 3) Walk ancestors to find fm.app (if user picked a path inside the bundle)
    if fm_app_candidate.is_none() {
        let mut current = Some(game_target);
        while let Some(p) = current {
            if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                if name.eq_ignore_ascii_case("fm.app") {
                    fm_app_candidate = Some(p.to_path_buf());
                    break;
                }
            }
            current = p.parent();
        }
    }

    // 4) If it's a StreamingAssets path, walk back up to the app bundle
    if fm_app_candidate.is_none() {
        let mut current = Some(game_target);
        while let Some(p) = current {
            if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                if name.eq_ignore_ascii_case("StreamingAssets") {
                    // StreamingAssets -> Data -> Resources -> Contents -> fm.app
                    if let Some(contents_dir) =
                        p.parent().and_then(|p| p.parent()).and_then(|p| p.parent())
                    {
                        if let Some(fm_app) = contents_dir.parent() {
                            fm_app_candidate = Some(fm_app.to_path_buf());
                        }
                    }
                    break;
                }
            }
            current = p.parent();
        }
    }

    let fm_app =
        fm_app_candidate.ok_or("Could not determine fm.app directory from the provided path")?;

    let db_dir = build_db_dir(&fm_app);

    if !db_dir.exists() {
        return Err(format!(
            "FM26 database directory not found at: {}. Please ensure FM26 is installed and you've launched it at least once.",
            db_dir.display()
        ));
    }

    Ok(db_dir)
}

#[cfg(target_os = "linux")]
fn get_db_dir_for_platform(game_target: &Path) -> Result<PathBuf, String> {
    fn build_db_dir(root: &Path) -> PathBuf {
        root.join("shared")
            .join("data")
            .join("database")
            .join("db")
            .join("2600")
    }

    let mut game_root_candidate = game_target.to_path_buf();
    let mut db_dir = build_db_dir(&game_root_candidate);

    if !db_dir.exists() {
        // Try to discover game root by walking up from a StreamingAssets-style path.
        let mut current = Some(game_target);
        while let Some(p) = current {
            if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                if name.eq_ignore_ascii_case("StreamingAssets") {
                    if let Some(root) = p
                        .parent() // fm_Data or data
                        .and_then(|p| p.parent())
                    // game root
                    {
                        game_root_candidate = root.to_path_buf();
                        db_dir = build_db_dir(&game_root_candidate);
                    }
                    break;
                }
            }
            current = p.parent();
        }
    }

    if !db_dir.exists() {
        return Err(format!(
            "FM26 database directory not found at: {}. Please ensure FM26 is installed and you've launched it at least once.",
            db_dir.display()
        ));
    }

    Ok(db_dir)
}
