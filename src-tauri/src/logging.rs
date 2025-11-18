use std::fs;
use std::path::PathBuf;
use tracing_appender::rolling;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn get_logs_dir() -> PathBuf {
    let app_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("FMMLoader26")
        .join("logs");

    fs::create_dir_all(&app_dir).ok();
    app_dir
}

pub fn init_logging() -> Result<(), String> {
    let logs_dir = get_logs_dir();

    // Clean up old log files (keep last 10)
    cleanup_old_logs(&logs_dir, 10)?;

    // Create a file appender with daily rotation
    let file_appender = rolling::daily(&logs_dir, "fmmloader");

    // Set up the tracing subscriber with both file and stdout
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().with_writer(file_appender))
        .with(fmt::layer().with_writer(std::io::stdout))
        .init();

    // Log system information on startup
    log_system_info();

    Ok(())
}

fn log_system_info() {
    tracing::info!("=== FMMLoader26 Started ===");
    tracing::info!("Version: {}", env!("CARGO_PKG_VERSION"));
    tracing::info!("OS: {}", std::env::consts::OS);
    tracing::info!("Architecture: {}", std::env::consts::ARCH);
    tracing::info!("Family: {}", std::env::consts::FAMILY);

    if let Ok(hostname) = hostname::get() {
        tracing::info!("Hostname: {:?}", hostname);
    }

    tracing::info!("========================");
}

fn cleanup_old_logs(logs_dir: &PathBuf, keep_count: usize) -> Result<(), String> {
    if !logs_dir.exists() {
        return Ok(());
    }

    let mut log_files: Vec<_> = fs::read_dir(logs_dir)
        .map_err(|e| format!("Failed to read logs directory: {}", e))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().is_file() && entry.file_name().to_string_lossy().starts_with("fmmloader")
        })
        .collect();

    // Sort by modified time (newest first)
    log_files.sort_by(|a, b| {
        let a_time = a.metadata().and_then(|m| m.modified()).ok();
        let b_time = b.metadata().and_then(|m| m.modified()).ok();
        b_time.cmp(&a_time)
    });

    // Remove old files beyond keep_count
    for old_file in log_files.iter().skip(keep_count) {
        let _ = fs::remove_file(old_file.path());
    }

    Ok(())
}
