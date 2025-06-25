use crate::{
    execute_command, AppError, ProtocolType, ScpHandler, SftpHandler, SourceType, TransferProfile,
    TransferProtocolHandler,
};
use anyhow::Result;
use log::{error, info};

pub async fn process_transfer_profile(profile: TransferProfile) -> Result<()> {
    // Validation
    profile.source.validate()?;
    profile.destination.validate()?;

    // Execute pre transfer command
    if let Some(pre_job) = &profile.pre_transfer_command {
        execute_command(pre_job, &profile.profile_id, "pre-transfer").await?;
    }

    // Execute transfer
    let transfer_result: Result<()> = match profile.transfer_protocol.protocol {
        ProtocolType::Sftp => {
            let handler = SftpHandler;

            match profile.source.kind {
                SourceType::Local => handler.send(&profile).await,
                SourceType::Sftp => handler.receive(&profile).await,
                _ => {
                    return Err(AppError::Validation("Unsupported transfer source type".into()).into());
                }
            }
        }
        // 将来のプロトコル（例：Scp, Httpなど）
        // Future protocols (e.g. Scp, Http, etc.)
        ProtocolType::Scp => {
            let handler = ScpHandler;

            match profile.source.kind {
                SourceType::Local => handler.send(&profile).await,
                SourceType::Scp => handler.receive(&profile).await,
                _ => {
                    return Err(AppError::Validation("Unsupported transfer source type".into()).into());
                }
            }
        }
        // _ => {
        //     return Err(AppError::Validation("Unsupported transfer protocol".into()).into());
        // }
    };

    // Execute post transfer or on-error command
    match transfer_result {
        Ok(_) => {
            // 転送が成功した場合
            // If the transfer was successful
            info!(
                "File transfer completed successfully for profile '{}'.",
                profile.profile_id
            );
            if let Some(post_job) = &profile.post_transfer_command {
                // post_job が失敗しても、転送自体は成功なので、エラーとして返すかどうかは要件次第
                // ここでは post_job の失敗もエラーとして伝播させる。
                // Even if post_job fails, the transfer itself is successful, so whether to return it as an error is up to your requirements.
                // Here, we'll also propagate post_job failures as errors.
                execute_command(post_job, &profile.profile_id, "post-transfer").await?
            }
            Ok(())
        }
        Err(e) => {
            // 転送が失敗した場合
            // If the transfer fails
            error!(
                "File transfer failed for profile '{}'. Error: {:?}",
                profile.profile_id, e
            );
            if let Some(on_error_job) = &profile.on_error_command {
                // on_error_job の実行は、メインのエラーとは別に扱うことが多い
                // ここで on_error_job が失敗しても、元の転送エラーを優先してログに出し、
                // 元のエラーを返すべきか、on_error_job のエラーを優先すべきか検討。
                // 一般的には、元のエラーを報告しつつ、on_error_job の成功/失敗もログに残す。
                // Execution of on_error_job is often handled separately from the main error.
                // Even if on_error_job fails here, the original transfer error is given priority and logged,
                // Consider whether to return the original error or the on_error_job error.
                // Generally, the original error is reported and the success/failure of on_error_job is also logged.
                match execute_command(on_error_job, &profile.profile_id, "on-error").await {
                    Ok(_) => {
                        info!(
                            "On-error command executed successfully for profile '{}'.",
                            profile.profile_id
                        );
                        // 元の転送エラーを再スロー
                        // Re-throw the original forwarding error
                        Err(e)
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
                // on_error_command が定義されていなければ、そのまま元のエラーを返す
                // If on_error_command is not defined, return the original error.
                Err(e)
            }
        }
    }
}
