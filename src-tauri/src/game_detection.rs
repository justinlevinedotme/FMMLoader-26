use std::path::PathBuf;

pub fn get_default_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    #[cfg(target_os = "windows")]
    {
        let program_files_x86 = std::env::var("PROGRAMFILES(X86)")
            .unwrap_or_else(|_| "C:/Program Files (x86)".to_string());
        let program_files =
            std::env::var("PROGRAMFILES").unwrap_or_else(|_| "C:/Program Files".to_string());

        // Steam
        let steam_base = PathBuf::from(&program_files_x86)
            .join("Steam")
            .join("steamapps")
            .join("common")
            .join("Football Manager 26");

        for sub in &[
            "fm_Data/StreamingAssets/aa/StandaloneWindows64",
            "data/StreamingAssets/aa/StandaloneWindows64",
        ] {
            let path = steam_base.join(sub);
            if path.exists() {
                candidates.push(path);
            }
        }

        // Epic Games
        let epic_base = PathBuf::from(&program_files)
            .join("Epic Games")
            .join("Football Manager 26");

        for sub in &[
            "fm_Data/StreamingAssets/aa/StandaloneWindows64",
            "data/StreamingAssets/aa/StandaloneWindows64",
        ] {
            let path = epic_base.join(sub);
            if path.exists() {
                candidates.push(path);
            }
        }

        // Xbox Game Pass - check C:, D:, E: drives
        for drive in &["C:", "D:", "E:"] {
            let gamepass_base = PathBuf::from(drive)
                .join("XboxGames")
                .join("Football Manager 26")
                .join("Content");

            if gamepass_base.exists() {
                for sub in &[
                    "fm_Data/StreamingAssets/aa/StandaloneWindows64",
                    "data/StreamingAssets/aa/StandaloneWindows64",
                ] {
                    let path = gamepass_base.join(sub);
                    if path.exists() {
                        candidates.push(path);
                    }
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir().unwrap();

        let paths = vec![
            home.join("Library/Application Support/Steam/steamapps/common/Football Manager 26/fm.app/Contents/Resources/Data/StreamingAssets/aa/StandaloneOSX"),
            home.join("Library/Application Support/Steam/steamapps/common/Football Manager 26/fm_Data/StreamingAssets/aa/StandaloneOSXUniversal"),
            home.join("Library/Application Support/Epic/Football Manager 26/fm_Data/StreamingAssets/aa/StandaloneOSXUniversal"),
        ];

        for path in paths {
            if path.exists() {
                candidates.push(path);
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let home = dirs::home_dir().unwrap();

        let paths = vec![
            home.join(".local/share/Steam/steamapps/common/Football Manager 26/fm_Data/StreamingAssets/aa/StandaloneLinux64"),
            PathBuf::from("/run/media/mmcblk0p1/steamapps/common/Football Manager 26/fm_Data/StreamingAssets/aa/StandaloneLinux64"),
        ];

        for path in paths {
            if path.exists() {
                candidates.push(path);
            }
        }
    }

    candidates
}

pub fn get_fm_user_dir(custom_path: Option<&str>) -> PathBuf {
    // Check if user has set a custom path
    if let Some(path) = custom_path {
        let custom = PathBuf::from(path);
        if custom.exists() {
            return custom;
        }
    }

    // Default paths
    #[cfg(target_os = "windows")]
    {
        dirs::home_dir()
            .unwrap()
            .join("Documents")
            .join("Sports Interactive")
            .join("Football Manager 26")
    }

    #[cfg(target_os = "macos")]
    {
        dirs::home_dir()
            .unwrap()
            .join("Library")
            .join("Application Support")
            .join("Sports Interactive")
            .join("Football Manager 26")
    }

    #[cfg(target_os = "linux")]
    {
        dirs::home_dir()
            .unwrap()
            .join(".local")
            .join("share")
            .join("Sports Interactive")
            .join("Football Manager 26")
    }
}
