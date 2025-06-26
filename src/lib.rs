pub mod cli;
pub mod config;
pub mod error;
pub mod profile;
pub mod transfer;
pub mod util;

use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{Context, Result};
pub use cli::*;
pub use config::*;
pub use error::*;
use log::LevelFilter;
pub use profile::*;
pub use transfer::*;
pub use util::*;

pub static MAX_FILE_SIZE_MB: OnceLock<AtomicU64> = OnceLock::new();


pub fn init_max_file_size_mb(val: u64) {
    MAX_FILE_SIZE_MB.set(AtomicU64::new(val)).expect("Already initialized");
}

pub fn get_max_file_size_mb() -> u64 {
    MAX_FILE_SIZE_MB
        .get()
        .expect("MAX_FILE_SIZE_MB not initialized")
        .load(Ordering::Relaxed)
}

pub fn setup_logging(app_config: &AppConfig) -> Result<()> {
    let level = app_config.log_level.as_deref().unwrap_or("info");
    let log_level = match level.to_lowercase().as_str() {
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Info, // default
    };

    let mut base_config = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] [{}] {}", // [Date Time][Level][Module] Message
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log_level);

    let should_log_to_stdout = app_config.log_stdout.unwrap_or(true); // Set default to true
    if should_log_to_stdout {
        base_config = base_config.chain(std::io::stdout());
    }

    // ログファイル出力が指定されている場合
    // If log file output is specified
    if let Some(log_file) = &app_config.log_file {
        let log_file_path = std::path::PathBuf::from(log_file);
        let log_dir = log_file_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));

        // ログディレクトリが存在しない場合は作成
        // If the log directory does not exist, create it.
        if !log_dir.exists() {
            std::fs::create_dir_all(log_dir).context(format!(
                "Failed to create log directory: {}",
                log_dir.display()
            ))?;
        }

        base_config = base_config.chain(fern::log_file(log_file_path)?);
    }

    base_config.apply()?;

    Ok(())
}
