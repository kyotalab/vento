use config::{Config, ConfigError, File};
use etcetera::{BaseStrategy, choose_base_strategy};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct AppConfig {
    pub default_profile_file: Option<String>,
    pub log_level: Option<String>,
    pub log_file: Option<String>,
}

impl AppConfig {
    pub fn load_config() -> Result<AppConfig, ConfigError> {
        let strategy = choose_base_strategy().expect("Unable to find the config directory!");
        let mut path = strategy.config_dir();
        path.push("vento");
        path.push("config.yaml");

        if !path.exists() {
            eprintln!("No config file found at: {}", path.display());
        }

        let builder = Config::builder().add_source(File::from(path));

        builder.build()?.try_deserialize()
    }
}
