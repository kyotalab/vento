use crate::{
    AppError, ProtocolType, SftpHandler, SourceType, TransferProfile, TransferProtocolHandler,
};
use anyhow::{Context, Result};
use log::{debug, error, info};
use tokio::process::Command;

pub async fn process_transfer_profile(profile: TransferProfile) -> Result<()> {
    // 1. バリデーション
    profile.source.validate()?;
    profile.destination.validate()?;

    if let Some(pre_job) = &profile.pre_transfer_command {
        // 転送前ジョブ実行
        info!(
            "Executing pre-transfer command for profile '{}': {}",
            profile.profile_id, pre_job
        );
        let status = Command::new("sh")
            .arg("-c")
            .arg(pre_job)
            .status()
            .await
            .with_context(|| {
                format!(
                    "Failed to execute pre-transfer command for profile '{}'",
                    profile.profile_id
                )
            })?;
        if !status.success() {
            error!(
                "Pre-transfer command failed with status: {:?} for profile '{}'",
                status.code(),
                profile.profile_id
            );
            return Err(anyhow::anyhow!("Pre-transfer command failed"));
        }
        info!(
            "Pre-transfer command completed successfully for profile '{}'.",
            profile.profile_id
        );
    }

    // 2. プロトコルに応じた handler を選んで転送実行
    match profile.transfer_protocol.protocol {
        ProtocolType::Sftp => {
            let handler = SftpHandler;

            match profile.source.kind {
                SourceType::Local => handler.send(&profile).await?,
                SourceType::Sftp => handler.receive(&profile).await?,
            }
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

    if let Some(post_job) = &profile.post_transfer_command {
        info!(
            "Executing post-transfer command for profile '{}': {}",
            profile.profile_id, post_job
        );
        let status = Command::new("sh")
            .arg("-c")
            .arg(post_job)
            .status()
            .await
            .with_context(|| {
                format!(
                    "Failed to execute post-transfer command for profile '{}'",
                    profile.profile_id
                )
            })?;

        if !status.success() {
            error!(
                "Post-transfer command failed with status: {:?} for profile '{}'",
                status.code(),
                profile.profile_id
            );
            // 転送後のジョブ失敗の場合、転送自体は成功しているので、エラーを返すかどうかは要件次第
            // ここでは一旦エラーとして伝播させます
            return Err(anyhow::anyhow!("Post-transfer command failed"));
        }
        info!(
            "Post-transfer command completed successfully for profile '{}'.",
            profile.profile_id
        );
    }

    Ok(())
}
