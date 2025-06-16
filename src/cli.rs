use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::{AppError, Profile, process_transfer_profile};

#[derive(Debug, Parser)]
pub struct Cli {
    #[arg(short, long)]
    pub config: String,
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
}

pub fn dispatch(cli: Cli, profiles: Profile) -> Result<()> {
    match cli.command {
        Commands::Transfer { profile_id } => {
            // 1. profile_id に該当する TransferProfile を探す
            match profiles
                .transfer_profiles
                .into_iter()
                .find(|p| p.profile_id == profile_id)
            {
                Some(profile) => process_transfer_profile(profile),
                None => {
                    return Err(AppError::Validation(format!(
                        "Profile '{}' not found in config.yaml",
                        profile_id
                    ))
                    .into());
                }
            }
        }
    }
}
