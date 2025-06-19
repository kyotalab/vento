use std::fs;

use anyhow::Result;
use clap::Parser;
use vento::{AppConfig, Cli, Profile, dispatch};

fn main() -> Result<()> {
    let app_config = AppConfig::load_config().unwrap_or_else(|e| {
        eprintln!("Failed to load config: {}", e);
        std::process::exit(1);
    });

    let cli = Cli::parse();
    let profile_path = &app_config.default_profile_file;
    if profile_path.is_none() {
        eprintln!("No default profile file specified in the config.");
        std::process::exit(1);
    }
    let profile_path = profile_path.as_ref().unwrap();
    let profiles = load_profiles(&profile_path)?;

    dispatch(cli, profiles)
}

fn load_profiles(path: &str) -> Result<Profile> {
    let yaml = fs::read_to_string(path)?;
    let profiles: Profile = serde_yaml::from_str(&yaml)?;
    Ok(profiles)
}
