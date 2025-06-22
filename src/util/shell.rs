use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use tokio::process::Command;

/// ターゲットOSに応じて適切なシェルと引数を返すヘルパー関数
#[cfg(target_os = "windows")]
fn get_os_shell_command() -> (&'static str, &'static str) {
    ("cmd.exe", "/C")
}

/// ターゲットOSに応じて適切なシェルと引数を返すヘルパー関数
#[cfg(not(target_os = "windows"))] // Windows以外のOS (Linux, macOSなど)
fn get_os_shell_command() -> (&'static str, &'static str) {
    ("sh", "-c") // Unix系OSでは'sh -c'が一般的
}

pub async fn execute_command(command_str: &str, profile_id: &str, job_type: &str) -> Result<()> {
    info!(
        "Executing {} command for profile '{}': {}",
        job_type, profile_id, command_str
    );

    let (shell, arg) = get_os_shell_command(); // OSに応じたシェルと引数を取得

    let output = Command::new(shell) // 例: Windowsなら "cmd.exe"
        .arg(arg) // 例: Windowsなら "/C"
        .arg(command_str) // ユーザーが定義したコマンド文字列
        .output()
        .await
        .with_context(|| {
            format!(
                "Failed to execute {} command for profile '{}'",
                job_type, profile_id
            )
        })?;

    let stdout_trimmed = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr_trimmed = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if !stdout_trimmed.is_empty() {
        debug!(
            "{} command Stdout for profile '{}': {}",
            job_type, profile_id, stdout_trimmed
        );
    } else {
        debug!(
            "{} command Stdout for profile '{}' was empty.",
            job_type, profile_id
        );
    }

    if !stderr_trimmed.is_empty() {
        warn!(
            "{} command Stderr for profile '{}': {}",
            job_type, profile_id, stderr_trimmed
        );
    } else {
        debug!(
            "{} command Stderr for profile '{}' was empty.",
            job_type, profile_id
        );
    }

    if !output.status.success() {
        error!(
            "{} command failed for profile '{}'. Code: {:?}",
            job_type,
            profile_id,
            output.status.code()
        );
        return Err(anyhow::anyhow!(
            "{} command failed for profile '{}'",
            job_type,
            profile_id
        ));
    } else {
        info!(
            "{} command completed successfully for profile '{}'.",
            job_type, profile_id
        );
    }
    Ok(())
}
