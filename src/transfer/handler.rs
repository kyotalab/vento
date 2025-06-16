use crate::{AppError, ProtocolType, SftpHandler, TransferProfile, TransferProtocolHandler};
use anyhow::Result;

pub fn process_transfer_profile(profile: TransferProfile) -> Result<()> {
    // 1. バリデーション
    profile.source.validate()?;
    profile.destination.validate()?;

    // 2. デバッグ出力
    // println!("{:?}", profile);

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
