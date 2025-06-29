use std::fs;

use anyhow::{Context, Result};
use config::{Config, File};
use etcetera::{choose_base_strategy, BaseStrategy};
use log::error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub default_profile_file: Option<String>,
    pub log_level: Option<String>,
    pub log_file: Option<String>,
    pub log_stdout: Option<bool>,
    pub max_file_size_mb: Option<u64>,
}

impl AppConfig {
    pub fn load_config() -> Result<AppConfig> {
        let strategy = choose_base_strategy().context("Unable to find the config directory!")?;
        let mut path = strategy.config_dir();
        path.push("vento");
        path.push("config.yaml");

        if !path.exists() {
            error!("No config file found at: {}", path.display());
        }

        let builder = Config::builder().add_source(File::from(path));

        builder
            .build()?
            .try_deserialize()
            .context("Failed to deserialize AppConfig")
    }

    pub fn over_ride_config(path: &str) -> Result<AppConfig> {
        let yaml = fs::read_to_string(path)?;
        let app_config: AppConfig = serde_yaml::from_str(&yaml)?;
        Ok(app_config)
    }
}
