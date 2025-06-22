use crate::{
    AppError, ProtocolType, SftpHandler, SourceType, TransferProfile, TransferProtocolHandler,
    execute_command,
};
use anyhow::Result;
use log::{error, info};

pub async fn process_transfer_profile(profile: TransferProfile) -> Result<()> {
    // 1. バリデーション
    profile.source.validate()?;
    profile.destination.validate()?;

    if let Some(pre_job) = &profile.pre_transfer_command {
        execute_command(pre_job, &profile.profile_id, "pre-transfer").await?;
    }

    // 2. プロトコルに応じた handler を選んで転送実行
    let transfer_result: Result<()> = match profile.transfer_protocol.protocol {
        ProtocolType::Sftp => {
            let handler = SftpHandler;

            match profile.source.kind {
                SourceType::Local => handler.send(&profile).await,
                SourceType::Sftp => handler.receive(&profile).await,
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
    };

    // 転送結果に基づいたジョブの実行
    match transfer_result {
        Ok(_) => {
            // 転送が成功した場合
            info!(
                "File transfer completed successfully for profile '{}'.",
                profile.profile_id
            );
            if let Some(post_job) = &profile.post_transfer_command {
                // post_job が失敗しても、転送自体は成功なので、エラーとして返すかどうかは要件次第
                // ここでは post_job の失敗もエラーとして伝播させます
                execute_command(post_job, &profile.profile_id, "post-transfer").await?
            }
            Ok(())
        }
        Err(e) => {
            // 転送が失敗した場合
            error!(
                "File transfer failed for profile '{}'. Error: {:?}",
                profile.profile_id, e
            );
            if let Some(on_error_job) = &profile.on_error_command {
                // on_error_job の実行は、メインのエラーとは別に扱うことが多い
                // ここで on_error_job が失敗しても、元の転送エラーを優先してログに出し、
                // 元のエラーを返すべきか、on_error_job のエラーを優先すべきか検討。
                // 一般的には、元のエラーを報告しつつ、on_error_job の成功/失敗もログに残す。
                match execute_command(on_error_job, &profile.profile_id, "on-error").await {
                    Ok(_) => {
                        info!(
                            "On-error command executed successfully for profile '{}'.",
                            profile.profile_id
                        );
                        Err(e) // 元の転送エラーを再スロー
                    }
                    Err(on_error_e) => {
                        error!(
                            "On-error command also failed for profile '{}'. Original transfer error: {:?}, On-error command error: {:?}",
                            profile.profile_id, e, on_error_e
                        );
                        Err(anyhow::anyhow!(
                            "Transfer failed and on-error command also failed: Original: {:?}, On-error: {:?}",
                            e,
                            on_error_e
                        ))
                    }
                }
            } else {
                Err(e) // on_error_command が定義されていなければ、そのまま元のエラーを返す
            }
        }
    }
}
