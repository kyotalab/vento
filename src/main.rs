use std::fs;

use anyhow::Result;
use clap::Parser;
use vento::{Cli, Profile, dispatch};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config_path = &cli.config;
    let profiles = load_profiles(config_path)?;
    // println!("{:?}", profiles);
    // Ok(())
    dispatch(cli, profiles)
}

fn load_profiles(config: &str) -> Result<Profile> {
    let yaml = fs::read_to_string(config)?;
    let profiles: Profile = serde_yaml::from_str(&yaml)?;
    Ok(profiles)
}
