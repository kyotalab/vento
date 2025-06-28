use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::{process_transfer_profile, AppConfig, AppError, Profile};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long)]
    pub config: Option<String>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(name = "transfer")]
    #[command(about = "Transfer by profile in config.yaml")]
    Transfer {
        #[arg(short, long)]
        profile_id: String,
    },
    #[command(name = "admin")]
    #[command(about = "Manages configuration settings and transfer profile information")]
    Admin,
}

pub async fn dispatch(cli: Cli, profiles: Profile, app_config: AppConfig) -> Result<()> {
    match cli.command {
        Commands::Transfer { profile_id } => {
            // profile_id に該当する TransferProfile を探す
            // Find the TransferProfile that matches the profile_id
            match profiles
                .transfer_profiles
                .into_iter()
                .find(|p| p.profile_id == profile_id)
            {
                Some(profile) => process_transfer_profile(profile).await,
                None => {
                    return Err(AppError::Validation(format!(
                        "Profile '{}' not found in config.yaml",
                        profile_id
                    ))
                    .into());
                }
            }
        }
        Commands::Admin => {
            println!("admin command");
            println!("{:?}", app_config);
            Ok(())
        }
    }
}
