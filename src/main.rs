use std::path::Path;

use anyhow::{Context, Result};
use chrono;
use clap::Parser;
use log::{LevelFilter, error, info};
use vento::{AppConfig, Cli, Profile, dispatch};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config_path = &cli.config;

    // `--config` オプションが指定された場合は、参照するconfigファイルをオーバーライドする
    let app_config: AppConfig = if let Some(path) = config_path {
        AppConfig::over_ride_config(path)?
    } else {
        AppConfig::load_config().context("Failed to load default application configuration")?
    };

    let profile_path = &app_config.default_profile_file;
    if profile_path.is_none() {
        error!("No default profile file specified in the config.");
        std::process::exit(1)
    }

    setup_logging(&app_config)?;

    let profile_path = profile_path.as_ref().unwrap();
    info!("Using profile file: {}", profile_path);
    let profiles = Profile::load_profiles(Path::new(&profile_path))?;

    let result = dispatch(cli, profiles);

    if let Err(e) = &result {
        error!("Application error: {:?}", e); // エラーをログに出力
    }

    result
}

fn setup_logging(app_config: &AppConfig) -> Result<()> {
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
