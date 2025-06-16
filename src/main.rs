use anyhow::Result;
use std::fs;
use vento::{
    AppError, Profile, ProtocolType, SftpHandler, TransferProfile, TransferProtocolHandler,
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

    // 3. プロトコルに応じた handler を選んで転送実行
    match profile.transfer_protocol.protocol {
        ProtocolType::Sftp => {
            let handler = SftpHandler;
            handler.send(&profile)?;
        }
        // 将来のプロトコル（例：Scp, Httpなど）
        // TransferProtocolType::Scp => {
        //     let handler = ScpHandler;
        //     handler.send(&profile)?;
        // }
        _ => {
            return Err(AppError::Validation("Unsupported transfer protocol".into()).into());
        }
    }

    Ok(())
}
