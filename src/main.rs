use std::path::Path;

use anyhow::{Context, Result};
use clap::Parser;
use log::{error, info};
use vento::{AppConfig, Cli, Profile, dispatch, setup_logging};

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
