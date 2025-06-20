pub mod cli;
pub mod config;
pub mod error;
pub mod profile;
pub mod transfer;

use anyhow::{Context, Result};
pub use cli::*;
pub use config::*;
pub use error::*;
use log::LevelFilter;
pub use profile::*;
pub use transfer::*;

pub fn setup_logging(app_config: &AppConfig) -> Result<()> {
    let level = app_config.log_level.as_deref().unwrap_or("info");
    let log_level = match level.to_lowercase().as_str() {
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Info, // デフォルト
    };

    let mut base_config = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                // ★この部分を修正★
                "{} [{}] [{}] {}", // [日付 時間][レベル][モジュール] メッセージ
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log_level);

    let should_log_to_stdout = app_config.log_stdout.unwrap_or(true); // デフォルトをtrueにする
    if should_log_to_stdout {
        base_config = base_config.chain(std::io::stdout());
    }

    // ログファイル出力が指定されている場合
    if let Some(log_file) = &app_config.log_file {
        let log_file_path = std::path::PathBuf::from(log_file);
        let log_dir = log_file_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));

        // ログディレクトリが存在しない場合は作成
        if !log_dir.exists() {
            std::fs::create_dir_all(log_dir).context(format!(
                "Failed to create log directory: {}",
                log_dir.display()
            ))?;
        }

        base_config = base_config.chain(fern::log_file(log_file_path)?);
    }

    base_config.apply()?; // ロガーを適用

    Ok(())
}
