use fmmloader26::{config, mod_manager, types::FileEntry};
use std::env;
use std::path::PathBuf;

fn parse_args() -> (Vec<String>, Option<String>, Option<String>, Vec<String>) {
    let mut mod_types = Vec::new();
    let mut target_path = None;
    let mut user_dir = None;
    let mut files = Vec::new();

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--mod-type" | "-m" => {
                if let Some(value) = args.next() {
                    mod_types.push(value);
                }
            }
            "--file" | "-f" => {
                if let Some(value) = args.next() {
                    files.push(value);
                }
            }
            "--target-path" | "-t" => {
                if let Some(value) = args.next() {
                    target_path = Some(value);
                }
            }
            "--user-dir" | "-u" => {
                if let Some(value) = args.next() {
                    user_dir = Some(value);
                }
            }
            "--help" | "-h" => {
                println!(
                    "Usage: fmml-path-debug [options]\n\n\
                     Options:\n  \
                     -m, --mod-type <type>    Mod type to preview (repeatable)\n  \
                     -t, --target-path <path> Override game target path\n  \
                     -u, --user-dir <path>    Override FM user directory\n  \
                     -f, --file <subpath>     Target subpath to preview (repeatable)\n"
                );
                std::process::exit(0);
            }
            _ => {}
        }
    }

    if mod_types.is_empty() {
        mod_types = vec![
            "bundle".to_string(),
            "ui".to_string(),
            "graphics".to_string(),
            "tactics".to_string(),
            "editor-data".to_string(),
        ];
    }

    (mod_types, target_path, user_dir, files)
}

fn ensure_dir(path: &PathBuf) {
    let _ = std::fs::create_dir_all(path);
}

fn default_files_for(mod_type: &str) -> Vec<String> {
    match mod_type {
        "graphics" => vec![
            "graphics/faces/preview.png".to_string(),
            "graphics/logos/logo.png".to_string(),
        ],
        "tactics" => vec!["tactics/4-3-3-custom.fmf".to_string()],
        "editor-data" => vec!["editor data/custom_rules.fmf".to_string()],
        "ui" => vec![
            "panels/client_object/browser.xml".to_string(),
            "skins/fmml/graphics/pictures/person/picture_person.xml".to_string(),
        ],
        "bundle" => vec!["data/ui.ui-bundle".to_string()],
        _ => vec!["misc/sample.txt".to_string()],
    }
}

fn main() {
    let (mod_types, target_override, user_override, file_subpaths) = parse_args();

    let config = match config::load_config() {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("Failed to load config: {}", err);
            std::process::exit(1);
        }
    };

    let target_path = target_override
        .or(config.target_path.clone())
        .unwrap_or_else(|| {
            eprintln!("No target path set. Provide --target-path or set it in config.");
            std::process::exit(1);
        });

    let user_dir = user_override
        .or(config.user_dir_path.clone())
        .inspect(|path| {
            let buf = PathBuf::from(path);
            ensure_dir(&buf);
        });

    let game_target = PathBuf::from(&target_path);
    ensure_dir(&game_target);

    println!("FMMLoader Path Preview");
    println!("Target base: {}", game_target.display());
    if let Some(ref user) = user_dir {
        println!("User dir: {}", user);
    } else {
        println!("User dir: <not set>");
    }
    println!();

    for mod_type in mod_types {
        let targets = if file_subpaths.is_empty() {
            default_files_for(&mod_type)
        } else {
            file_subpaths.clone()
        };

        let files: Vec<FileEntry> = targets
            .iter()
            .map(|subpath| FileEntry {
                source: subpath.clone(),
                target_subpath: subpath.clone(),
                platform: None,
            })
            .collect();

        let preview =
            mod_manager::preview_mod_install(&mod_type, &game_target, user_dir.as_deref(), &files);

        println!("Mod type: {}", mod_type);
        println!("  Base: {}", preview.base_target);
        for file in preview.resolved_files {
            println!("  - {} -> {}", file.target_subpath, file.resolved_path);
        }
        println!();
    }
}
