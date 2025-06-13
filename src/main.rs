use anyhow::{Context, Result};
use std::fs;
use vento::{
    AppError, Authentication, AuthenticationMethod, DestinationType, Profile, TransferProfile,
};

fn main() -> Result<()> {
    let yaml = fs::read_to_string("config.yaml")?;
    let profiles: Profile = serde_yaml::from_str(&yaml)?;

    for profile in profiles.transfer_profiles {
        process_transfer_profile(profile)?;
    }

    Ok(())
}

fn process_transfer_profile(profile: TransferProfile) -> Result<()> {
    // 1. バリデーション
    profile.source.validate()?;
    profile.destination.validate()?;

    // 2. デバッグ出力
    println!("{:?}", profile);

    // 3. SFTP秘密鍵の取得と出力 (必要な場合のみ)
    if profile.destination.kind == DestinationType::Sftp {
        if let Some(auth) = &profile.destination.authentication {
            handle_sftp_authentication(auth)?;
        }
    }

    Ok(())
}

fn handle_sftp_authentication(auth: &Authentication) -> Result<()> {
    if matches!(
        auth.method,
        AuthenticationMethod::PrivateKey | AuthenticationMethod::EnvKey
    ) {
        if let Some(key_ref) = &auth.private_key_ref {
            let private_key = std::env::var(key_ref)
                .map_err(|_| AppError::EnvVarNotFound(key_ref.clone()))
                .context(format!(
                    "Failed to get private key from environment variable '{}'",
                    key_ref
                ))?;
            println!("Private Key (from env var {}):", key_ref);
            // ここで private_key を使用してSFTP接続を確立
            println!("{}", private_key);
        }
    }
    Ok(())
}
